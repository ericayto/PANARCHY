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
const ctx = cityCanvas.getContext('2d');

const CANVAS_WIDTH = 960;
const CANVAS_HEIGHT = 540;
const MINUTES_PER_DAY = 1440;
const MINUTE_STEP_MS = 40;
const SPEED_MULTIPLIERS = [0.25, 0.5, 1, 2, 4, 8];

const ROAD_PATH = [
  { x: 70, y: 360 },
  { x: 200, y: 360 },
  { x: 420, y: 330 },
  { x: 620, y: 330 },
  { x: 840, y: 360 },
];

const WALKWAY_CURVES = [
  { start: { x: 120, y: 420 }, mid: { x: 360, y: 260 }, end: { x: 620, y: 220 } },
  { start: { x: 200, y: 460 }, mid: { x: 420, y: 320 }, end: { x: 700, y: 260 } },
];

const HOME_ZONES = [
  { x1: 60, x2: 280, y1: 340, y2: 520 },
  { x1: 260, x2: 420, y1: 360, y2: 520 },
];

const WORK_ZONES = [
  { x1: 520, x2: 820, y1: 140, y2: 320 },
  { x1: 500, x2: 780, y1: 320, y2: 460 },
];

const CIVIC_ZONE = { x1: 420, x2: 540, y1: 220, y2: 320 };

const state = {
  frames: [],
  totalTicks: 0,
  playback: {
    minute: 0,
    playing: true,
    speedIndex: 2,
    accumulator: 0,
    lastTimestamp: 0,
  },
  regionVisuals: new Map(),
  dayBlueprints: new Map(),
  lastRenderedFrameIndex: null,
  logTick: 0,
};

let eventSource;

init().catch((err) => console.error(err));

async function init() {
  setupCanvas();
  setupControls();
  await bootstrap();
  startEventStream();
  requestAnimationFrame(playbackLoop);
}

function setupCanvas() {
  cityCanvas.width = CANVAS_WIDTH;
  cityCanvas.height = CANVAS_HEIGHT;
  cityCanvas.style.width = '100%';
  cityCanvas.style.height = 'auto';
  window.addEventListener('resize', () => {
    cityCanvas.style.width = '100%';
  });
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
    setMinute(Number(event.target.value));
    state.playback.playing = false;
    selectors.playToggle.textContent = 'Play';
  });
}

function adjustSpeed(delta) {
  const next = clampIndex(state.playback.speedIndex + delta, SPEED_MULTIPLIERS.length);
  state.playback.speedIndex = next;
  selectors.speedLabel.textContent = `${SPEED_MULTIPLIERS[next]}x`;
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
      const frames = payload.frames || [];
      frames.forEach((frame, idx) => {
        state.frames[idx] = frame;
        cacheFrame(idx, frame);
      });
      if (state.frames.length > 0) {
        state.playback.minute = Math.max(0, state.frames.length * MINUTES_PER_DAY - MINUTES_PER_DAY);
        renderCurrentMinute();
      }
      updateTimelineBounds();
    }
  } catch (err) {
    console.error('bootstrap failed', err);
  }
}

function startEventStream() {
  if (eventSource) eventSource.close();
  eventSource = new EventSource('/api/events');
  eventSource.onmessage = (event) => {
    try {
      const frame = JSON.parse(event.data);
      const index = state.frames.push(frame) - 1;
      cacheFrame(index, frame);
      updateTimelineBounds();
      if (state.playback.playing) {
        setMinute(index * MINUTES_PER_DAY);
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

function cacheFrame(index, frame) {
  if (!frame || !frame.snapshot) return;
  state.dayBlueprints.set(index, buildDayBlueprint(frame.snapshot));
  updateRegionTargets(frame.snapshot);
}

function updateTimelineBounds() {
  const totalMinutes = Math.max(1, state.frames.length * MINUTES_PER_DAY);
  selectors.timelineSlider.max = totalMinutes - 1;
  selectors.timelineSlider.value = String(clampMinute(state.playback.minute));
}

function setMinute(nextMinute) {
  if (!state.frames.length) return;
  const clamped = clampMinute(nextMinute);
  if (clamped === state.playback.minute && state.lastRenderedFrameIndex !== null) {
    return;
  }
  state.playback.minute = clamped;
  selectors.timelineSlider.value = String(clamped);
  renderCurrentMinute();
}

function renderCurrentMinute() {
  const pointer = pointerForMinute(state.playback.minute);
  if (!pointer) return;
  const { frameIndex, minuteOfDay, snapshot } = pointer;
  if (!state.dayBlueprints.has(frameIndex)) {
    state.dayBlueprints.set(frameIndex, buildDayBlueprint(snapshot));
  }
  const blueprint = state.dayBlueprints.get(frameIndex);
  updateHud(snapshot, minuteOfDay);
  if (state.lastRenderedFrameIndex !== frameIndex) {
    updateRegionCards(snapshot.regions);
    processLogs(snapshot);
    state.lastRenderedFrameIndex = frameIndex;
  }
  drawCityScene(blueprint, minuteOfDay);
}

function updateHud(snapshot, minuteOfDay) {
  selectors.tick.textContent = formatNumber(snapshot.tick);
  selectors.days.textContent = snapshot.days_elapsed.toFixed(1);
  selectors.starving.textContent = snapshot.starving_regions.length;
  selectors.population.textContent = formatNumber(snapshot.total_population);
  const timeLabel = formatClock(minuteOfDay);
  selectors.timelineLabel.textContent = `Tick ${snapshot.tick} · ${timeLabel}`;
}

function updateRegionTargets(snapshot) {
  const regions = [...snapshot.regions].sort((a, b) => a.id - b.id);
  const maxPop = Math.max(...regions.map((r) => r.citizens || 1), 1);
  regions.forEach((region, index) => {
    if (!state.regionVisuals.has(region.id)) {
      state.regionVisuals.set(region.id, {
        slotIndex: index,
        height: 40,
        targetHeight: 40,
        glow: 0.4,
        targetGlow: 0.4,
        shortage: 0,
        targetShortage: 0,
        stress: 0,
        targetStress: 0,
      });
    }
    const visual = state.regionVisuals.get(region.id);
    visual.slotIndex = index;
    visual.targetHeight = 28 + (region.citizens / maxPop) * 110;
    visual.targetGlow = 0.25 + (region.infrastructure_reliability || 0) * 0.7;
    visual.targetShortage = Math.max(region.food_shortage_ratio, region.energy_shortage_ratio);
    visual.targetStress = region.credit_stress;
  });
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
    title.innerHTML = `<strong>${region.name}</strong><br><span class="muted">${formatShortNumber(
      region.citizens
    )} citizens</span>`;
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
  logs.forEach((entry) => appendLog(entry.text, entry.type));
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

function buildDayBlueprint(snapshot) {
  const totalPopulation = snapshot.total_population || 1;
  const agentCount = clamp(Math.round(totalPopulation / 1500), 60, 220);
  const agents = [];
  snapshot.regions.forEach((region, regionIndex) => {
    const share = region.citizens / totalPopulation;
    const count = Math.max(4, Math.round(agentCount * share));
    for (let i = 0; i < count; i += 1) {
      agents.push(createAgent(region, regionIndex, i));
    }
  });
  return { snapshot, agents };
}

function createAgent(region, regionIndex, offset) {
  const homeZone = HOME_ZONES[(regionIndex + offset) % HOME_ZONES.length];
  const workZone = WORK_ZONES[(regionIndex + offset) % WORK_ZONES.length];
  const commuteMode = Math.random() < 0.55 + Math.min(0.25, (region.transport_capacity || 0) / Math.max(1, region.transport_capacity || 1)) ? 'car' : 'walk';
  const departBase = 360 + randomBetween(-30, 30) - (region.transport_shortfall || 0) * 180;
  const returnBase = 1050 + randomBetween(-40, 40) + (region.transport_shortfall || 0) * 120;
  const commuteDuration = commuteMode === 'car'
    ? 25 + Math.random() * 20 + (region.transport_shortfall || 0) * 60
    : 45 + Math.random() * 35 + (region.transport_shortfall || 0) * 80;
  return {
    home: randomPoint(homeZone),
    work: randomPoint(workZone),
    commuteMode,
    departMinute: clampMinuteValue(departBase),
    returnMinute: clampMinuteValue(returnBase),
    commuteDuration: Math.max(20, commuteDuration),
    laneOffset: (regionIndex % 3) - 1,
    color: commuteMode === 'car' ? '#6df5f9' : '#ffd86b',
  };
}

function drawCityScene(blueprint, minuteOfDay) {
  const snapshot = blueprint.snapshot;
  drawBackdrop();
  drawWater();
  drawLandMass();
  drawRoadNetwork();
  drawSidewalks();
  animateRegionTowers(snapshot);
  drawAgents(blueprint.agents, minuteOfDay);
}

function drawBackdrop() {
  const gradient = ctx.createLinearGradient(0, 0, 0, CANVAS_HEIGHT);
  gradient.addColorStop(0, '#04050a');
  gradient.addColorStop(0.5, '#05070e');
  gradient.addColorStop(1, '#061329');
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);
}

function drawWater() {
  const gradient = ctx.createLinearGradient(0, CANVAS_HEIGHT * 0.6, 0, CANVAS_HEIGHT);
  gradient.addColorStop(0, 'rgba(16, 60, 120, 0.7)');
  gradient.addColorStop(1, 'rgba(7, 22, 48, 0.85)');
  ctx.fillStyle = gradient;
  ctx.fillRect(0, CANVAS_HEIGHT * 0.55, CANVAS_WIDTH, CANVAS_HEIGHT * 0.45);
}

function drawLandMass() {
  ctx.save();
  ctx.beginPath();
  ctx.moveTo(60, CANVAS_HEIGHT * 0.65);
  ctx.bezierCurveTo(180, 280, 420, 260, 740, 300);
  ctx.bezierCurveTo(880, 360, 820, 430, 600, 470);
  ctx.bezierCurveTo(360, 500, 180, 470, 60, CANVAS_HEIGHT * 0.65);
  ctx.closePath();
  ctx.fillStyle = 'rgba(20, 64, 54, 0.95)';
  ctx.fill();
  ctx.restore();
}

function drawRoadNetwork() {
  ctx.save();
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';
  ctx.strokeStyle = 'rgba(30, 34, 42, 0.95)';
  ctx.lineWidth = 28;
  ctx.beginPath();
  ctx.moveTo(ROAD_PATH[0].x, ROAD_PATH[0].y);
  for (let i = 1; i < ROAD_PATH.length; i += 1) {
    ctx.lineTo(ROAD_PATH[i].x, ROAD_PATH[i].y);
  }
  ctx.stroke();
  ctx.strokeStyle = 'rgba(255, 255, 255, 0.12)';
  ctx.lineWidth = 4;
  ctx.setLineDash([14, 16]);
  ctx.beginPath();
  ctx.moveTo(ROAD_PATH[0].x, ROAD_PATH[0].y);
  for (let i = 1; i < ROAD_PATH.length; i += 1) {
    ctx.lineTo(ROAD_PATH[i].x, ROAD_PATH[i].y);
  }
  ctx.stroke();
  ctx.restore();
}

function drawSidewalks() {
  ctx.save();
  ctx.strokeStyle = 'rgba(255, 255, 255, 0.08)';
  ctx.lineWidth = 6;
  WALKWAY_CURVES.forEach((curve) => {
    ctx.beginPath();
    ctx.moveTo(curve.start.x, curve.start.y);
    ctx.quadraticCurveTo(curve.mid.x, curve.mid.y, curve.end.x, curve.end.y);
    ctx.stroke();
  });
  ctx.restore();
}

function animateRegionTowers(snapshot) {
  state.regionVisuals.forEach((visual, regionId) => {
    const slot = REGION_SLOTS[visual.slotIndex % REGION_SLOTS.length];
    if (!slot) return;
    visual.height = lerp(visual.height, visual.targetHeight, 0.15);
    visual.glow = lerp(visual.glow, visual.targetGlow, 0.1);
    visual.shortage = lerp(visual.shortage, visual.targetShortage, 0.2);
    visual.stress = lerp(visual.stress, visual.targetStress, 0.2);
    ctx.save();
    ctx.translate(slot.x, slot.y);
    const width = 58;
    const height = visual.height;
    ctx.fillStyle = `rgba(99, 255, 227, ${0.3 + visual.glow * 0.4})`;
    ctx.fillRect(-width / 2, -height, width, height);
    ctx.fillStyle = 'rgba(255,255,255,0.07)';
    for (let x = -width / 2 + 8; x < width / 2 - 8; x += 14) {
      ctx.fillRect(x, -height + 10, 6, height - 24);
    }
    if (visual.shortage > 0.2) {
      ctx.fillStyle = `rgba(255, 110, 139, ${visual.shortage * 0.4})`;
      ctx.fillRect(-width / 2, -height, width, height);
    }
    if (visual.stress > 0.25) {
      ctx.strokeStyle = `rgba(255, 214, 107, ${visual.stress * 0.7})`;
      ctx.lineWidth = 2;
      ctx.strokeRect(-width / 2, -height, width, height);
    }
    ctx.restore();
  });
}

function drawAgents(agents, minuteOfDay) {
  ctx.save();
  agents.forEach((agent) => {
    const pose = agentPose(agent, minuteOfDay);
    if (agent.commuteMode === 'car') {
      drawCar(pose);
    } else {
      drawWalker(pose);
    }
  });
  ctx.restore();
}

function agentPose(agent, minuteOfDay) {
  const { departMinute, returnMinute, commuteDuration } = agent;
  const morningEnd = departMinute + commuteDuration;
  const eveningEnd = returnMinute + commuteDuration;
  if (minuteOfDay < departMinute || minuteOfDay >= eveningEnd) {
    return { ...agent.home, status: 'home', mode: agent.commuteMode };
  }
  if (minuteOfDay < morningEnd) {
    const progress = (minuteOfDay - departMinute) / commuteDuration;
    return agent.commuteMode === 'car'
      ? carRoutePosition(progress, agent.laneOffset)
      : walkwayPosition(agent.home, agent.work, progress);
  }
  if (minuteOfDay < returnMinute) {
    return { ...agent.work, status: 'work', mode: agent.commuteMode };
  }
  const progress = (minuteOfDay - returnMinute) / commuteDuration;
  return agent.commuteMode === 'car'
    ? carRoutePosition(1 - progress, agent.laneOffset)
    : walkwayPosition(agent.work, agent.home, progress);
}

function drawCar(pose) {
  const length = 20;
  const width = 9;
  ctx.fillStyle = '#67f6ff';
  ctx.fillRect(pose.x - length / 2, pose.y - width / 2, length, width);
  ctx.fillStyle = 'rgba(255,255,255,0.7)';
  ctx.fillRect(pose.x - length / 2 + 2, pose.y - width / 2 + 1, 4, width - 2);
  ctx.fillRect(pose.x + length / 2 - 6, pose.y - width / 2 + 1, 4, width - 2);
}

function drawWalker(pose) {
  ctx.fillStyle = '#ffd86b';
  ctx.beginPath();
  ctx.arc(pose.x, pose.y, 3.5, 0, Math.PI * 2);
  ctx.fill();
}

function carRoutePosition(progress, laneOffset = 0) {
  const point = pointAlongPolyline(ROAD_PATH, progress);
  const normal = point.normal || { x: 0, y: -1 };
  return {
    x: point.x + normal.x * (laneOffset * 6),
    y: point.y + normal.y * (laneOffset * 6),
  };
}

function walkwayPosition(start, end, progress) {
  const t = clamp(progress, 0, 1);
  const control = {
    x: (start.x + end.x) / 2 + 30 * Math.sin((start.y - end.y) / 120),
    y: (start.y + end.y) / 2 - 60,
  };
  const oneMinusT = 1 - t;
  const x = oneMinusT * oneMinusT * start.x + 2 * oneMinusT * t * control.x + t * t * end.x;
  const y = oneMinusT * oneMinusT * start.y + 2 * oneMinusT * t * control.y + t * t * end.y;
  return { x, y };
}

function playbackLoop(timestamp) {
  if (!state.playback.lastTimestamp) state.playback.lastTimestamp = timestamp;
  const delta = timestamp - state.playback.lastTimestamp;
  state.playback.lastTimestamp = timestamp;
  if (state.playback.playing && state.frames.length) {
    const speed = SPEED_MULTIPLIERS[state.playback.speedIndex];
    state.playback.accumulator += delta * speed;
    const minutesToAdvance = Math.floor(state.playback.accumulator / MINUTE_STEP_MS);
    if (minutesToAdvance > 0) {
      state.playback.accumulator -= minutesToAdvance * MINUTE_STEP_MS;
      setMinute(state.playback.minute + minutesToAdvance);
    }
  }
  requestAnimationFrame(playbackLoop);
}

function pointerForMinute(minute) {
  if (!state.frames.length) return null;
  const totalMinutes = state.frames.length * MINUTES_PER_DAY;
  const clamped = clamp(minute, 0, totalMinutes - 1);
  const frameIndex = Math.min(state.frames.length - 1, Math.floor(clamped / MINUTES_PER_DAY));
  const minuteOfDay = clamped % MINUTES_PER_DAY;
  const frame = state.frames[frameIndex];
  if (!frame || !frame.snapshot) return null;
  return { frameIndex, minuteOfDay, snapshot: frame.snapshot };
}

function pointAlongPolyline(points, progress) {
  const totalLength = polylineLength(points);
  let target = progress * totalLength;
  for (let i = 0; i < points.length - 1; i += 1) {
    const start = points[i];
    const end = points[i + 1];
    const length = distance(start, end);
    if (target <= length) {
      const ratio = length === 0 ? 0 : target / length;
      const x = start.x + (end.x - start.x) * ratio;
      const y = start.y + (end.y - start.y) * ratio;
      const angle = Math.atan2(end.y - start.y, end.x - start.x);
      const normal = { x: -Math.sin(angle), y: Math.cos(angle) };
      return { x, y, normal };
    }
    target -= length;
  }
  const last = points[points.length - 1];
  return { x: last.x, y: last.y, normal: { x: 0, y: -1 } };
}

function polylineLength(points) {
  let length = 0;
  for (let i = 0; i < points.length - 1; i += 1) {
    length += distance(points[i], points[i + 1]);
  }
  return length;
}

function distance(a, b) {
  return Math.hypot(b.x - a.x, b.y - a.y);
}

function randomPoint(rect) {
  return {
    x: rect.x1 + Math.random() * (rect.x2 - rect.x1),
    y: rect.y1 + Math.random() * (rect.y2 - rect.y1),
  };
}

function randomBetween(min, max) {
  return min + Math.random() * (max - min);
}

function clampMinute(value) {
  const totalMinutes = Math.max(1, state.frames.length * MINUTES_PER_DAY);
  return clamp(value, 0, totalMinutes - 1);
}

function clampMinuteValue(value) {
  return Math.max(0, Math.min(MINUTES_PER_DAY - 1, value));
}

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

function clampIndex(index, length) {
  return Math.max(0, Math.min(length - 1, index));
}

function lerp(current, target, factor) {
  return current + (target - current) * factor;
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

function formatClock(minute) {
  const hours = Math.floor(minute / 60);
  const minutes = minute % 60;
  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`;
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
