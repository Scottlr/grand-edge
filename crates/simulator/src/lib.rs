//! Paper-trading simulation for Grand Edge.

pub mod blast_radius;
pub mod config;
pub mod engine;
pub mod errors;
pub mod fills;
pub mod orders;
pub mod pnl;
pub mod replay;

pub use blast_radius::{
    BlastRadiusConfig, BlastRadiusImpact, BlastRadiusResult, BlastScenarioMode,
    BlastScenarioMultipliers, Shock, ShockType, propagate_shock, scenario_multipliers,
    simulate_blast_radius,
};
pub use config::SimulatorConfig;
pub use engine::SimulationEngine;
pub use errors::SimulatorError;
pub use orders::{
    PaperBetOutcome, SimulatedOrderRequest, SimulatedOrderSide, SimulatedOrderStatus,
};
