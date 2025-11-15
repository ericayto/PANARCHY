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

const CANVAS_WIDTH = 960;
const CANVAS_HEIGHT = 540;
const REGION_SLOTS = [
  { x: 150, y: 200 },
  { x: 270, y: 200 },
  { x: 390, y: 200 },
  { x: 510, y: 200 },
  { x: 630, y: 200 },
  { x: 750, y: 200 },
  { x: 150, y: 310 },
  { x: 270, y: 310 },
  { x: 390, y: 310 },
  { x: 510, y: 310 },
  { x: 630, y: 310 },
  { x: 750, y: 310 },
  { x: 210, y: 420 },
  { x: 330, y: 420 },
  { x: 450, y: 420 },
  { x: 570, y: 420 },
  { x: 690, y: 420 },
];

const ROAD_BLUEPRINTS = [
  { points: [ { x: 120, y: 360 }, { x: 840, y: 360 } ], lanes: 3 },
  { points: [ { x: 200, y: 140 }, { x: 200, y: 500 } ], lanes: 2 },
  { points: [ { x: 780, y: 120 }, { x: 780, y: 500 } ], lanes: 2 },
  { points: [ { x: 120, y: 260 }, { x: 860, y: 260 } ], lanes: 2 },
];

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
    baseInterval: 650,
    accumulator: 0,
    lastTimestamp: 0,
  },
  latestFrame: null,
  metricTargets: { employment: 0, reliability: 0, stress: 0, shortage: 0, traffic: 0 },
  metrics: { employment: 0, reliability: 0, stress: 0, shortage: 0, traffic: 0 },
  regionLayout: new Map(),
  regionVisuals: new Map(),
  roads: ROAD_BLUEPRINTS.map(buildRoadGeometry),
  cars: [],
  logTick: 0,
};

let eventSource;

init();

function init() {
  setupControls();
  setupCars();
  bootstrap();
  startEventStream();
  window.addEventListener('resize', resizeCanvas);
  resizeCanvas();
  requestAnimationFrame(playbackLoop);
}

function setupControls() {
  selectors.playToggle.addEventListener('click', () => {
    state.playback.playing = !state.playback.playing;
    selectors.playToggle.textContent = state.playback.playing ? 'Pause' : 'Play';
    updateStatus(state.playback.playing ? 'streaming' : 'paused');
  });
  selectors.speedUp.addEventListener('click', () => adjustSpeed(1));
  selectors.speedDown.addEventListener('click', () => adjustSpeed(-1));
  selectors.timelineSlider.addEventListener('input', (event) => {
    const idx = Number(event.target.value);
    const frame = state.frames[idx];
    if (!frame) return;
    state.playback.index = idx;
    state.playback.playing = false;
    selectors.playToggle.textContent = 'Play';
    ingestFrame(frame);
  });
}

function adjustSpeed(delta) {
  const allowed = [0.25, 0.5, 1, 2, 4, 8];
  let idx = allowed.indexOf(state.playback.speed);
  if (idx === -1) idx = allowed.indexOf(1);
  idx = Math.min(allowed.length - 1, Math.max(0, idx + delta));
  state.playback.speed = allowed[idx];
  selectors.speedLabel.textContent = `${allowed[idx]}x`;
}

async function bootstrap() {
  try {
    const [stateResp, framesResp] = await Promise.all([
      fetch('/api/state'),
      fetch('/api/frames'),
    ]);
    if (stateResp.ok) {
      const info = await stateResp.json();
      state.totalTicks = info.total_ticks || 0;
      selectors.scenario.textContent = info.scenario || 'Unknown world';
      updateStatus(info.completed ? 'complete' : 'warming');
    }
    if (framesResp.ok) {
      const payload = await framesResp.json();
      state.frames = payload.frames || [];
      state.totalTicks = payload.total_ticks || state.totalTicks;
      if (state.frames.length > 0) {
        state.playback.index = state.frames.length - 1;
        ingestFrame(state.frames[state.playback.index]);
      }
      updateTimelineBounds();
    }
  } catch (err) {
    console.error(err);
  }
}

function startEventStream() {
  if (eventSource) eventSource.close();
  eventSource = new EventSource('/api/events');
  eventSource.onmessage = (event) => {
    try {
      const frame = JSON.parse(event.data);
      state.frames.push(frame);
      updateTimelineBounds();
      if (state.playback.playing) {
        state.playback.index = state.frames.length - 1;
        ingestFrame(frame);
      }
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

function updateTimelineBounds() {
  const maxIndex = Math.max(0, state.frames.length - 1);
  selectors.timelineSlider.max = Math.max(state.totalTicks || 0, maxIndex);
  selectors.timelineSlider.value = String(Math.min(state.playback.index, maxIndex));
  if (state.latestFrame) {
    selectors.timelineLabel.textContent = `Tick ${state.latestFrame.snapshot.tick} / ${state.totalTicks || state.latestFrame.snapshot.tick}`;
  }
}

function ingestFrame(frame) {
  if (!frame || !frame.snapshot) return;
  state.latestFrame = frame;
  updateStatus(frame.completed ? 'complete' : 'streaming');
  const snapshot = frame.snapshot;
  updateHud(snapshot);
  updateRegionCards(snapshot.regions);
  updateMetricTargets(snapshot);
  updateRegionTargets(snapshot);
  processLogs(snapshot);
  selectors.timelineLabel.textContent = `Tick ${snapshot.tick} / ${state.totalTicks || snapshot.tick}`;
}

function updateHud(snapshot) {
  selectors.tick.textContent = formatNumber(snapshot.tick);
  selectors.days.textContent = snapshot.days_elapsed.toFixed(1);
  selectors.starving.textContent = snapshot.starving_regions.length;
  selectors.population.textContent = formatNumber(snapshot.total_population);
}

function updateMetricTargets(snapshot) {
  const metrics = aggregateMetrics(snapshot);
  selectors.employment.textContent = formatPercent(metrics.employment);
  selectors.reliability.textContent = formatPercent(metrics.reliability);
  selectors.stress.textContent = formatPercent(metrics.stress);
  state.metricTargets = metrics;
}

function aggregateMetrics(snapshot) {
  const totals = snapshot.regions.reduce(
    (acc, region) => {
      acc.employment += 1 - region.unemployment_rate;
      acc.reliability += region.infrastructure_reliability;
      acc.stress += region.credit_stress;
      acc.shortage += Math.max(region.food_shortage_ratio, region.energy_shortage_ratio);
      acc.traffic += region.transport_utilization || 0;
      return acc;
    },
    { employment: 0, reliability: 0, stress: 0, shortage: 0, traffic: 0 }
  );
  const count = Math.max(snapshot.regions.length, 1);
  return {
    employment: totals.employment / count,
    reliability: totals.reliability / count,
    stress: totals.stress / count,
    shortage: totals.shortage / count,
    traffic: Math.min(1, totals.traffic / count || 0),
  };
}

function updateRegionTargets(snapshot) {
  const regions = [...snapshot.regions].sort((a, b) => a.id - b.id);
  const maxPop = Math.max(...regions.map((r) => r.citizens), 1);
  regions.forEach((region, index) => {
    if (!state.regionLayout.has(region.id)) {
      state.regionLayout.set(region.id, index % REGION_SLOTS.length);
    }
    const slotIndex = state.regionLayout.get(region.id);
    const visual = ensureRegionVisual(region.id, slotIndex);
    const shortage = Math.max(region.food_shortage_ratio, region.energy_shortage_ratio);
    const popRatio = region.citizens / maxPop;
    visual.targetHeight = 26 + popRatio * 95;
    visual.targetShortage = shortage;
    visual.targetStress = region.credit_stress;
    visual.targetGlow = 0.25 + region.infrastructure_reliability * 0.6;
  });
}

function ensureRegionVisual(id, slotIndex) {
  if (!state.regionVisuals.has(id)) {
    state.regionVisuals.set(id, {
      slotIndex,
      height: 30,
      targetHeight: 30,
      shortage: 0,
      targetShortage: 0,
      stress: 0,
      targetStress: 0,
      glow: 0.3,
      targetGlow: 0.3,
    });
  }
  const visual = state.regionVisuals.get(id);
  visual.slotIndex = slotIndex;
  return visual;
}

function updateRegionCards(regions) {
  selectors.regionsGrid.innerHTML = '';
  const sorted = [...regions].sort((a, b) => a.id - b.id);
  sorted.forEach((region) => {
    const card = document.createElement('div');
    card.className = 'region-card';
    const header = document.createElement('div');
    header.className = 'region-header';
    const title = document.createElement('div');
    title.innerHTML = `<strong>${region.name}</strong><br><span class="muted">${formatShortNumber(region.citizens)} citizens</span>`;
    const chip = document.createElement('div');
    chip.className = 'region-chip';
    const alerting = region.food_shortage_ratio > 0.12 || region.energy_shortage_ratio > 0.12;
    chip.dataset.state = alerting ? 'alert' : 'stable';
    chip.textContent = alerting ? 'Shortage' : 'Stable';
    header.append(title, chip);

    const stats = document.createElement('div');
    stats.className = 'region-stats';
    stats.append(
      createStat('Employment', formatPercent(1 - region.unemployment_rate)),
      createStat('Wage', formatCurrency(region.wage)),
      createStat('Transport', formatPercent(1 - (region.transport_shortfall || 0))),
      createStat('Credit Stress', formatPercent(region.credit_stress))
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
  if (snapshot.tick <= state.logTick) return;
  const logs = [];
  if (snapshot.starving_regions.length) {
    logs.push({ type: 'alert', text: `Food stress: ${snapshot.starving_regions.join(', ')}` });
  }
  const stressed = [...snapshot.regions].sort((a, b) => b.credit_stress - a.credit_stress)[0];
  if (stressed && stressed.credit_stress > 0.25) {
    logs.push({ type: 'alert', text: `${stressed.name} bank stress ${formatPercent(stressed.credit_stress)}` });
  }
  const infra = [...snapshot.regions].sort((a, b) => a.infrastructure_reliability - b.infrastructure_reliability)[0];
  if (infra) {
    logs.push({ type: 'info', text: `${infra.name} infra ${formatPercent(infra.infrastructure_reliability)}` });
  }
  if (!logs.length) {
    logs.push({ type: 'info', text: 'All subsystems nominal.' });
  }
  logs.forEach((line) => appendLog(line.text, line.type));
  state.logTick = snapshot.tick;
}

function appendLog(text, type = 'info') {
  const placeholder = selectors.logTerminal.querySelector('.placeholder');
  if (placeholder) placeholder.remove();
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
    setTimeout(() => typeText(target, text, cursor, index + 1), 12 + Math.random() * 24);
  } else if (cursor) {
    cursor.remove();
    target.textContent = text;
  }
}

function setupCars() {
  const totalCars = 28;
  state.cars = Array.from({ length: totalCars }, (_, idx) => ({
    roadIndex: idx % state.roads.length,
    lane: idx % 3,
    length: 26,
    width: 12,
    baseSpeed: 90 + Math.random() * 70,
    color: Math.random() > 0.5 ? '#63ffe3' : '#ff8fa3',
    t: Math.random(),
    active: true,
  }));
}

function playbackLoop(timestamp) {
  if (!state.playback.lastTimestamp) state.playback.lastTimestamp = timestamp;
  const delta = timestamp - state.playback.lastTimestamp;
  state.playback.lastTimestamp = timestamp;

  if (state.playback.playing && state.frames.length > 0) {
    state.playback.accumulator += delta * state.playback.speed;
    const maxIndex = state.frames.length - 1;
    while (state.playback.accumulator >= state.playback.baseInterval) {
      state.playback.accumulator -= state.playback.baseInterval;
      if (state.playback.index < maxIndex) {
        state.playback.index += 1;
        ingestFrame(state.frames[state.playback.index]);
        selectors.timelineSlider.value = String(state.playback.index);
      }
    }
  }

  animateScene(delta);
  requestAnimationFrame(playbackLoop);
}

function animateScene(delta) {
  smoothMetrics(delta);
  smoothRegionVisuals(delta);
  updateCars(delta);
  drawCityScene();
}

function smoothMetrics(delta) {
  const ease = 1 - Math.pow(0.0001, delta / 1000);
  Object.keys(state.metrics).forEach((key) => {
    const current = state.metrics[key] || 0;
    const target = state.metricTargets[key] || 0;
    state.metrics[key] = current + (target - current) * ease;
  });
}

function smoothRegionVisuals(delta) {
  const ease = 1 - Math.pow(0.0001, delta / 1200);
  state.regionVisuals.forEach((visual) => {
    visual.height += (visual.targetHeight - visual.height) * ease;
    visual.shortage += (visual.targetShortage - visual.shortage) * ease;
    visual.stress += (visual.targetStress - visual.stress) * ease;
    visual.glow += (visual.targetGlow - visual.glow) * ease;
  });
}

function updateCars(delta) {
  const activity = clamp(
    0.25 + state.metrics.employment * 0.6 + (1 - state.metrics.shortage) * 0.3,
    0,
    1.4
  );
  const activeCars = Math.max(6, Math.floor(activity * state.cars.length));
  state.cars.forEach((car, idx) => {
    const road = state.roads[car.roadIndex];
    car.active = idx < activeCars;
    if (!road || !car.active) return;
    const speedScale = 0.6 + activity * 0.9;
    car.t = (car.t + (delta / 1000) * car.baseSpeed * speedScale / road.length) % 1;
  });
}

function drawCityScene() {
  const ctx = cityCtx;
  ctx.clearRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);
  drawBackdrop(ctx);
  drawRoads(ctx);
  drawBuildings(ctx);
  drawCars(ctx);
  drawOverlay(ctx);
}

function drawBackdrop(ctx) {
  const gradient = ctx.createLinearGradient(0, 0, 0, CANVAS_HEIGHT);
  gradient.addColorStop(0, '#04050a');
  gradient.addColorStop(0.55, '#05070e');
  gradient.addColorStop(1, '#061329');
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);

  ctx.fillStyle = 'rgba(10, 95, 150, 0.55)';
  ctx.beginPath();
  ctx.moveTo(0, CANVAS_HEIGHT * 0.7);
  ctx.quadraticCurveTo(CANVAS_WIDTH * 0.35, CANVAS_HEIGHT * 0.6, CANVAS_WIDTH, CANVAS_HEIGHT * 0.8);
  ctx.lineTo(CANVAS_WIDTH, CANVAS_HEIGHT);
  ctx.lineTo(0, CANVAS_HEIGHT);
  ctx.closePath();
  ctx.fill();
}

function drawRoads(ctx) {
  state.roads.forEach((road) => {
    ctx.save();
    ctx.strokeStyle = 'rgba(30, 34, 42, 0.95)';
    ctx.lineWidth = 28;
    ctx.lineCap = 'round';
    ctx.beginPath();
    ctx.moveTo(road.points[0].x, road.points[0].y);
    for (let i = 1; i < road.points.length; i += 1) {
      ctx.lineTo(road.points[i].x, road.points[i].y);
    }
    ctx.stroke();

    ctx.strokeStyle = 'rgba(255,255,255,0.12)';
    ctx.lineWidth = 4;
    ctx.setLineDash([14, 16]);
    ctx.beginPath();
    ctx.moveTo(road.points[0].x, road.points[0].y);
    for (let i = 1; i < road.points.length; i += 1) {
      ctx.lineTo(road.points[i].x, road.points[i].y);
    }
    ctx.stroke();
    ctx.restore();
  });
}

function drawBuildings(ctx) {
  state.regionVisuals.forEach((visual) => {
    const slot = REGION_SLOTS[visual.slotIndex % REGION_SLOTS.length];
    if (!slot) return;
    ctx.save();
    ctx.translate(slot.x, slot.y);
    const width = 60;
    const height = visual.height;
    const shortage = visual.shortage || 0;
    const stress = visual.stress || 0;
    const baseColor = shortage > 0.2 ? '#ff6e8b' : '#63ffe3';
    ctx.fillStyle = adjustAlpha(baseColor, 0.55 + visual.glow * 0.35);
    ctx.fillRect(-width / 2, -height, width, height);
    ctx.fillStyle = 'rgba(5, 6, 12, 0.65)';
    ctx.fillRect(-width / 2 + 4, -14, width - 8, 14);
    ctx.fillStyle = 'rgba(255,255,255,0.08)';
    for (let i = -width / 2 + 8; i < width / 2 - 8; i += 14) {
      ctx.fillRect(i, -height + 12, 8, height - 26);
    }
    if (stress > 0.25) {
      ctx.strokeStyle = adjustAlpha('#ffb347', 0.3 + stress * 0.4);
      ctx.lineWidth = 2;
      ctx.strokeRect(-width / 2, -height, width, height);
    }
    ctx.restore();
  });
}

function drawCars(ctx) {
  state.cars.forEach((car) => {
    if (!car.active) return;
    const road = state.roads[car.roadIndex];
    if (!road) return;
    const pos = pointOnRoad(road, car.t);
    const laneOffset = (car.lane - (road.lanes - 1) / 2) * 12;
    const x = pos.x + pos.normal.x * laneOffset;
    const y = pos.y + pos.normal.y * laneOffset;
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(pos.angle);
    ctx.fillStyle = car.color;
    ctx.fillRect(-car.length / 2, -car.width / 2, car.length, car.width);
    ctx.fillStyle = 'rgba(255,255,255,0.7)';
    ctx.fillRect(-car.length / 2 + 4, -car.width / 2 + 2, 6, car.width - 4);
    ctx.fillRect(car.length / 2 - 10, -car.width / 2 + 2, 6, car.width - 4);
    ctx.restore();
  });
}

function drawOverlay(ctx) {
  ctx.save();
  ctx.globalAlpha = 0.25;
  ctx.strokeStyle = 'rgba(255,255,255,0.08)';
  for (let x = 0; x < CANVAS_WIDTH; x += 80) {
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, CANVAS_HEIGHT);
    ctx.stroke();
  }
  for (let y = 0; y < CANVAS_HEIGHT; y += 80) {
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(CANVAS_WIDTH, y);
    ctx.stroke();
  }
  ctx.restore();
}

function buildRoadGeometry(blueprint) {
  const segments = [];
  let length = 0;
  for (let i = 0; i < blueprint.points.length - 1; i += 1) {
    const start = blueprint.points[i];
    const end = blueprint.points[i + 1];
    const segLength = Math.hypot(end.x - start.x, end.y - start.y);
    length += segLength;
    segments.push({ start, end, length: segLength });
  }
  return { ...blueprint, segments, length };
}

function pointOnRoad(road, t) {
  let target = t * road.length;
  for (const segment of road.segments) {
    if (target <= segment.length) {
      const ratio = segment.length === 0 ? 0 : target / segment.length;
      const x = segment.start.x + (segment.end.x - segment.start.x) * ratio;
      const y = segment.start.y + (segment.end.y - segment.start.y) * ratio;
      const angle = Math.atan2(segment.end.y - segment.start.y, segment.end.x - segment.start.x);
      const normal = { x: -Math.sin(angle), y: Math.cos(angle) };
      return { x, y, angle, normal };
    }
    target -= segment.length;
  }
  const last = road.segments[road.segments.length - 1];
  const angle = Math.atan2(last.end.y - last.start.y, last.end.x - last.start.x);
  return {
    x: last.end.x,
    y: last.end.y,
    angle,
    normal: { x: -Math.sin(angle), y: Math.cos(angle) },
  };
}

function resizeCanvas() {
  cityCanvas.width = CANVAS_WIDTH;
  cityCanvas.height = CANVAS_HEIGHT;
  cityCanvas.style.width = '100%';
  cityCanvas.style.height = 'auto';
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
  return `${(clamp(value, 0, 1) * 100).toFixed(1)}%`;
}

function formatCurrency(value) {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    maximumFractionDigits: 0,
  }).format(value || 0);
}

function clamp(value, min = 0, max = 1) {
  return Math.max(min, Math.min(max, value));
}

function adjustAlpha(color, alpha) {
  if (!color) return `rgba(255, 255, 255, ${alpha})`;
  if (color.startsWith('#')) {
    const hex = color.slice(1);
    const normalized = hex.length === 3
      ? hex.split('').map((c) => c + c).join('')
      : hex.padStart(6, '0');
    const value = parseInt(normalized, 16);
    const r = (value >> 16) & 255;
    const g = (value >> 8) & 255;
    const b = value & 255;
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }
  if (color.startsWith('rgb')) {
    return color.replace('rgb', 'rgba').replace(')', `, ${alpha})`);
  }
  return color;
}
