# Grand Edge Agent Instructions

Grand Edge is a Rust-backed OSRS Grand Exchange recommendation and paper-trading
terminal. The current source of truth is
`features/rust-backed-osrs-recommendation-terminal/tasks.md`, with detailed task
files in `features/rust-backed-osrs-recommendation-terminal/tasks/`.

Treat this repository as greenfield but not vague: the feature spec defines the
architecture, safety boundaries, audit model, and implementation order. Build one
coherent slice at a time and preserve the planning invariants unless a human
explicitly changes them.

## Installed Skills to Use

Use the installed Codex skills intentionally. Read the relevant `SKILL.md` before
work that falls under it.

- `rust-engineering`: Rust workspace setup, crates, domain contracts, errors,
  serde contracts, tests, Cargo commands, and architecture decisions.
- `rust-concurrency-threading`: Tokio services, ingestion loops, schedulers,
  channels, shared state, SSE/WebSocket streams, cancellation, and backpressure.
- `rust-packaging-release`: Cargo workspace metadata, feature flags, dependency
  placement, lockfiles, toolchains, packaging, and release hygiene.
- `trading-systems`: market data, OSRS Wiki price API usage, features,
  strategies, recommendations, simulation, paper trading, metrics, risk, and
  no-lookahead review.
- `large-feature-planning`: only when expanding or restructuring the feature
  plan, not for routine implementation.
- `browser:control-in-app-browser`: frontend/local browser QA after meaningful
  UI changes.
- `github:github`, `github:gh-fix-ci`, `github:gh-address-comments`, and
  `github:yeet`: GitHub orientation, CI fixes, review comments, and publishing.

Repo-local GrandEdge GitHub issue skills live under `docs/agent-skills/`.
Use these path-specific skills for GrandEdge issue workflows instead of any
global issue skills that target other repositories:

- `docs/agent-skills/grandedge-github-issue-writing/SKILL.md`: create one
  GitHub issue per feature task with `Blocked By`, `Blocking`, labels, and task
  table links for `Scottlr/grand-edge`.
- `docs/agent-skills/grandedge-github-issue-selector/SKILL.md`: select one
  unblocked implementation issue without modifying GitHub state.
- `docs/agent-skills/grandedge-github-issue-complete-one/SKILL.md`: complete one
  selected issue through branch, PR, merge, close, and cleanup.
- `docs/agent-skills/grandedge-github-issue-drain-queue/SKILL.md`: repeatedly
  select and complete issues until the queue is empty or blocked.
- `docs/agent-skills/grandedge-github-issue-loop/SKILL.md`: self-contained
  autonomous implementation loop for `Scottlr/grand-edge`.

## Project Boundaries

- This is a recommendation and paper-trading terminal, not an OSRS bot.
- Do not build game-client automation, credential storage, login flows, or code
  that clicks the Grand Exchange UI.
- OSRS Wiki price API data is the external market-data source of truth.
- Rust owns the production spine: ingestion, features, inference,
  recommendations, simulation, API, and live dashboard streams.
- Python is allowed only for research, notebooks, offline training, evaluation
  reports, and artifact export. Production Rust must not require Python.

## Planned Architecture

Respect these crate ownership boundaries as the workspace takes shape:

- `crates/domain`: shared IDs, newtypes, enums, DTOs, market rules, and
  cross-crate contracts.
- `crates/storage`: migrations, repositories, query contracts, and persistence
  ownership.
- `crates/ingest`: the only OSRS Wiki HTTP client and normalization owner.
- `crates/features`: deterministic feature generation from stored observations.
- `crates/strategies`: strategy traits, registries, validation, and signals.
- `crates/simulator`: paper-trading orders, fills, taxes, stops, targets, and
  replay logic.
- `crates/metrics`: accuracy, calibration, risk, reason-level, and trading
  metrics.
- `crates/recommender`: user-facing actions, risk settings, evidence, and
  explanations.
- `crates/api`: Axum routes, OpenAPI, auth/session surfaces, and live events.
- `crates/model_runtime`: Rust validation and serving of exported model
  artifacts.
- `apps/web`: React/Vite terminal UI and typed API consumption.
- `ml`: research-only Python workspace. It must import Rust where useful; Rust
  production code must not import Python.

Lower-level crates must not depend on API or frontend crates. Strategy,
simulator, recommender, API, and frontend bindings should consume shared domain
contracts rather than duplicate enums or string values.

## Clean Code Rules

- Prefer explicit structs, enums, and newtypes over primitive-heavy or stringly
  typed logic.
- Validate invariants at constructors and repository/API boundaries.
- Use local error enums and `Result<T, E>` for recoverable failures. Avoid
  panics outside tests and impossible invariants.
- Keep modules small and cohesive. Add abstractions only when they remove real
  duplication or clarify ownership.
- Use comments sparingly for non-obvious domain or systems reasoning.
- Keep public contracts stable and intentional. For serde types, use explicit
  rename/default/skip rules where compatibility depends on them.
- Do not hide blocking I/O in async code. Use `spawn_blocking` or an established
  blocking boundary.

## Rust Code Organisation and Testing Guidelines

Core principle: keep Rust files small, focused, and easy for both humans and
coding agents to reason about. Avoid turning implementation files into
token-heavy dumping grounds. Large files make agentic changes slower, less
reliable, and harder to review.

### Testing Policy

Inline `#[cfg(test)]` modules are allowed, but only for small unit tests that are
tightly coupled to private implementation details.

Good inline test candidates:

- Small pure functions.
- Private helper behaviour.
- Edge cases for local logic.
- Small regression tests.
- Invariants that are easiest to test beside the implementation.

Avoid putting large scenario tests, fixture-heavy tests, or full pipeline tests
inside production source files. Inline tests should be surgical, not exhaustive.

Larger behavioural tests belong in `tests/`. Use Rust integration tests for
public behaviour and feature-level coverage:

```text
crate_name/
  src/
    lib.rs
    parse/
    lower/
    render/
  tests/
    parse_tests.rs
    lower_tests.rs
    pipeline_tests.rs
    diagnostics_tests.rs
```

Use `tests/` for:

- Public API behaviour.
- Parser/compiler/lowering pipeline tests.
- End-to-end feature tests.
- Regression tests involving realistic input.
- Tests that describe how the crate should behave from the outside.

If a test reads like product behaviour rather than private implementation
detail, it probably belongs in `tests/`.

Put shared test helpers in `tests/common/`:

```text
tests/
  common/
    mod.rs
  parse_tests.rs
  lower_tests.rs
```

Use `tests/common/mod.rs` for shared fixture loaders, common assertion helpers,
test builders, and reusable setup functions. Keep helpers test-only. Do not
pollute production modules with test scaffolding.

Put large fixtures outside Rust code. Do not embed large XML, JSON, SVG, XVSVG,
RON, or generated data directly inside test functions unless the example is very
small.

```text
tests/
  fixtures/
    xvsvg/
      basic_box.xvsvg
      expressive_player.xvsvg
      malformed_missing_viewbox.xvsvg
```

Load fixtures with `include_str!("fixtures/xvsvg/basic_box.xvsvg")`. Large test
data belongs in fixture files, not production files and not giant inline
strings.

Use snapshot testing for large outputs. Do not hand-write huge expected outputs
inside Rust tests. For large diagnostics, manifests, lowered models, ASTs,
compiled artifacts, source maps, render manifests, generated metadata, or
pretty-printed debug output, prefer snapshot testing such as `insta` where
appropriate. The test should describe the intent; the snapshot should hold the
large expected output.

Prefer table-driven tests over many repetitive tests. Use `rstest` when testing
many cases of the same behaviour:

```rust
#[rstest]
#[case("Idle", true)]
#[case("Run", true)]
#[case("", false)]
fn validates_animation_names(#[case] name: &str, #[case] expected: bool) {
    assert_eq!(is_valid_animation_name(name), expected);
}
```

If the test shape is repeated, consolidate it.

### DTO and Contract Organisation Policy

Do not create generic dumping-ground files. Avoid files named:

- `contracts.rs`
- `dto.rs`
- `dtos.rs`
- `models.rs`
- `types.rs`

These files tend to grow without ownership boundaries. Contracts are not a layer
by themselves. They belong to the feature or domain concept they describe.

Organise types by domain ownership. Prefer vertical ownership:

```text
src/
  source/
    mod.rs
    sprite_document.rs
    visual_shape.rs
    animation.rs
    collision.rs
    socket.rs
  parse/
    mod.rs
    xml_parser.rs
    parse_error.rs
  validate/
    mod.rs
    validator.rs
    diagnostic.rs
  lower/
    mod.rs
    lowered_sprite.rs
    artifact.rs
    source_map.rs
  render/
    mod.rs
    render_view.rs
    manifest.rs
```

Avoid horizontal buckets:

```text
src/
  contracts.rs
  dtos.rs
  models.rs
  types.rs
```

Keep source, parsed, validated, lowered, and rendered types separate. Use clear
ownership boundaries. For XVSVG/vector-style crates, prefer:

| Area | Owns |
|---|---|
| `source/` | Authored source model and user-facing document structures |
| `parse/` | XML/parser-specific intermediates and parse errors |
| `validate/` | Semantic validation and diagnostics |
| `lower/` | Compiled/runtime-facing structures |
| `render/` | Render/debug/export views and manifests |

Do not mix authored source DTOs, parser internals, compiled artifacts, and
render/debug views in the same large file.

Re-export through module boundaries. Use `mod.rs` to expose a clean public
surface while keeping files small:

```rust
mod sprite_document;
mod visual_shape;
mod animation;
mod collision;
mod socket;

pub use animation::*;
pub use collision::*;
pub use socket::*;
pub use sprite_document::*;
pub use visual_shape::*;
```

Callers should get a clean API without requiring every type to live in one huge
file.

A file may contain several closely related types. Avoid splitting every tiny
enum into its own file too early, but split a file when:

- It exceeds roughly 300-500 lines.
- It contains unrelated concepts.
- Most changes touch only one part of the file.
- Tests or fixtures dominate the file.
- Agents repeatedly append new unrelated types to the bottom.

Files should be coherent, not merely short.

### Agent Behaviour Rules

When modifying Rust code:

1. Do not append new DTOs or contracts to a generic catch-all file.
2. Find the domain module that owns the concept.
3. Create a focused file if no suitable module exists.
4. Re-export public types through the module boundary.
5. Keep inline tests small and local.
6. Put behavioural, scenario, and pipeline tests in `tests/`.
7. Put large inputs in `tests/fixtures/`.
8. Put large expected outputs in snapshots.
9. Use table-driven tests for repeated cases.
10. Do not let production source files grow mainly because of tests.

The goal is not minimal file count. The goal is clear ownership, low token
noise, and predictable places for future changes.

## Rust Systems and Zero-Copy Guidance

Write simple, correct Rust first, then make hot paths allocation-aware.

- Use references, slices, iterators, and borrowed parameters (`&str`, `&[T]`)
  where ownership is not needed.
- Avoid casual `.clone()` in ingestion, feature generation, strategy loops,
  recommendation scoring, and serialization paths. If cloning is clearer and not
  hot, prefer clarity.
- Keep persisted/domain records owned when they must outlive a request, task, or
  transaction. Use borrowed DTOs only where lifetimes remain obvious.
- Consider `Cow<'a, str>`, `Arc<str>`, `Bytes`, and streaming serde patterns for
  large API payloads or fanout surfaces when they reduce copying without making
  lifetimes fragile.
- Do not introduce `unsafe` for performance unless there is a measured bottleneck,
  a documented safety invariant, and no safe alternative.
- Represent GP, quantities, basis points, probabilities, horizons, timestamps,
  and IDs with explicit types. Avoid `f64` for money-like values.
- Benchmark important loops with Criterion and realistic fixtures before
  optimizing APIs around assumptions.

## Async, Concurrency, and Services

- Use Tokio tasks for network services, ingestion polling, timers, async database
  work, and live event streams.
- Use bounded channels where unbounded growth could exhaust memory.
- Make cancellation and shutdown explicit for loops, streams, and spawned tasks.
- Do not hold `Mutex` or `RwLock` guards across `.await`.
- Prefer `tokio::sync` primitives in async code and `std::sync` primitives in
  synchronous code unless the surrounding crate has a clear reason to differ.
- Surface task failures through joins, logs, health state, or channels. Avoid
  detached tasks with silent failure modes.

## Trading and Evidence Invariants

- Fetch `/latest` in bulk and filter locally for normal ingestion. Item-specific
  latest calls are debug or smoke-test tools.
- Every OSRS Wiki request must use a descriptive configured `User-Agent`.
- Treat API prices as near-real-time snapshots updated every few minutes, not
  tick data or order-book depth.
- Convert Unix seconds to UTC timestamps at ingestion boundaries.
- Missing high/low/average/volume values are real states. Model them with
  `Option` or explicit skip reasons, never fake zeroes.
- Avoid lookahead in features, strategy signals, recommendations, simulations,
  and metrics.
- Instant simulation buys at high and sells at low. Passive fills may use only
  future observations after order creation.
- Market rules such as tax, caps, thresholds, slot limits, and buy-limit windows
  must be versioned configuration.
- Keep predictions separate from recommendations. Predictions describe expected
  item behavior; recommendations decide action after taxes, spread, liquidity,
  holdings, risk, and calibration.
- Store structured reason atoms, confidence breakdowns, recommendation-prediction
  links, outcomes, and reason-level metrics. Prose explanations are derived from
  structured evidence.
- Given a `recommendation_id`, the system must be able to reconstruct the market
  data, feature snapshot, predictions, strategy/model versions, weights, action,
  reasons, confidence, outcome, and reasoning quality.

## Storage and API

- The storage crate owns migrations and query contracts.
- Do not construct SQL with ad hoc string concatenation in API or frontend code.
- Store normalized, queryable tables for raw-enough observations and derived
  artifacts. Do not compress JSON blobs and call storage solved.
- Keep hot operational storage and cold historical storage conceptually separate.
- API routes should expose typed contracts generated from or aligned with domain
  types. Do not duplicate business enums in frontend-only code.
- Live streams must communicate stale, degraded, empty, and error states rather
  than silently serving confident advice from bad data.

## Frontend Product Rules

The UI should feel calm, analytical, dark, precise, and trust-building.

- Build the actual terminal experience, not a marketing landing page.
- The first screen should answer what action to take, why, how reliable it is,
  and what happened last time similar advice appeared.
- Do not use gamer-neon, crypto-casino styling, flashing prices, red/green
  overload, or fake precision.
- Every major component needs `loading`, `live`, `stale`, `degraded`, `empty`,
  and `error` states.
- Confidence displays must connect to model agreement, recent accuracy, data
  quality, calibration, liquidity, or explicit uncertainty.
- Never use copy like "guaranteed", "sure thing", "free money", "always buy", or
  "risk-free".
- After significant UI work, run the local app and verify with
  `browser:control-in-app-browser` screenshots/interactions at desktop and
  mobile sizes.

## Verification Expectations

Use the smallest meaningful check first, then broaden.

- After Rust changes, prefer `cargo fmt --check`, focused `cargo test -p <crate>`,
  then `cargo check --workspace`.
- Add `cargo clippy --workspace --all-targets -- -D warnings` when the workspace
  has stabilized around that standard.
- Use deterministic fixtures before live API work.
- Live OSRS API tests must be ignored/gated and never required for normal CI.
- Add property tests for money, tax, quantity, risk, fill, and no-lookahead
  invariants.
- Add snapshot tests for stable evidence payloads and frontend view models.
- Add mocked HTTP tests for ingestion and containerized database tests for
  storage boundaries when those crates exist.
- Bench hot ingestion, feature, scoring, recommendation, and serialization paths
  once realistic fixtures exist.

## Task Workflow

- Start from `tasks.md`, confirm dependencies, then read the specific `T###.md`
  file before implementation.
- Keep each implementation aligned with its task's planned files and acceptance
  checks.
- Do not skip early domain newtype and evidence-ledger work; the spec calls out
  those tasks as early foundations even though their IDs were appended later.
- Implement the corpus router task (`T060`) early after scaffolding. Corpus-heavy
  tasks should use its route-first docs/skill instead of loading broad corpus
  files into context.
- Keep changes scoped. Avoid unrelated refactors, dependency churn, or style
  rewrites.
- Update task tracking only when the requested workflow calls for it or when a
  task is genuinely complete.
- If a task conflicts with these instructions, pause and call out the conflict
  with file references and the exact invariant at risk.

## Completion Checklist

Before handing work back:

- The relevant installed skills were used.
- The task spec and architecture boundaries were followed.
- Market, evidence, no-lookahead, and safety invariants still hold.
- Tests/checks were run or the reason they could not run is documented.
- No secrets, credentials, live automation, or unrelated user changes were
  introduced.
- The final response states what changed, what was verified, and any remaining
  risk plainly.
