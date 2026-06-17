# GrandEdge Mathematical Feature and Model Design

Source evidence imported from the pasted document titled `GrandEdge Mathematical Feature and Model Design Document` on 2026-06-16.

The chunks preserve the original headings and code blocks while grouping related sections into smaller files for review and future citation.

| File | Source Sections | Contents |
|---|---:|---|
| [00-overview-and-market-data.md](00-overview-and-market-data.md) | 1-2 | Purpose, prediction/recommendation separation, and OSRS price candle inputs. |
| [01-price-and-return-features.md](01-price-and-return-features.md) | 3-4 | Mid price, spread, spread percentage, simple returns, and log returns. |
| [02-statistical-and-moving-features.md](02-statistical-and-moving-features.md) | 5-6 | Rolling mean, rolling standard deviation, z-score, EMA, and MACD-style momentum. |
| [03-liquidity-tax-and-edge.md](03-liquidity-tax-and-edge.md) | 7-8 | Volume, fill capacity, fill probability, tax, slippage, net profit, and ROI. |
| [04-volatility-technical-and-quality-features.md](04-volatility-technical-and-quality-features.md) | 9-11 | Realised/EWMA/GARCH volatility, Bollinger position, RSI, staleness, and data quality confidence. |
| [05-baseline-strategy-models.md](05-baseline-strategy-models.md) | 12-14 | Spread edge, momentum, and mean reversion strategy models. |
| [06-statistical-forecasting-models.md](06-statistical-forecasting-models.md) | 15-17 | Kalman fair value, AR/ARIMA-style forecasting, and logistic direction classifier. |
| [07-ml-ranking-regime-and-allocators.md](07-ml-ranking-regime-and-allocators.md) | 18-21 | Gradient-boosted ranking, regime detection, contextual bandits, and online ensemble weighting. |
| [08-labeling-and-intervals.md](08-labeling-and-intervals.md) | 22-23 | Triple-barrier labels, meta-labeling targets, and conformal prediction intervals. |
| [09-recommendation-confidence-and-calibration.md](09-recommendation-confidence-and-calibration.md) | 24-26 | Recommendation scoring, action mapping, multi-confidence system, probability calibration, and Brier score. |
| [10-portfolio-and-simulation-metrics.md](10-portfolio-and-simulation-metrics.md) | 27-28 | Expected value, Kelly sizing, constrained portfolio optimisation, and simulation metrics. |
| [11-evidence-contracts-and-artifacts.md](11-evidence-contracts-and-artifacts.md) | 29-33 | Reason atoms, feature snapshots, prediction/recommendation contracts, and Python-to-Rust model artifacts. |
| [12-roadmap-verification-and-system-rule.md](12-roadmap-verification-and-system-rule.md) | 34-36 | Minimum viable model order, required verification rules, and final reconstruction rule. |

## Use Notes

- Treat this as supporting evidence for the implementation plan in `features/rust-backed-osrs-recommendation-terminal/tasks.md`.
- Preserve prediction/recommendation separation, no-lookahead constraints, explicit missing-data handling, and recommendation reconstruction requirements when applying these notes.
- Add future model-design evidence as separate evidence sets under `corpus/model-design/` rather than appending unrelated material here.
