use std::time::Duration;

use grand_edge_domain::PriceInterval;
use tokio::time::{Interval, MissedTickBehavior};

use crate::IngestionJobConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduledJob {
    SyncMapping,
    Latest,
    Interval(PriceInterval),
}

pub struct PollingSchedule {
    pub sync_mapping: Interval,
    pub latest: Interval,
    pub five_minute: Interval,
    pub one_hour: Interval,
}

impl PollingSchedule {
    pub fn new(config: &IngestionJobConfig) -> Self {
        Self {
            sync_mapping: build_interval(config.sync_mapping_seconds),
            latest: build_interval(config.poll_latest_seconds),
            five_minute: build_interval(config.poll_5m_seconds),
            one_hour: build_interval(config.poll_1h_seconds),
        }
    }
}

fn build_interval(seconds: u64) -> Interval {
    let mut interval = tokio::time::interval(Duration::from_secs(seconds));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    interval
}
