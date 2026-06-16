//! Strategy traits and signal modules for Grand Edge.

pub mod builtin;
pub mod config;
pub mod context;
pub mod errors;
pub mod math;
pub mod registry;
pub mod traits;
pub mod uncertainty;
pub mod validation;

pub use builtin::{
    AdvancedRiskOverlayStrategy, ArBaselineStrategy, ConformalIntervalStrategy,
    ExecutionConfidenceEstimate, KalmanFairValueStrategy, PortfolioCandidate,
    PortfolioOrderSuggestion, RegimeHmmStrategy, estimate_execution_confidence, optimize_portfolio,
    register_baseline_strategies,
};
pub use config::{RiskConfig, StrategyConfig};
pub use context::{LookbackSpec, StrategyContext};
pub use errors::StrategyError;
pub use math::{
    ArBaselineConfig, ArForecast, KalmanConfig, KalmanState, KalmanUpdate, forecast_next_price,
    kalman_update,
};
pub use registry::{StrategyRegistry, StrategyRunResult};
pub use traits::Strategy;
pub use uncertainty::{
    ConformalInterval, MarketRegime, RegimeEstimate, RegimeHeuristicConfig, RegimeMethod,
    RiskOverlay, RiskOverlayReason, advanced_risk_overlay, classify_regime, conformal_interval,
};
