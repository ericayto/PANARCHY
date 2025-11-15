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

const spriteSources = {
  ground: '/sprites/terrain_grass.png',
  soil: '/sprites/terrain_mud.png',
  water: '/sprites/water.png',
  road: '/sprites/road.png',
  residential: '/sprites/residential.png',
  commercial: '/sprites/commercial.png',
  industrial: '/sprites/industrial.png',
  park: '/sprites/park.png',
};

const cityState = {
  tileSize: 48,
  cols: 28,
  rows: 18,
  grid: [],
  sprites: {},
  ready: false,
  cars: [],
  loopLength: 1,
};

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
  lastLoggedTick: 0,
};

let eventSource;

init();

async function init() {
  await preloadSprites();
  configureCanvas();
  setupPlaybackControls();
  await bootstrap();
  startEventStream();
  window.addEventListener('resize', resizeCanvas);
  resizeCanvas();
  requestAnimationFrame(playbackLoop);
}

async function preloadSprites() {
  const entries = Object.entries(spriteSources);
  const resolved = await Promise.all(
    entries.map(([key, src]) =>
      loadImage(src).then((img) => [key, img])
    )
  );
  resolved.forEach(([key, img]) => {
    cityState.sprites[key] = img;
  });
  cityState.ready = true;
}

function loadImage(src) {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.decoding = 'async';
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.src = src;
  });
}

function configureCanvas() {
  cityCanvas.width = cityState.cols * cityState.tileSize;
  cityCanvas.height = cityState.rows * cityState.tileSize;
  cityState.grid = createGrid('ground');
  drawCityScene();
}

function setupPlaybackControls() {
  selectors.playToggle.addEventListener('click', () => {
    state.playback.playing = !state.playback.playing;
    selectors.playToggle.textContent = state.playback.playing ? 'Pause' : 'Play';
    updateStatus(state.playback.playing ? 'streaming' : 'paused');
  });
  selectors.speedUp.addEventListener('click', () => adjustSpeed(1));
  selectors.speedDown.addEventListener('click', () => adjustSpeed(-1));
  selectors.timelineSlider.addEventListener('input', (event) => {
    const nextIndex = Number(event.target.value);
    state.playback.index = nextIndex;
    state.playback.playing = false;
    selectors.playToggle.textContent = 'Play';
    renderFrame(state.frames[nextIndex]);
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
      selectors.scenario.textContent = info.scenario || 'Unknown world';
      state.totalTicks = info.total_ticks || 0;
      selectors.timelineSlider.max = state.totalTicks || 0;
      selectors.timelineLabel.textContent = `Tick 0 / ${state.totalTicks}`;
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
    setTimeout(bootstrap, 3000);
  }
}

function startEventStream() {
  if (eventSource) eventSource.close();
  eventSource = new EventSource('/api/events');
  eventSource.onmessage = (event) => {
    try {
      const frame = JSON.parse(event.data);
      state.frames.push(frame);
      if (state.playback.playing) {
        state.playback.index = state.frames.length - 1;
      }
      selectors.timelineSlider.max = Math.max(state.totalTicks, state.frames.length - 1);
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
  if (!frame || !frame.snapshot) return;
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

  if (cityState.ready) {
    rebuildCityTiles(snap);
    drawCityScene();
  }
  updateRegions(snap.regions);
  processLogs(snap);
}

function aggregateMetrics(snapshot) {
  const totals = snapshot.regions.reduce(
    (acc, region) => {
      acc.employment += 1 - region.unemployment_rate;
      acc.reliability += region.infrastructure_reliability;
      acc.stress += region.credit_stress;
      acc.shortage += region.food_shortage_ratio + region.energy_shortage_ratio;
      return acc;
    },
    { employment: 0, reliability: 0, stress: 0, shortage: 0 }
  );
  const count = Math.max(snapshot.regions.length, 1);
  return {
    employment: totals.employment / count,
    reliability: totals.reliability / count,
    stress: totals.stress / count,
    shortage: totals.shortage / (2 * count),
  };
}

function rebuildCityTiles(snapshot) {
  const grid = createGrid('ground');
  const rows = cityState.rows;
  const cols = cityState.cols;
  // waterway band
  for (let r = 2; r < 5; r += 1) {
    for (let c = 0; c < cols; c += 1) {
      grid[r][c].sprite = 'water';
    }
  }
  // soil diagonal accent
  for (let i = 0; i < Math.min(rows, cols); i += 1) {
    const r = rows - 1 - i;
    const c = i;
    if (grid[r] && grid[r][c]) {
      grid[r][c].sprite = 'soil';
    }
  }
  const midRow = Math.floor(rows / 2);
  const midCol = Math.floor(cols / 2);
  for (let c = 0; c < cols; c += 1) {
    grid[midRow][c].sprite = 'road';
  }
  for (let r = 0; r < rows; r += 1) {
    grid[r][midCol].sprite = 'road';
  }
  const nodes = computeRegionNodes(snapshot.regions.length);
  nodes.forEach((node) => carveCorridor(grid, node.x, node.y, midCol, midRow));
  nodes.forEach((node, idx) => layDistrict(grid, node, snapshot.regions[idx]));
  cityState.grid = grid;
  if (!cityState.cars.length) {
    initCars();
  }
}

function createGrid(fillSprite) {
  return Array.from({ length: cityState.rows }, () =>
    Array.from({ length: cityState.cols }, () => ({ sprite: fillSprite }))
  );
}

function computeRegionNodes(count) {
  if (!count) return [];
  const nodes = [];
  const radius = Math.min(cityState.cols, cityState.rows) * 0.35;
  for (let i = 0; i < count; i += 1) {
    const angle = (i / count) * Math.PI * 2;
    const x = Math.round(cityState.cols / 2 + radius * Math.cos(angle));
    const y = Math.round(cityState.rows / 2 + radius * Math.sin(angle));
    nodes.push({
      x: clamp(Math.min(cityState.cols - 4, x), 3, cityState.cols - 4),
      y: clamp(Math.min(cityState.rows - 4, y), 3, cityState.rows - 4),
    });
  }
  return nodes;
}

function carveCorridor(grid, x, y, midCol, midRow) {
  let cx = x;
  let cy = y;
  while (cx !== midCol) {
    grid[cy][cx].sprite = 'road';
    cx += cx < midCol ? 1 : -1;
  }
  while (cy !== midRow) {
    grid[cy][cx].sprite = 'road';
    cy += cy < midRow ? 1 : -1;
  }
}

function layDistrict(grid, node, region) {
  const palette = districtPalette(region);
  let index = 0;
  for (let dy = -1; dy <= 1; dy += 1) {
    for (let dx = -1; dx <= 1; dx += 1) {
      const sprite = palette[Math.min(palette.length - 1, index)];
      setTile(grid, node.x + dx, node.y + dy, sprite);
      index += 1;
    }
  }
  for (let dx = -2; dx <= 2; dx += 1) {
    setTile(grid, node.x + dx, node.y - 2, 'road');
    setTile(grid, node.x + dx, node.y + 2, 'road');
  }
  for (let dy = -2; dy <= 2; dy += 1) {
    setTile(grid, node.x - 2, node.y + dy, 'road');
    setTile(grid, node.x + 2, node.y + dy, 'road');
  }
}

function districtPalette(region) {
  const palette = ['residential'];
  if (region.household_budget > region.citizens * 1.2) {
    palette.push('commercial');
  }
  if (region.energy_shortage_ratio > 0.12 || region.energy_curtailed > 500) {
    palette.push('industrial');
  }
  if (region.policy_approval > 0.6 || region.budget_balance > 0) {
    palette.push('park');
  }
  while (palette.length < 4) {
    palette.push(palette[palette.length - 1] || 'residential');
  }
  return palette;
}

function setTile(grid, x, y, sprite) {
  if (y < 0 || y >= cityState.rows) return;
  if (x < 0 || x >= cityState.cols) return;
  grid[y][x].sprite = sprite;
}

function drawCityScene() {
  const ctx = cityCtx;
  const width = cityCanvas.width;
  const height = cityCanvas.height;
  const tileSize = cityState.tileSize;
  ctx.clearRect(0, 0, width, height);
  ctx.fillStyle = '#050505';
  ctx.fillRect(0, 0, width, height);
  for (let row = 0; row < cityState.rows; row += 1) {
    for (let col = 0; col < cityState.cols; col += 1) {
      const tile = cityState.grid[row][col];
      const sprite = cityState.sprites[tile.sprite];
      if (sprite) {
        ctx.drawImage(sprite, col * tileSize, row * tileSize, tileSize, tileSize);
      }
    }
  }
  drawCars(ctx);
}

function initCars() {
  const width = cityState.cols * cityState.tileSize;
  const height = cityState.rows * cityState.tileSize;
  cityState.loopLength = 2 * (width + height);
  cityState.cars = Array.from({ length: 24 }, () => ({
    progress: Math.random(),
    speed: 0.02 + Math.random() * 0.05,
  }));
}

function animateCity(delta) {
  if (!cityState.cars.length) return;
  cityState.cars.forEach((car) => {
    car.progress = (car.progress + (delta / 1000) * car.speed) % 1;
  });
}

function drawCars(ctx) {
  if (!cityState.cars.length) return;
  ctx.save();
  ctx.fillStyle = 'rgba(255,255,255,0.9)';
  cityState.cars.forEach((car) => {
    const pos = positionAlongLoop(car.progress);
    ctx.fillRect(pos.x, pos.y, 6, 6);
  });
  ctx.restore();
}

function positionAlongLoop(progress) {
  const width = cityState.cols * cityState.tileSize;
  const height = cityState.rows * cityState.tileSize;
  const perimeter = 2 * (width + height);
  let distance = progress * perimeter;
  if (distance < width) {
    return { x: distance, y: Math.max(4, height / 2 - 8) };
  }
  distance -= width;
  if (distance < height) {
    return { x: width / 2 + 8, y: distance };
  }
  distance -= height;
  if (distance < width) {
    return { x: width - distance, y: height / 2 + 8 };
  }
  distance -= width;
  return { x: width / 2 - 8, y: height - distance };
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
  logs.forEach((entry) => appendLog(entry.text));
  state.lastLoggedTick = snapshot.tick;
}

function appendLog(text) {
  const placeholder = selectors.logTerminal.querySelector('.placeholder');
  if (placeholder) placeholder.remove();
  const line = document.createElement('div');
  line.className = 'log-line';
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
    setTimeout(() => typeText(target, text, cursor, index + 1), 14 + Math.random() * 24);
  } else if (cursor) {
    cursor.remove();
    target.textContent = text;
  }
}

function resizeCanvas() {
  const containerWidth = cityCanvas.parentElement.clientWidth;
  const aspect = (cityState.rows * cityState.tileSize) / (cityState.cols * cityState.tileSize);
  cityCanvas.style.height = `${containerWidth * aspect}px`;
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

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}
