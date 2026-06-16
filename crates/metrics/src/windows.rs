use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricWindow {
    SevenDays,
    ThirtyDays,
    AllTime,
}

impl MetricWindow {
    pub fn as_name(self) -> &'static str {
        match self {
            Self::SevenDays => "seven_days",
            Self::ThirtyDays => "thirty_days",
            Self::AllTime => "all_time",
        }
    }

    pub fn bounds(self, as_of: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
        let start = match self {
            Self::SevenDays => as_of - Duration::days(7),
            Self::ThirtyDays => as_of - Duration::days(30),
            Self::AllTime => DateTime::<Utc>::UNIX_EPOCH,
        };
        (start, as_of)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::MetricWindow;

    #[test]
    fn all_time_uses_epoch_start() {
        let as_of = Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap();
        let (start, end) = MetricWindow::AllTime.bounds(as_of);
        assert_eq!(start, chrono::DateTime::<Utc>::UNIX_EPOCH);
        assert_eq!(end, as_of);
    }
}
