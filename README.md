# PANARCHY – World Simulation Engine (MacBook-Optimized Full Spec)

Below is the consolidated, implementation-ready spec that merges the original design with all MacBook and AI-usage adjustments.

---

## 0. Current Implementation Status (Phase 0)

Phase 0 runs as a Rust crate with a deterministic ECS core, tick scheduler, RNG streams, JSON snapshotting, and the `tiny_island` scenario. The CLI executes the Environment → Population → Bookkeeping loop and writes snapshots to `snapshots/<scenario>/`.

### What was delivered

1. **ECS + Scheduler** – Entities (regions) carry typed population and resource components that run through the ordered system pipeline (`src/systems/*`).
2. **Deterministic RNG** – Each system consumes its own ChaCha8 stream (see `src/rng.rs`) so identical seeds reproduce runs exactly.
3. **Scenario loader** – `scenarios/tiny_island.yaml` defines the 50k-person world, runtime defaults, and resource regeneration rates.
4. **Snapshots** – `snapshots/SCENARIO/tick_XXXXXX.json` captures tick state in a simple Arrow/Parquet-ready JSON schema; the interval is configurable via CLI/YAML.
5. **Tests** – `cargo test` exercises scenario parsing, deterministic ticks, and snapshot persistence.

### Try it locally

```bash
cargo run -- --scenario scenarios/tiny_island.yaml --ticks 90
```

Snapshots land in `snapshots/tiny_island/`. Override the interval with `--snapshot-interval N` (use `0` to disable).

Run the automated checks with:

```bash
cargo test
```

---

## 1. Vision & Principles

**Vision:** Build a modular, AI-augmented world simulator that runs comfortably on a single MacBook while still modeling a rich, plausible world with emergent societies, economies, and technologies. It supports AI agents (local models or API-hosted ones) without sacrificing determinism, performance, or safety.

**Core principles**

1. **Plausible, not prophetic:** Match stylized facts and empirical regularities rather than perfect prediction.
2. **MacBook-first:** All core features must run on a single MacBook (Apple Silicon or Intel) without needing a workstation or cluster.
3. **Modular & composable:** Subsystems (environment, population, economy, politics, etc.) remain separate modules communicating via typed state and events.
4. **Deterministic by default:** Same seed ⇒ same run. AI is optional and can be made deterministic via distilled policies.
5. **Performance-aware:** Level-of-detail (LoD), representative agents, and configurable complexity to scale world size up/down.
6. **Config-driven:** World size, complexity, and AI usage are configured via YAML/JSON. No hard-coded parameters.
7. **Safety & auditability:** Every AI decision is validated and logged. State transitions obey invariants.

---

## 2. High-Level Architecture

### 2.1 Core Stack

* **Engine core:** Rust (preferred) or C++, built around an ECS (Entity-Component-System) pattern.
* **AI & tooling:** Python for optional AI services and training (local models, distillation, RL).
* **UI/Visualization:** Web-based frontend (React/Next.js) or a minimal Rust+egui UI for quick iteration.
* **Data formats:**

  * Columnar storage: Apache Arrow in-memory, Parquet on disk for snapshots.
  * Config & scenarios: YAML/JSON.

### 2.2 Process Layout

* **Default (Laptop Mode):** Single process:

  * Simulation engine
  * AI provider(s) (local models & remote API clients)
  * Embedded HTTP/WebSocket server for UI.
* **Optional (Advanced Mode):** AI providers in separate process via gRPC, feature-gated and disabled by default.

### 2.3 Modules

1. **Engine Core** (Rust)

   * ECS: entities, components, systems.
   * Scheduler: tick loop, system ordering, substeps.
   * RNG: deterministic, with per-system streams.
   * Events bus: typed events and queues.

2. **Subsystems**

   * Environment & Resources
   * Population & Demography
   * Firms, Production & Markets
   * Finance & Banking
   * Energy & Infrastructure
   * Technology & Innovation
   * Culture, Media & Information
   * Politics & Governance
   * Health & Epidemiology
   * Diplomacy, Trade & Conflict

3. **AI Layer**

   * Local AI (Ollama, llama.cpp, MLX, ONNX) via a unified provider interface.
   * Remote AI (OpenAI, Anthropic, etc.) via API keys.
   * Rules-based & heuristic agents for baseline and deterministic mode.
   * Safety & validation.

4. **Persistence & Snapshots**

   * Periodic state snapshots to Parquet.
   * Event log for replay.

5. **UI & Tools**

   * Map visualization with tiles.
   * KPIs & charts.
   * Inspector for entities.
   * Scenario editor (later phase).

---

## 3. Simulation Core

### 3.1 Time & Scheduler

* **Tick:** Default 1 day per tick.
* **Substeps:** Subticks for fast subsystems (e.g., markets, movement) if necessary.
* **System order per tick (baseline):**

  1. Environment & climate indices
  2. Population & demography
  3. Firms, production & inventories
  4. Goods markets & labor markets
  5. Finance & banking
  6. Government & policy
  7. Diplomacy, trade & conflict
  8. Infrastructure & energy
  9. Media & information
  10. Health & epidemiology
  11. Bookkeeping (ledgers, invariants, KPIs)

Pseudocode:

```rust
fn tick(dt_days: f64) {
    rng::advance_tick();
    env_system::update(dt_days);
    pop_system::update(dt_days);
    prod_system::update(dt_days);
    market_system::update(dt_days);
    finance_system::update(dt_days);
    politics_system::update(dt_days);
    diplomacy_system::update(dt_days);
    infra_system::update(dt_days);
    media_system::update(dt_days);
    health_system::update(dt_days);
    bookkeeping::update(dt_days);
    snapshots::maybe_checkpoint();
}
```

### 3.2 ECS & Data Layout

* Entities: simple numeric IDs.
* Components: plain old data (POD) structs, stored column-wise (SoA).
* Systems: functions that read/write specific component sets.
* Use ECS crates (e.g., `hecs`, `legion`, or custom) with careful SoA design.

Example components (Rust):

```rust
struct Location { tile_id: u32 }
struct Age { years: u8 }
struct Wealth { real: f32 }
struct Inventory { good_id: u16, qty: f32 }
```

### 3.3 Spatial Model & LoD

* World represented as a grid of tiles using a quadtree or simple 2D grid:

  * Coarser levels (for MacBook): L=6 or L=7 (vs L=8+ originally).
  * Example: 256 x 128 tiles for "laptop_small" scenario.
* Each tile stores:

  * Environment indices: temperature, precipitation, fertility.
  * Land use/cover: urban, cropland, forest, etc.
  * Resource stocks: mineral deposits, water, biomass.

### 3.4 Determinism

* Use a counter-based RNG (e.g., `rand_chacha` or `philox`) with seeds derived from `(system_id, entity_id, tick)`.
* Strict mode:

  * No LLM calls or only deterministic policies.
  * Fixed order of system execution.
  * Stable floating-point operations where possible.

---

## 4. MacBook Performance Design

### 4.1 Target Presets

Define configurable presets tuned for a laptop:

| Preset   | Persons | Firms | Goods | Tick (days) | Target tick time | RAM budget |
| -------- | ------: | ----: | ----: | ----------: | ---------------: | ---------: |
| Tiny     |     50k |    3k |     6 |           1 |         ≤ 150 ms |     ≤ 4 GB |
| Small    |    150k |    8k |     8 |           1 |         ≤ 400 ms |     ≤ 8 GB |
| Medium   |    300k |   15k |    10 |           1 |          ≤ 1.2 s |    ≤ 16 GB |
| Large(*) |    500k |   25k |    12 |           1 |          1.5–3 s |   24–32 GB |

* Large is a stretch goal usable on higher-end MBPs.

### 4.2 Level of Detail & Aggregation

* Coarse spatial resolution at laptop presets.
* Representative agents:

  * Instead of 10 households per tile, use a single "Household" entity with a `weight` field representing many households.
* Markets:

  * Start with simple posted-price markets and upgrade to double-auction only if needed.
* Finance & conflict:

  * Interbank, FX, and detailed conflict models are toggles; off by default in Tiny.

### 4.3 Memory & Types

* Use `f32` for most state; `f64` for aggregates and sums (e.g., ledgers, KPIs).
* Use compact integers (`u8`, `u16`, `u32`) for indexes and enums.

---

## 5. Subsystems & State Models

Each subsystem includes: entities, components, processes, actions, invariants, and key KPIs.

### 5.1 Environment & Resources

**Entities:**

* `Tile`, `ResourceDeposit`, `Forest`, `WaterBody`.

**Components (examples):**

* Environment: `temp_idx`, `precip_idx`, `soil_fertility`.
* Resources: `resource_type`, `remaining_tonnage`, `ore_grade`.

**Processes:**

* Seasonal cycles as sinusoidal functions over `temp_idx`, `precip_idx`.
* Resource extraction reduces `remaining_tonnage` and quality (`ore_grade`).
* Land use transitions between forest/cropland/urban with friction.

**Actions:**

* `open_mine(tile, resource_type)`
* `harvest_forest(tile, rate)`
* `protect_area(tile)`

**Invariants:**

* No negative resource stocks.

**KPIs:**

* Total extracted vs initial resource stock.
* Land-cover fractions over time.

---

### 5.2 Population & Demography

**Entities:**

* `Person`, `Household`.

**Person component fields (minimal):**

```text
person_id: u64
household_id: u64
age_years: u8
sex: u8
health_score: f32         // 0..1
education_level: u8
skills_vec: [f32; 8]
employment_status: u8
employer_firm_id: u64 (nullable)
wage_real: f32
wealth_real: f32
tile_id: u32
```

**Processes:**

* Aging, births, deaths.
* Education progression by age; skill accumulation through work.
* Migration between tiles based on wage differentials, distance, and policies.

**Actions:**

* `seek_job()`
* `accept_job(firm_id)`
* `migrate(tile_id)`
* `enroll_training(skill)`

**Invariants:**

* No negative populations.

**KPIs:**

* Age pyramid shape.
* Employment rate, migration flows.

---

### 5.3 Firms, Production & Markets

**Entities:**

* `Firm`, `Sector`, `Market`.

**Firm component fields (simplified):**

```text
firm_id: u64
sector_id: u16
tile_id: u32
technology_id: u32
capacity_units: f32
inventories: map<good_id, f32>
posted_price: map<good_id, f32>
num_employees: u32
cash_real: f32
debt_real: f32
wage_offer: f32
```

**Processes:**

* Production based on a Cobb-Douglas or CES function.
* Inventory management via base-stock policy.
* Posted-price model with adaptive markups:

  * If inventory low or demand high → raise price.
  * If inventory high or demand low → decrease price.
* Job matching between persons and firms.

**Actions:**

* `set_price(good_id, price)`
* `post_vacancies(count, wage)`
* `invest(capacity_delta)`
* `choose_supplier(firm_id)`

**Invariants:**

* Material balance: inputs consumed equal outputs + waste + change in inventory.

**KPIs:**

* Distribution of firm sizes.
* Price dispersion within sectors.
* Unemployment and vacancy rates.

---

### 5.4 Finance & Banking

**Entities:**

* `Bank`, `Account`, `Loan`, `CentralBank`.

**Processes:**

* Basic loan issuance and repayment.
* Interest payments.
* Optional central bank policy rate affecting loan rates.

**Actions:**

* `issue_loan(firm_id, amount, rate)`
* `set_policy_rate(rate)`

**Invariants:**

* No negative account balances without explicit overdraft.
* Basic accounting identities on each tick.

**KPIs:**

* Credit-to-GDP ratio.
* Default rates.

---

### 5.5 Energy & Infrastructure

**Entities:**

* `PowerPlant`, `GridNode`, `Road`, `Port`, `Warehouse`.

**Processes:**

* Simple merit-order dispatch: cheapest plants first until demand met.
* Transport: use shortest paths on a road network; congestion approximated via capacity functions.

**Actions:**

* `build_infra(type, tile_id)`
* `maintain_infra(id)`

**Invariants:**

* Demand cannot exceed supply + allowed outages.

**KPIs:**

* Average electricity price.
* Fraction of unmet demand.

---

### 5.6 Technology & Innovation

**Entities:**

* `TechNode`, `ResearchProject`.

**Processes:**

* R&D investment leads to probability of tech progress.
* Learning-by-doing reduces costs based on cumulative output.

**Actions:**

* `start_RnD(tech_id, budget)`
* `adopt_tech(firm_id, tech_id)`

**Invariants:**

* Tech dependencies must be satisfied before adoption.

**KPIs:**

* Tech diffusion curves (S-shaped).

---

### 5.7 Culture, Media & Information (Lite)

**Entities:**

* `MediaOutlet`, `OpinionState` (per tile or group).

**Processes:**

* Opinion update via weighted averaging of media and neighbors.

**Actions:**

* `broadcast(message)`
* `run_campaign(target_group)`

**KPIs:**

* Polarization metrics, media influence on voting outcomes.

---

### 5.8 Politics & Governance

**Entities:**

* `Government`, `Party`, `PolicySet`.

**Government state (simplified):**

```text
budget_revenue: f64
budget_spending: f64
public_debt: f64
approval_rating: f32
tax_rate: f32
transfer_level: f32
policy_flags: ...
```

**Processes:**

* Revenue from taxes.
* Spending on public services.
* Elections based on party preferences and approval.

**Actions:**

* `set_tax_rate(rate)`
* `set_transfers(level)`
* `invest_public(amount)`

**Invariants:**

* Budget identity: revenue – spending – interest ≈ deficit.

**KPIs:**

* GDP growth, unemployment, inequality, debt/GDP.

---

### 5.9 Health & Epidemiology (Optional)

**Entities:**

* `Disease`, `HealthFacility`.

**Processes:**

* SEIR (Susceptible, Exposed, Infectious, Recovered) dynamics per tile.

**Actions:**

* `deploy_NPI(policy)`
* `vaccinate(group)`

**KPIs:**

* Peak infection rates, deaths, hospital overload.

---

### 5.10 Diplomacy, Trade & Conflict (Optional)

**Entities:**

* `Polity`, `TradeAgreement`, `ConflictEvent`.

**Processes:**

* Trade using gravity-like rules.
* Simple conflict model (onset, escalation, resolution).

**Actions:**

* `set_tariff(partner, good, rate)`
* `impose_sanction(partner)`

**KPIs:**

* Trade volumes, welfare changes, conflict frequency.

---

## 6. AI Integration (Local & API)

### 6.1 Decision API

AI is used to control certain agents (governments, large firms, etc.). The engine exposes a decision API internally (Rust) and optionally via gRPC.

**Observation (concept):**

```text
actor_id
actor_type
state_slice (aggregated KPIs, not raw entities)
action_space (allowed verbs + JSON schemas)
constraints (hard bounds, budgets, invariants)
```

**ActionProposal (concept):**

```text
verb
args_json
confidence
rationale_summary  // short human-readable summary
```

### 6.2 AI Providers

On MacBook, AI providers are pluggable.

**Provider types:**

* `Rules`: hand-coded heuristics, fastest and fully deterministic.
* `Local`: local LLMs (Ollama, llama.cpp, MLX), or small neural policies via ONNX.
* `Remote`: hosted models (OpenAI, Anthropic, etc.) using API keys.

**Rust trait (simplified):**

```rust
pub trait AIProvider: Send + Sync {
    fn propose(&self, obs: &Observation) -> Result<ActionProposal, AIError>;
}

pub struct ProviderRouter {
    gov: Box<dyn AIProvider>,
    firm: Box<dyn AIProvider>,
    bank: Box<dyn AIProvider>,
}
```

### 6.3 Local AI

**Options:**

* **Ollama:** easiest; run quantized models locally via HTTP.
* **llama.cpp:** efficient C/C++ backend; can be wrapped for Rust.
* **MLX (Apple):** Python library optimized for Apple Silicon.
* **ONNX Runtime:** for deterministic small policies.

**Usage pattern:**

* For development: choose a small quantized model (e.g., 7B q4_0) and limit calls.
* For performance and determinism: distill policies to ONNX models and call them locally.

### 6.4 Remote AI

* Use environment variables for API keys (`AI_OPENAI_KEY`, etc.).
* Rate limiting and budgets per tick:

  * `max_calls_per_tick`
  * `max_tokens_per_decision`
  * `max_cost_per_run` (if tracked)

### 6.5 Observations & Summaries

* Observations are compact, aggregated views:

  * Rolling averages, deltas, top-K lists.
  * No raw individual data to keep token usage small and preserve performance.

Example compressed observation for a government:

```json
{
  "gdp_growth": 0.02,
  "inflation": 0.03,
  "unemployment": 0.07,
  "debt_to_gdp": 0.60,
  "inequality_gini": 0.34,
  "tax_rate": 0.25,
  "transfer_level": 0.10,
  "options": [
    {"verb": "set_tax_rate", "min": 0.15, "max": 0.40},
    {"verb": "set_transfers", "min": 0.05, "max": 0.20}
  ]
}
```

### 6.6 Safety & Validation

All AI-proposed actions pass through a safety gate:

1. **Schema validation:** ensure `verb` and `args_json` match the allowed schema.
2. **Pre-state invariants:** check constraints like max tax rate change per period, deficit bounds.
3. **Dry-run simulation:** fast local simulation of the action on a copied state slice (optional for laptop-mode, but recommended).
4. **Rate limiting:** enforce per-tick change bounds.

Failed actions fall back to rules-based decisions.

---

## 7. Configuration: MacBook-Optimized Scenarios

### 7.1 `laptop_small.yaml`

```yaml
name: "laptop_small"
random_seed: 42

snapshot:
  every_ticks: 30
  compression: "zstd"

spatial:
  level: 7
  width_tiles: 256
  height_tiles: 128

population:
  persons: 150_000
  use_representative_households: true

population_params:
  fertility_base: 1.8
  mortality_base: 0.01


economy:
  sectors: ["agri", "manufacturing", "services", "energy"]
  goods: ["wheat", "steel", "energy", "goods", "services", "food", "tools", "chemicals"]
  market_model: "posted_price"   # or "order_book" for higher complexity
  inventory_policy: "base_stock"
  enable_supply_chain_latency: true

finance:
  enable_banks: true
  interbank_market: false

energy:
  grid_resolution: "low"
  dispatch_model: "merit_order_lite"

politics:
  election_rule: "PR"
  gov_ai_control: true

ai:
  mode: "strict"           # "strict" or "exploratory"
  cadence:
    firm_days: 7
    government_days: 30
  providers:
    firm: "rules"          # "rules" | "local" | "remote"
    bank: "rules"
    gov: "local"
  budgets:
    max_tokens_per_decision: 512
    max_calls_per_tick: 50
  local_backend: "ollama"   # "ollama" | "llamacpp" | "mlx"
  remote_backend: "openai"  # or others
  model_map:
    local_default: "q4_0:7b"
    remote_default: "gpt-4o-mini"

logging:
  decisions: "summary"      # "none" | "summary" | "full"
  kpi_interval_days: 30
```

### 7.2 `tiny_island.yaml`

A smaller scenario ideal for early development and testing.

```yaml
name: "tiny_island"
random_seed: 7

spatial:
  level: 7
  width_tiles: 128
  height_tiles: 64

population:
  persons: 50_000
  use_representative_households: true

economy:
  sectors: ["agri", "manufacturing", "services"]
  goods: ["food", "tools", "energy", "services", "steel", "wheat"]
  market_model: "posted_price"
  inventory_policy: "base_stock"

finance:
  enable_banks: true
  interbank_market: false

energy:
  dispatch_model: "merit_order_lite"
  grid_resolution: "low"

ai:
  mode: "strict"
  providers:
    firm: "rules"
    gov: "rules"
    bank: "rules"

logging:
  decisions: "summary"
  kpi_interval_days: 30
```

---

## 8. Persistence & Snapshots

### 8.1 State Storage

* All core state is stored in Arrow-compatible columnar structures.
* Snapshots written as Parquet files with:

  * Partitioning by component type (persons, firms, tiles, etc.).
  * Zstd compression.

### 8.2 Snapshot Strategy

* Save every N ticks (configurable, e.g. 30 days).
* Optionally store deltas only (changed entities) between full snapshots.

### 8.3 Replay

* Deterministic mode allows replay from a snapshot by re-running ticks with the same seed.
* Non-deterministic AI mode can be made replayable by logging AI inputs/outputs.

---

## 9. UI & Developer Tools

### 9.1 Minimal UI Features

* World map with tile-level overlays:

  * Population density
  * Unemployment
  * Prices/inflation
  * Energy supply/demand
* Entity inspector:

  * View a specific firm, government, or tile.
* KPI dashboard:

  * GDP, unemployment, inflation, trade balances, energy reliability.

### 9.2 Developer Utilities

* Step controls: play, pause, step single tick.
* Scenario loader: load YAML scenario and restart.
* Perf panel: current tick time, #entities, memory usage.

---

## 10. Testing & Validation

### 10.1 Invariants

* Non-negative stocks, wealth, and balances.
* Budget identities hold for governments.
* Conservation of goods: inputs, outputs, waste, and inventories reconcile.

### 10.2 Stylized Facts

* Firm size distribution skewness.
* Price dispersion within sectors.
* Responses of unemployment and wages to demand shocks.

### 10.3 Tests

* Unit tests for each subsystem (Rust).
* Property-based tests (e.g., `proptest`) for invariants.
* Scenario regression tests: replay standard scenarios and compare KPI trajectories.

---

## 11. Build & Deployment on macOS

### 11.1 Toolchain

* Rust stable with optimized flags: `-C target-cpu=native -C opt-level=3 -C lto=thin -C codegen-units=1`.
* Python 3.10+ for optional AI tools.
* Node 18+ for the web UI.

### 11.2 Scripts (example)

* `just build` – build engine in release mode.
* `just run` – run default scenario.
* `just bench` – run performance benchmarks.
* `just ui` – start the web UI.

---

## 12. Phased Implementation Plan (Laptop Scope)

### Phase 0 – Core Engine & Tiny Scenario

**Status:** ✅ Complete in the Rust crate (`src/`) with the `tiny_island` scenario and snapshotting CLI.

* ECS, scheduler, RNG, snapshots.
* `tiny_island` scenario with 50k people, posted-price markets disabled at first.
* Target: ≤ 150 ms/tick.

### Phase 1 – Population & Simple Economy

* Implement population, jobs, basic production, and posted-price markets.
* Validate demography and basic macro behavior.

### Phase 2 – Finance, Energy, & Infrastructure

* Add banking, loans, basic energy dispatch, transport.
* Target: `laptop_small` ≤ 400 ms/tick.

### Phase 3 – Tech & Policy

* Add technology DAG, R&D, and government policy module.

### Phase 4 – AI Agents (Local & Remote)

* Implement AI providers, safety gate, and logging.
* First AI-controlled government using local model or API.

### Phase 5 – Optional Modules & Tools

* Health, conflict, diplomacy, advanced UI.
* Scenario editor, plugin system.

---

## 13. Summary

This spec describes a laptop-optimized world simulator that remains ambitious but realistic to build solo on a MacBook. Complexity is controlled with presets, feature flags, and level-of-detail settings. AI is optional, pluggable, and strictly sandboxed. Core subsystems are designed to be modular so you can implement them one by one and still have a playable, evolving world at each stage.
