const selectors = {
  scenario: document.querySelector('[data-scenario]'),
  tick: document.getElementById('tick-counter'),
  days: document.getElementById('day-counter'),
  population: document.getElementById('population-counter'),
  starving: document.getElementById('starving-counter'),
  progress: document.getElementById('progress-fill'),
  status: document.getElementById('status-pill'),
  regionsGrid: document.getElementById('regions-grid'),
  eventFeed: document.getElementById('event-feed'),
  kpiEmployment: document.querySelector('#kpi-employment .value'),
  kpiReliability: document.querySelector('#kpi-reliability .value'),
  kpiInnovation: document.querySelector('#kpi-innovation .value'),
  kpiFinance: document.querySelector('#kpi-finance .value'),
};

const state = {
  totalTicks: 0,
  histories: new Map(),
  halo: {
    canvas: document.getElementById('halo-canvas'),
    ctx: null,
    phase: 0,
  },
  streaming: false,
  lastSnapshot: null,
  kpis: {
    employment: 0,
    reliability: 0,
    innovation: 0,
    stress: 0,
  },
};

const regionCache = new Map();
let eventSource;
let renderCycle = 0;

init();

function init() {
  if (state.halo.canvas) {
    state.halo.ctx = state.halo.canvas.getContext('2d');
    resizeHalo();
    window.addEventListener('resize', resizeHalo);
    requestAnimationFrame(drawHalo);
  }
  bootstrap();
  startEventStream();
}

async function bootstrap() {
  try {
    const response = await fetch('/api/state');
    if (!response.ok) throw new Error('Failed to fetch initial state');
    const payload = await response.json();
    state.totalTicks = payload.total_ticks || state.totalTicks;
    selectors.scenario.textContent = payload.scenario || 'unknown world';
    updateStatus(payload.completed ? 'complete' : 'warming');
    if (payload.frame) {
      renderFrame(payload.frame);
    }
  } catch (err) {
    console.error(err);
    updateStatus('error');
    setTimeout(bootstrap, 2500);
  }
}

function startEventStream() {
  if (eventSource) {
    eventSource.close();
  }
  eventSource = new EventSource('/api/events');
  eventSource.onmessage = (event) => {
    try {
      const frame = JSON.parse(event.data);
      state.streaming = true;
      updateStatus(frame.completed ? 'complete' : 'streaming');
      renderFrame(frame);
    } catch (err) {
      console.error('Failed to parse frame', err);
    }
  };
  eventSource.onerror = () => {
    updateStatus('reconnecting');
    eventSource.close();
    setTimeout(startEventStream, 2000);
  };
}

function renderFrame(frame) {
  if (!frame || !frame.snapshot) return;
  state.lastSnapshot = frame.snapshot;
  renderCycle += 1;
  updateMeta(frame.snapshot);
  updateKpis(frame.snapshot);
  updateEvents(frame.snapshot);
  updateRegions(frame.snapshot);
}

function updateMeta(snapshot) {
  selectors.tick.textContent = formatNumber(snapshot.tick);
  selectors.days.textContent = snapshot.days_elapsed.toFixed(1);
  selectors.population.textContent = formatNumber(snapshot.total_population);
  selectors.starving.textContent = snapshot.starving_regions.length;
  const progress = state.totalTicks > 0 ? snapshot.tick / state.totalTicks : 0;
  selectors.progress.style.width = `${Math.min(progress, 1) * 100}%`;
}

function updateKpis(snapshot) {
  const totals = snapshot.regions.reduce(
    (acc, region) => {
      acc.employment += 1 - region.unemployment_rate;
      acc.reliability += region.infrastructure_reliability;
      acc.innovation += region.innovation_score;
      acc.stress += region.credit_stress;
      return acc;
    },
    { employment: 0, reliability: 0, innovation: 0, stress: 0 }
  );
  const count = Math.max(snapshot.regions.length, 1);
  state.kpis = {
    employment: totals.employment / count,
    reliability: totals.reliability / count,
    innovation: totals.innovation / count,
    stress: totals.stress / count,
  };
  selectors.kpiEmployment.textContent = formatPercent(state.kpis.employment);
  selectors.kpiReliability.textContent = formatPercent(state.kpis.reliability);
  selectors.kpiInnovation.textContent = state.kpis.innovation.toFixed(2);
  selectors.kpiFinance.textContent = formatPercent(state.kpis.stress);
}

function updateEvents(snapshot) {
  const events = [];
  if (snapshot.starving_regions.length) {
    events.push({
      type: 'alert',
      text: `Food stress detected in <strong>${snapshot.starving_regions.join(', ')}</strong>.`,
    });
  }
  const highestStress = [...snapshot.regions].sort(
    (a, b) => b.credit_stress - a.credit_stress
  )[0];
  if (highestStress && highestStress.credit_stress > 0.25) {
    events.push({
      type: 'alert',
      text: `${highestStress.name} banking stress at <strong>${formatPercent(
        highestStress.credit_stress
      )}</strong>.`,
    });
  }
  const richestEnergy = [...snapshot.regions].sort(
    (a, b) => a.energy_shortage_ratio - b.energy_shortage_ratio
  )[0];
  if (richestEnergy) {
    events.push({
      type: 'info',
      text: `${richestEnergy.name} energy balance holding at <strong>${formatPercent(
        1 - richestEnergy.energy_shortage_ratio
      )}</strong>.`,
    });
  }
  if (!events.length) {
    events.push({ type: 'info', text: 'All subsystems nominal.' });
  }
  selectors.eventFeed.innerHTML = events
    .map(
      (event) =>
        `<div class="event ${event.type === 'alert' ? 'alert' : ''}">${event.text}</div>`
    )
    .join('');
}

function updateRegions(snapshot) {
  const regionOrder = [...snapshot.regions].sort((a, b) => a.id - b.id);
  regionOrder.forEach((region) => {
    const entry = ensureRegionCard(region);
    updateRegionCard(entry, region, snapshot.tick);
  });
}

function ensureRegionCard(region) {
  if (regionCache.has(region.id)) {
    return regionCache.get(region.id);
  }
  const card = document.createElement('div');
  card.className = 'region-card';
  card.dataset.regionId = region.id;

  const header = document.createElement('div');
  header.className = 'region-header';
  const heading = document.createElement('div');
  const title = document.createElement('div');
  title.className = 'region-title';
  const meta = document.createElement('div');
  meta.className = 'region-meta';
  heading.append(title, meta);

  const chip = document.createElement('div');
  chip.className = 'region-chip';
  header.append(heading, chip);

  const canvas = document.createElement('canvas');
  canvas.className = 'region-canvas';
  canvas.width = 320;
  canvas.height = 150;

  const stats = document.createElement('div');
  stats.className = 'region-stats';
  const statEmployment = createStatBlock('Employment');
  const statWage = createStatBlock('Wage');
  const statPower = createStatBlock('Power');
  const statBudget = createStatBlock('Budget');
  stats.append(statEmployment.root, statWage.root, statPower.root, statBudget.root);

  card.append(header, canvas, stats);
  selectors.regionsGrid.appendChild(card);

  const entry = {
    card,
    title,
    meta,
    chip,
    canvas,
    ctx: canvas.getContext('2d'),
    statEmployment,
    statWage,
    statPower,
    statBudget,
  };
  regionCache.set(region.id, entry);
  return entry;
}

function createStatBlock(labelText) {
  const root = document.createElement('div');
  root.className = 'stat-block';
  const label = document.createElement('div');
  label.className = 'label';
  label.textContent = labelText;
  const value = document.createElement('div');
  value.className = 'value';
  root.append(label, value);
  return { root, value };
}

function updateRegionCard(entry, region, tick) {
  entry.title.textContent = region.name;
  entry.meta.textContent = `${formatShortNumber(region.citizens)} citizens`; 
  entry.statEmployment.value.textContent = formatPercent(
    1 - region.unemployment_rate
  );
  entry.statWage.value.textContent = `${formatCurrency(region.wage)}`;
  entry.statPower.value.textContent = `${formatShortNumber(
    region.power_capacity
  )} MW`;
  entry.statBudget.value.textContent = formatCurrency(region.household_budget);
  const alerting = region.food_shortage_ratio > 0.15 || region.energy_shortage_ratio > 0.15;
  entry.chip.dataset.state = alerting ? 'alert' : 'stable';
  entry.chip.textContent = alerting ? 'Signal: Shortage' : 'Signal: Stable';
  drawPixelMatrix(entry.ctx, region, tick);
}

function drawPixelMatrix(ctx, region, tick) {
  if (!ctx) return;
  const width = ctx.canvas.width;
  const height = ctx.canvas.height;
  ctx.clearRect(0, 0, width, height);
  const cols = 24;
  const rows = 12;
  const total = cols * rows;
  const employment = clamp01(region.employed / Math.max(region.citizens || 1, 1));
  const food = clamp01(1 - region.food_shortage_ratio);
  const energy = clamp01(1 - region.energy_shortage_ratio);
  for (let i = 0; i < total; i += 1) {
    const row = Math.floor(i / cols);
    const col = i % cols;
    const ratio = i / total;
    let color;
    if (ratio < employment) {
      const glow = 0.35 + 0.55 * Math.abs(Math.sin((tick + i) * 0.08));
      color = `rgba(53, 255, 206, ${glow})`;
    } else if (ratio < employment + food * 0.3) {
      const glow = 0.25 + 0.5 * Math.abs(Math.sin((tick + i) * 0.04));
      color = `rgba(255, 220, 140, ${glow})`;
    } else {
      const glow = 0.2 + 0.5 * Math.abs(Math.sin((tick + i) * 0.06));
      color = `rgba(90, 190, 255, ${glow * energy})`;
    }
    ctx.fillStyle = color;
    ctx.fillRect(col * (width / cols), row * (height / rows), width / cols + 1, height / rows + 1);
  }
  if (region.credit_stress > 0.35) {
    ctx.strokeStyle = `rgba(255, 110, 158, ${region.credit_stress})`;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, height);
    ctx.lineTo(width, 0);
    ctx.stroke();
  }
}

function resizeHalo() {
  if (!state.halo.canvas) return;
  state.halo.canvas.width = state.halo.canvas.clientWidth;
  state.halo.canvas.height = state.halo.canvas.clientHeight;
}

function drawHalo() {
  const canvas = state.halo.canvas;
  const ctx = state.halo.ctx;
  if (!canvas || !ctx) return;
  const { width, height } = canvas;
  ctx.clearRect(0, 0, width, height);
  ctx.globalCompositeOperation = 'lighter';
  const centerX = width / 2;
  const centerY = height / 2;
  const maxRadius = Math.min(width, height) * 0.4;
  const reliability = state.kpis.reliability || 0.2;
  const innovation = state.kpis.innovation || 0.0;
  for (let i = 0; i < 4; i += 1) {
    const radius = maxRadius * (0.45 + i * 0.16);
    ctx.strokeStyle = `rgba(42, 245, 255, ${0.05 + reliability * (0.3 + i * 0.1)})`;
    ctx.lineWidth = 1.5 + i * 0.6;
    ctx.beginPath();
    const wobble = 0.2 + 0.1 * Math.sin(state.halo.phase + i);
    ctx.ellipse(
      centerX,
      centerY,
      radius,
      radius * (0.6 + wobble),
      state.halo.phase * (0.4 + i * 0.1),
      0,
      Math.PI * 2
    );
    ctx.stroke();
  }
  state.halo.phase += 0.01 + innovation * 0.0005;
  requestAnimationFrame(drawHalo);
}

function updateStatus(stateKey) {
  const map = {
    warming: 'Preparing Simulation',
    streaming: 'Streaming Live',
    complete: 'Simulation Complete',
    reconnecting: 'Reconnecting…',
    error: 'Connection Lost',
  };
  selectors.status.textContent = map[stateKey] || 'Observing';
  selectors.status.dataset.state = stateKey;
}

function formatNumber(value) {
  return value.toLocaleString('en-US');
}

function formatShortNumber(value) {
  if (value >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(1)}B`;
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return value.toString();
}

function formatPercent(value) {
  return `${(clamp01(value) * 100).toFixed(1)}%`;
}

function formatCurrency(value) {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    maximumFractionDigits: 0,
  }).format(value || 0);
}

function clamp01(value) {
  return Math.min(1, Math.max(0, value || 0));
}
