use chrono::{DateTime, Utc};
use grand_edge_domain::PriceInterval;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeseriesCheckpoint {
    pub item_id: i64,
    pub interval: PriceInterval,
    pub completed_at: DateTime<Utc>,
}

pub fn timeseries_checkpoint_key(item_id: i64, interval: PriceInterval) -> String {
    format!(
        "timeseries:{interval_key}:{item_id}",
        interval_key = interval_key(interval)
    )
}

fn interval_key(interval: PriceInterval) -> &'static str {
    match interval {
        PriceInterval::FiveMinute => "5m",
        PriceInterval::OneHour => "1h",
        PriceInterval::SixHour => "6h",
        PriceInterval::TwentyFourHour => "24h",
    }
}

#[cfg(test)]
mod tests {
    use grand_edge_domain::PriceInterval;

    use super::timeseries_checkpoint_key;

    #[test]
    fn checkpoint_key_is_stable() {
        assert_eq!(
            timeseries_checkpoint_key(4151, PriceInterval::TwentyFourHour),
            "timeseries:24h:4151"
        );
    }
}
