use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Gp, HorizonSecs, Rate, RecommendationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeLabel {
    Win,
    Loss,
    BreakEven,
    Expired,
    Unevaluable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationOutcome {
    pub recommendation_id: RecommendationId,
    pub evaluated_at: DateTime<Utc>,
    pub horizon_secs: HorizonSecs,
    pub actual_return: Option<Rate>,
    pub actual_net_gp: Option<Gp>,
    pub direction_correct: Option<bool>,
    pub hit_take_profit: bool,
    pub hit_stop_loss: bool,
    pub max_favourable_excursion: Option<Rate>,
    pub max_adverse_excursion: Option<Rate>,
    pub outcome_label: OutcomeLabel,
}

#[cfg(test)]
mod tests {
    use crate::OutcomeLabel;

    #[test]
    fn outcome_label_distinguishes_unevaluable() {
        assert_eq!(
            serde_json::to_string(&OutcomeLabel::Unevaluable).unwrap(),
            "\"unevaluable\""
        );
        assert_ne!(OutcomeLabel::Unevaluable, OutcomeLabel::Loss);
    }
}
