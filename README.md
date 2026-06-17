# Grand Edge

Grand Edge is a Rust-backed OSRS Grand Exchange recommendation and
paper-trading terminal. It turns OSRS Wiki market data into clear buy, sell,
hold, wait, and watch guidance with the evidence needed to judge each call.

It is built for players who want more than a margin list. Grand Edge combines
price movement, spread, tax, buy limits, stale data, observed volume proxies,
model agreement, recent accuracy, portfolio exposure, and paper-trading results
before it suggests an action.

Grand Edge is not an OSRS bot. It does not log in, store game credentials, click
the Grand Exchange UI, or place orders. Recommendations are evidence-backed
decision support, and paper trading is used to test whether the advice would
have held up under conservative fill assumptions.

## What It Helps Answer

- What should I do now: buy, sell, hold, wait, or watch?
- Why is this recommendation appearing?
- How reliable is the signal compared with recent outcomes?
- What price, tax, spread, and liquidity assumptions matter?
- What happened last time similar advice appeared?
- How would this affect my current holdings and exposure?
- Which connected items, recipes, repairs, or market events could change the
  picture?

## Feature Set

- Action-first command center for the highest-priority opportunities and current
  holdings.
- Buy, sell, portfolio, linked-item, simulation, accuracy, and settings views
  organized around user decisions rather than model internals.
- OSRS Wiki price ingestion for latest prices, interval candles, item mapping,
  and item image metadata through a descriptive configured `User-Agent`.
- Typed Rust domain contracts for items, GP values, quantities, probabilities,
  horizons, market rules, predictions, recommendations, evidence, and outcomes.
- Deterministic feature generation that keeps market observations, features,
  predictions, recommendations, and outcomes as separate auditable layers.
- Strategy modules for spread edge, momentum, mean reversion, volatility filters,
  execution confidence, portfolio optimization, and advanced research-backed
  baselines.
- Recommendation scoring that accounts for tax, spread, liquidity confidence,
  fill estimates, user risk settings, current holdings, calibration, and model
  agreement.
- Structured evidence atoms, confidence breakdowns, invalidation rules, model
  vote summaries, and plain-language explanations.
- Conservative paper-trading simulation with instant buy-at-high and sell-at-low
  semantics, plus proxy passive-fill modes that avoid lookahead.
- Accuracy, calibration, reason-level, risk, forecast, and trading metrics based
  on persisted predictions and realized outcomes.
- Market graph intelligence for substitutes, recipes, item sets, charge links,
  repair links, alchemy relationships, event notes, and blast-radius scenarios.
- Offline analytics and ML research workflow where Python can train or export
  artifacts, while Rust validates and serves production inference.
- Axum API, generated schemas, OpenAPI output, live event surfaces, and a
  React/Vite terminal UI.
- Local no-Docker and Docker-assisted development paths with runbooks and helper
  scripts.

## How It Works

1. Grand Edge ingests OSRS Wiki market snapshots and item metadata in bulk.
2. The Rust ingestion layer normalizes prices, timestamps, missing values, and
   market metadata into owned records.
3. Storage keeps raw-enough observations so recommendations and backtests can be
   reconstructed later.
4. Feature generation builds no-lookahead snapshots from data that existed at
   the decision time.
5. Strategy modules and validated model artifacts emit predictions with stable
   strategy IDs, versions, horizons, confidence, and evidence.
6. The recommender turns predictions into user actions after applying market
   rules, spread, tax, liquidity uncertainty, holdings, risk settings, and recent
   calibration.
7. Every recommendation persists its evidence chain: market snapshot, feature
   snapshot, supporting predictions, score components, reason atoms, confidence,
   and outcome status.
8. The simulator replays recommendations through conservative fill rules, and
   metrics feed accuracy and reason-performance views back into the terminal.

## Product Surfaces

- **Dashboard:** current best action, model health, top opportunities, and
  recommendation inspector.
- **Buy and Sell:** focused action queues with reasons, confidence, and suggested
  guardrails.
- **Portfolio:** holdings, exposure, after-tax PnL, and guidance for hold,
  cashout, trim, or watch decisions.
- **Items:** item intelligence, price context, liquidity signals, and related
  market evidence.
- **Linked Items:** relationship paths, upstream and downstream pressure, event
  impacts, and portfolio contagion checks.
- **Simulations:** paper-bet replay, conservative versus proxy fill comparison,
  win/loss distribution, drawdown, stops, targets, and horizon outcomes.
- **Accuracy:** model cards, calibration, sample size, recent performance, and
  reason-level trust signals.
- **Settings:** strategy toggles, risk preferences, API/session surfaces, and
  local environment configuration.

## Built For Auditability

Grand Edge treats predictions and recommendations as different things. A model
can expect an item to rise while the recommender still returns watch or avoid if
spread, tax, stale data, weak observed volume, poor calibration, or portfolio
exposure makes the trade unattractive.

The goal is to make every recommendation reproducible from stored inputs: what
data was available, which features were computed, which strategies voted, how
confidence was formed, what action was chosen, and what happened afterward.

## Run It Locally

No Docker:

- `pwsh ./scripts/dev/grandedge-dev.ps1 doctor`
- `pwsh ./scripts/dev/grandedge-dev.ps1 no-docker`

Docker:

- `pwsh ./scripts/dev/grandedge-dev.ps1 docker-up`

Expected local URLs:

- API health: `http://localhost:3000/health`
- OpenAPI: `http://localhost:3000/api/openapi.json`
- Frontend: `http://localhost:5173`

## Documentation

- [Developer guide](docs/developer-guide.md): commands, architecture, schemas,
  run modes, and ML workflow.
- [No-Docker setup](docs/running/no-docker.md)
- [Docker setup](docs/running/docker.md)
- [ML workflow](docs/running/ml-workflow.md)
- [Feature plan](features/rust-backed-osrs-recommendation-terminal/tasks.md)
