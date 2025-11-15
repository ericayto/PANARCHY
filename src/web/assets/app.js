const selectors = {
  scenario: document.querySelector('[data-scenario]'),
  status: document.getElementById('status-pill'),
  statusLabel: document.getElementById('status-label'),
  tick: document.getElementById('tick-counter'),
  days: document.getElementById('day-counter'),
  starving: document.getElementById('starving-counter'),
  population: document.getElementById('population-counter'),
  employment: document.getElementById('employment-counter'),
  reliability: document.getElementById('reliability-counter'),
  stress: document.getElementById('stress-counter'),
  regionsGrid: document.getElementById('regions-grid'),
  logTerminal: document.getElementById('log-terminal'),
  timelineSlider: document.getElementById('timeline-slider'),
  timelineLabel: document.getElementById('timeline-label'),
  speedLabel: document.getElementById('speed-label'),
  playToggle: document.getElementById('play-toggle'),
  speedUp: document.getElementById('speed-up'),
  speedDown: document.getElementById('speed-down'),
};

const cityCanvas = document.getElementById('city-canvas');
const cityCtx = cityCanvas.getContext('2d');

const state = {
  frames: [],
  totalTicks: 0,
  scenario: '',
  playback: {
    index: 0,
    playing: true,
    speed: 1,
    baseInterval: 600,
    accumulator: 0,
    lastTimestamp: 0,
  },
  lastLoggedTick: 0,
  cars: [],
  houses: [],
  skyline: [],
};

let eventSource;

init();

function init() {
  setupPlaybackControls();
  setupCityLayout();
  fetchInitialFrameData();
  startEventStream();
  requestAnimationFrame(playbackLoop);
  window.addEventListener('resize', resizeCanvas);
  resizeCanvas();
}

function setupPlaybackControls() {
  selectors.playToggle.addEventListener('click', () => {
    state.playback.playing = !state.playback.playing;
    selectors.playToggle.textContent = state.playback.playing ? 'Pause' : 'Play';
    setStatus(state.playback.playing ? 'streaming' : 'paused');
  });
  selectors.speedUp.addEventListener('click', () => adjustSpeed(1));
  selectors.speedDown.addEventListener('click', () => adjustSpeed(-1));
  selectors.timelineSlider.addEventListener('input', (event) => {
    state.playback.index = Number(event.target.value);
    state.playback.playing = false;
    selectors.playToggle.textContent = 'Play';
    renderFrame(state.frames[state.playback.index]);
  });
}

function adjustSpeed(delta) {
  const allowed = [0.25, 0.5, 1, 2, 4, 8];
  const current = state.playback.speed;
  let idx = allowed.indexOf(current);
  if (idx === -1) idx = allowed.indexOf(1);
  idx = Math.min(allowed.length - 1, Math.max(0, idx + delta));
  state.playback.speed = allowed[idx];
  selectors.speedLabel.textContent = `${allowed[idx]}x`;
}

async function fetchInitialFrameData() {
  try {
    const [stateResp, framesResp] = await Promise.all([
      fetch('/api/state'),
      fetch('/api/frames'),
    ]);
    if (stateResp.ok) {
      const info = await stateResp.json();
      selectors.scenario.textContent = info.scenario || 'Unknown world';
      state.totalTicks = info.total_ticks || 0;
      selectors.timelineSlider.max = info.total_ticks || 0;
      selectors.timelineLabel.textContent = `Tick 0 / ${info.total_ticks || 0}`;
      updateStatus(info.completed ? 'complete' : 'warming');
    }
    if (framesResp.ok) {
      const payload = await framesResp.json();
      state.frames = payload.frames || [];
      state.totalTicks = payload.total_ticks || state.totalTicks;
      selectors.timelineSlider.max = state.totalTicks;
      if (state.frames.length > 0) {
        renderFrame(state.frames[state.frames.length - 1]);
      }
    }
  } catch (err) {
    console.error(err);
    setTimeout(fetchInitialFrameData, 3000);
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
      state.frames.push(frame);
      if (state.frames.length > state.totalTicks) {
        selectors.timelineSlider.max = state.frames.length;
      }
      if (state.playback.playing) {
        state.playback.index = state.frames.length - 1;
      }
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

function playbackLoop(timestamp) {
  if (!state.playback.lastTimestamp) {
    state.playback.lastTimestamp = timestamp;
  }
  const delta = timestamp - state.playback.lastTimestamp;
  state.playback.lastTimestamp = timestamp;
  animateCity(delta);
  if (state.playback.playing && state.frames.length) {
    state.playback.accumulator += delta * state.playback.speed;
    while (state.playback.accumulator >= state.playback.baseInterval) {
      advanceFrame();
      state.playback.accumulator -= state.playback.baseInterval;
    }
  }
  requestAnimationFrame(playbackLoop);
}

function advanceFrame() {
  if (!state.frames.length) return;
  const nextIndex = Math.min(state.frames.length - 1, state.playback.index + 1);
  state.playback.index = nextIndex;
  selectors.timelineSlider.value = String(nextIndex);
  renderFrame(state.frames[nextIndex]);
}

function renderFrame(frame) {
  if (!frame) return;
  const snap = frame.snapshot;
  selectors.tick.textContent = formatNumber(snap.tick);
  selectors.days.textContent = snap.days_elapsed.toFixed(1);
  selectors.population.textContent = formatNumber(snap.total_population);
  selectors.starving.textContent = snap.starving_regions.length;
  selectors.timelineLabel.textContent = `Tick ${snap.tick} / ${state.totalTicks || snap.tick}`;
  selectors.timelineSlider.value = String(state.playback.index);
  selectors.statusLabel.textContent = frame.completed ? 'Complete' : selectors.statusLabel.textContent;

  const aggregates = aggregateMetrics(snap);
  selectors.employment.textContent = formatPercent(aggregates.employment);
  selectors.reliability.textContent = formatPercent(aggregates.reliability);
  selectors.stress.textContent = formatPercent(aggregates.stress);

  drawCityScene(snap, aggregates);
  updateRegions(snap.regions);
  processLogs(snap);
}

function aggregateMetrics(snapshot) {
  const totals = snapshot.regions.reduce(
    (acc, region) => {
      acc.employment += 1 - region.unemployment_rate;
      acc.reliability += region.infrastructure_reliability;
      acc.stress += region.credit_stress;
      acc.food += region.food_shortage_ratio;
      acc.energy += region.energy_shortage_ratio;
      return acc;
    },
    { employment: 0, reliability: 0, stress: 0, food: 0, energy: 0 }
  );
  const count = Math.max(snapshot.regions.length, 1);
  return {
    employment: totals.employment / count,
    reliability: totals.reliability / count,
    stress: totals.stress / count,
    shortage: (totals.food + totals.energy) / (2 * count),
  };
}

function updateRegions(regions) {
  if (!regions) return;
  const sorted = [...regions].sort((a, b) => a.id - b.id);
  selectors.regionsGrid.innerHTML = '';
  sorted.forEach((region) => {
    const card = document.createElement('div');
    card.className = 'region-card';
    const header = document.createElement('div');
    header.className = 'region-header';
    const title = document.createElement('div');
    title.innerHTML = `<strong>${region.name}</strong><br><span class="muted">${formatShortNumber(
      region.citizens
    )} citizens</span>`;
    const chip = document.createElement('div');
    chip.className = 'region-chip';
    const alerting = region.food_shortage_ratio > 0.1 || region.energy_shortage_ratio > 0.1;
    chip.dataset.state = alerting ? 'alert' : 'stable';
    chip.textContent = alerting ? 'Shortage' : 'Stable';
    header.append(title, chip);

    const stats = document.createElement('div');
    stats.className = 'region-stats';
    stats.append(
      createStat('Employment', formatPercent(1 - region.unemployment_rate)),
      createStat('Wage', formatCurrency(region.wage)),
      createStat('Power Cap', `${formatShortNumber(region.power_capacity)} MW`),
      createStat('Budget', formatCurrency(region.household_budget))
    );
    card.append(header, stats);
    selectors.regionsGrid.appendChild(card);
  });
}

function createStat(label, value) {
  const root = document.createElement('div');
  root.className = 'stat-block';
  const labelEl = document.createElement('div');
  labelEl.className = 'label';
  labelEl.textContent = label;
  const valueEl = document.createElement('div');
  valueEl.className = 'value';
  valueEl.textContent = value;
  root.append(labelEl, valueEl);
  return root;
}

function processLogs(snapshot) {
  if (snapshot.tick <= state.lastLoggedTick) return;
  const logs = [];
  if (snapshot.starving_regions.length) {
    logs.push({
      type: 'alert',
      text: `Food stress: ${snapshot.starving_regions.join(', ')}`,
    });
  }
  const hotRegion = [...snapshot.regions].sort((a, b) => b.credit_stress - a.credit_stress)[0];
  if (hotRegion && hotRegion.credit_stress > 0.25) {
    logs.push({
      type: 'alert',
      text: `${hotRegion.name} bank stress ${formatPercent(hotRegion.credit_stress)}`,
    });
  }
  const infra = [...snapshot.regions].sort(
    (a, b) => a.infrastructure_reliability - b.infrastructure_reliability
  )[0];
  if (infra) {
    logs.push({
      type: infra.infrastructure_reliability > 0.9 ? 'success' : 'info',
      text: `${infra.name} infra reliability ${formatPercent(
        infra.infrastructure_reliability
      )}`,
    });
  }
  if (!logs.length) {
    logs.push({ type: 'info', text: 'All subsystems nominal.' });
  }
  logs.forEach((entry) => appendLog(entry.text, entry.type));
  state.lastLoggedTick = snapshot.tick;
}

function appendLog(text, type = 'info') {
  const placeholder = selectors.logTerminal.querySelector('.placeholder');
  if (placeholder) {
    placeholder.remove();
  }
  const line = document.createElement('div');
  line.className = `log-line ${type}`;
  line.innerHTML = '<span class="content"></span><span class="cursor">▋</span>';
  selectors.logTerminal.appendChild(line);
  while (selectors.logTerminal.children.length > 8) {
    selectors.logTerminal.removeChild(selectors.logTerminal.firstChild);
  }
  typeText(line.querySelector('.content'), text, line.querySelector('.cursor'));
}

function typeText(target, text, cursor, index = 0) {
  if (!target) return;
  if (index <= text.length) {
    target.textContent = text.slice(0, index);
    setTimeout(() => typeText(target, text, cursor, index + 1), 12 + Math.random() * 30);
  } else if (cursor) {
    cursor.remove();
    target.textContent = text;
  }
}

function setupCityLayout() {
  const width = cityCanvas.width;
  const height = cityCanvas.height;
  for (let row = 0; row < 5; row += 1) {
    for (let col = 0; col < 10; col += 1) {
      state.houses.push({
        x: 80 + col * 80 + Math.random() * 12,
        y: 80 + row * 70 + Math.random() * 8,
        size: 18 + Math.random() * 10,
        phase: Math.random() * Math.PI * 2,
      });
    }
  }
  for (let i = 0; i < 12; i += 1) {
    state.cars.push({
      t: Math.random(),
      speed: 0.05 + Math.random() * 0.08,
      color: Math.random() > 0.5 ? '#58ffe0' : '#ff8dd6',
      lane: i % 3,
    });
  }
  for (let i = 0; i < 4; i += 1) {
    state.skyline.push({
      x: 120 + i * 180,
      width: 100 + Math.random() * 80,
      height: 120 + Math.random() * 80,
    });
  }
}

function resizeCanvas() {
  // Canvas scales via CSS for responsiveness; drawing uses normalized coordinates.
}

function drawCityScene(snapshot, aggregates) {
  const ctx = cityCtx;
  const width = cityCanvas.width;
  const height = cityCanvas.height;
  ctx.clearRect(0, 0, width, height);
  ctx.fillStyle = '#010101';
  ctx.fillRect(0, 0, width, height);
  drawWater(ctx, width, height);
  drawRoads(ctx, width, height);
  drawSkyline(ctx, aggregates);
  drawHouses(ctx, aggregates);
  drawCars(ctx, width, height, aggregates);
  drawIslandDetails(ctx, snapshot);
}

function drawWater(ctx, width, height) {
  const gradient = ctx.createLinearGradient(0, height * 0.6, 0, height);
  gradient.addColorStop(0, 'rgba(35, 80, 120, 0.7)');
  gradient.addColorStop(1, 'rgba(10, 30, 50, 0.8)');
  ctx.fillStyle = gradient;
  ctx.fillRect(0, height * 0.65, width, height * 0.35);
}

function drawRoads(ctx, width, height) {
  ctx.strokeStyle = 'rgba(255,255,255,0.08)';
  ctx.lineWidth = 6;
  ctx.setLineDash([14, 16]);
  for (let i = 0; i < 4; i += 1) {
    const y = 120 + i * 90;
    ctx.beginPath();
    ctx.moveTo(60, y);
    ctx.lineTo(width - 60, y);
    ctx.stroke();
  }
  ctx.setLineDash([]);
}

function drawSkyline(ctx, aggregates) {
  ctx.save();
  ctx.globalAlpha = 0.7;
  state.skyline.forEach((tower) => {
    ctx.fillStyle = 'rgba(255,255,255,0.08)';
    ctx.fillRect(tower.x, 320 - tower.height, tower.width, tower.height);
    const windows = Math.floor(tower.width / 14);
    for (let i = 0; i < windows; i += 1) {
      ctx.fillStyle = Math.random() > aggregates.shortage ? '#ffe066' : '#424242';
      ctx.fillRect(tower.x + i * 14 + 5, 320 - tower.height + 10, 6, tower.height - 20);
    }
  });
  ctx.restore();
}

function drawHouses(ctx, aggregates) {
  state.houses.forEach((house, index) => {
    const pulse = 0.4 + 0.4 * Math.sin(house.phase + Date.now() * 0.001 + index);
    const shortageFactor = 1 - aggregates.shortage;
    ctx.fillStyle = `rgba(88, 255, 224, ${0.35 + pulse * shortageFactor})`;
    ctx.fillRect(house.x, house.y, house.size, house.size * 0.6);
    ctx.fillStyle = `rgba(255, 255, 255, ${0.4 + pulse * 0.3})`;
    ctx.fillRect(house.x + house.size * 0.3, house.y - house.size * 0.4, house.size * 0.4, house.size * 0.4);
  });
}

function animateCity(delta) {
  state.cars.forEach((car) => {
    car.t = (car.t + (delta / 1000) * car.speed) % 1;
  });
}

function drawCars(ctx, width, height, aggregates) {
  ctx.save();
  ctx.shadowColor = '#28f7ff';
  ctx.shadowBlur = 12;
  state.cars.forEach((car) => {
    const pathLength = width - 140;
    const x = 70 + pathLength * car.t;
    const laneOffset = car.lane * 8;
    const y = 120 + laneOffset + Math.sin(car.t * Math.PI * 2) * 4;
    ctx.fillStyle = car.color;
    ctx.beginPath();
    ctx.arc(x, y, 4 + aggregates.employment * 2, 0, Math.PI * 2);
    ctx.fill();
  });
  ctx.restore();
}

function drawIslandDetails(ctx, snapshot) {
  ctx.save();
  ctx.font = '12px "IBM Plex Mono", monospace';
  ctx.fillStyle = 'rgba(255,255,255,0.5)';
  ctx.fillText(`Tick ${snapshot.tick}`, 20, 26);
  ctx.fillText(`Regions ${snapshot.regions.length}`, 20, 44);
  ctx.restore();
}

function updateStatus(stateKey) {
  const map = {
    warming: 'Preparing Simulation',
    streaming: 'Streaming Live',
    paused: 'Paused',
    reconnecting: 'Reconnecting…',
    complete: 'Simulation Complete',
  };
  selectors.status.textContent = map[stateKey] || 'Observing';
  selectors.status.dataset.state = stateKey;
  selectors.statusLabel.textContent = map[stateKey] || 'Observing';
}

function setStatus(key) {
  updateStatus(key);
}

function formatNumber(value) {
  return (value || 0).toLocaleString('en-US');
}

function formatShortNumber(value) {
  if (value >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(1)}B`;
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return (value || 0).toString();
}

function formatPercent(value) {
  return `${(Math.min(1, Math.max(0, value || 0)) * 100).toFixed(1)}%`;
}

function formatCurrency(value) {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    maximumFractionDigits: 0,
  }).format(value || 0);
}
