//! Strategy traits and signal modules for Grand Edge.

pub mod builtin;
pub mod config;
pub mod context;
pub mod errors;
pub mod registry;
pub mod traits;
pub mod validation;

pub use config::{RiskConfig, StrategyConfig};
pub use context::{LookbackSpec, StrategyContext};
pub use errors::StrategyError;
pub use registry::{StrategyRegistry, StrategyRunResult};
pub use traits::Strategy;
