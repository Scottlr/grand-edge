use chrono::{DateTime, Duration, Utc};
use grand_edge_domain::{
    Gp, HorizonSecs, IntervalPrice, ItemId, MarketRules, OutcomeLabel, PriceInterval, Rate,
    Recommendation, RecommendationAction, RecommendationId, RecommendationOutcome, StrategySignal,
};
use grand_edge_storage::{RecommendationEvidenceRecord, Storage};
use serde::{Deserialize, Serialize};

use crate::MetricsError;

const BREAK_EVEN_TOLERANCE: f64 = 0.001;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeEvaluationConfig {
    pub max_batch_size: usize,
    pub min_history_points: usize,
    pub evaluate_grace_period_secs: i64,
    pub use_passive_fill_model: bool,
    pub market_rules: MarketRules,
    pub price_interval: PriceInterval,
}

impl Default for OutcomeEvaluationConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 500,
            min_history_points: 2,
            evaluate_grace_period_secs: 300,
            use_passive_fill_model: false,
            market_rules: MarketRules::default(),
            price_interval: PriceInterval::OneHour,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeEvaluationJob {
    pub as_of: DateTime<Utc>,
    pub horizon_secs: HorizonSecs,
    pub config: OutcomeEvaluationConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeEvaluationResult {
    pub evaluated: usize,
    pub inserted: usize,
    pub skipped_not_due: usize,
    pub unevaluable: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationPriceMode {
    ConservativeExecutable,
    PassiveEstimated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionOutcomeRule {
    ProfitSeekingBuy,
    CashoutOrSell,
    HoldPosition,
    WatchOnly,
    AvoidRisk,
}

pub fn evaluate_recommendation_outcome(
    recommendation: &Recommendation,
    interval_history_after_as_of: &[IntervalPrice],
    market_rules: &MarketRules,
    evaluated_at: DateTime<Utc>,
) -> Result<RecommendationOutcome, MetricsError> {
    let signal = select_signal(recommendation)?;
    let horizon_at = recommendation.as_of + Duration::seconds(signal.horizon_secs.as_i64());
    let future_rows = sorted_future_rows(recommendation.as_of, interval_history_after_as_of);
    let path_until_horizon = future_rows
        .iter()
        .filter(|row| row.bucket_start <= horizon_at)
        .cloned()
        .collect::<Vec<_>>();
    let horizon_row = future_rows
        .iter()
        .find(|row| row.bucket_start >= horizon_at);

    if recommendation.explanation.market_rules_version != market_rules.version {
        return Ok(unevaluable_outcome(
            recommendation.recommendation_id,
            signal.horizon_secs,
            evaluated_at,
        ));
    }

    if path_until_horizon.is_empty() || horizon_row.is_none() {
        return Ok(unevaluable_outcome(
            recommendation.recommendation_id,
            signal.horizon_secs,
            evaluated_at,
        ));
    }

    let outcome = match rule_for_action(recommendation.action) {
        ActionOutcomeRule::ProfitSeekingBuy => evaluate_buy_like(
            recommendation,
            signal,
            &path_until_horizon,
            market_rules,
            evaluated_at,
        )?,
        ActionOutcomeRule::CashoutOrSell => evaluate_cashout(
            recommendation,
            signal,
            &path_until_horizon,
            horizon_row,
            market_rules,
            evaluated_at,
        )?,
        ActionOutcomeRule::HoldPosition => evaluate_hold(
            recommendation,
            signal,
            &path_until_horizon,
            horizon_row,
            market_rules,
            evaluated_at,
        )?,
        ActionOutcomeRule::WatchOnly => evaluate_watch(
            recommendation,
            signal,
            &path_until_horizon,
            horizon_row,
            evaluated_at,
        )?,
        ActionOutcomeRule::AvoidRisk => evaluate_avoid(
            recommendation,
            signal,
            &path_until_horizon,
            market_rules,
            evaluated_at,
        )?,
    };

    Ok(outcome)
}

pub async fn evaluate_due_recommendations(
    storage: &Storage,
    job: OutcomeEvaluationJob,
) -> Result<OutcomeEvaluationResult, MetricsError> {
    let pending = storage
        .recommendations()
        .list_pending_outcomes(job.config.max_batch_size as i64)
        .await?;
    let mut result = OutcomeEvaluationResult {
        evaluated: 0,
        inserted: 0,
        skipped_not_due: 0,
        unevaluable: 0,
    };

    for recommendation in pending {
        let evidence = storage
            .evidence()
            .evidence_for_recommendation(recommendation.recommendation_id)
            .await?;
        let Some(evidence) = evidence else {
            continue;
        };

        let evaluation_horizon = due_horizon(&evidence).unwrap_or(job.horizon_secs);
        let due_at = recommendation.as_of
            + Duration::seconds(
                evaluation_horizon.as_i64() + job.config.evaluate_grace_period_secs.max(0),
            );
        if job.as_of < due_at {
            result.skipped_not_due += 1;
            continue;
        }

        let history = storage
            .prices()
            .interval_history_between(
                recommendation.item_id,
                job.config.price_interval,
                recommendation.as_of,
                job.as_of,
            )
            .await?;
        let outcome = evaluate_with_history_policy(
            &evidence,
            &history,
            &job,
            evaluation_horizon,
            &job.config.market_rules,
        )?;
        storage
            .outcomes()
            .upsert_recommendation_outcome(&outcome)
            .await?;

        result.evaluated += 1;
        result.inserted += 1;
        if outcome.outcome_label == OutcomeLabel::Unevaluable {
            result.unevaluable += 1;
        }
    }

    Ok(result)
}

fn evaluate_with_history_policy(
    evidence: &RecommendationEvidenceRecord,
    history: &[IntervalPrice],
    job: &OutcomeEvaluationJob,
    horizon_secs: HorizonSecs,
    market_rules: &MarketRules,
) -> Result<RecommendationOutcome, MetricsError> {
    if history.len() < job.config.min_history_points {
        return Ok(unevaluable_outcome(
            evidence.recommendation.recommendation_id,
            horizon_secs,
            job.as_of,
        ));
    }

    evaluate_recommendation_outcome(&evidence.recommendation, history, market_rules, job.as_of)
}

fn evaluate_buy_like(
    recommendation: &Recommendation,
    signal: &StrategySignal,
    path: &[IntervalPrice],
    market_rules: &MarketRules,
    evaluated_at: DateTime<Utc>,
) -> Result<RecommendationOutcome, MetricsError> {
    let entry =
        path.first()
            .and_then(|row| row.avg_high_price)
            .ok_or(MetricsError::MissingPriceData(
                recommendation.recommendation_id,
            ))?;
    let quantity = signal
        .max_quantity
        .map(|value| value.as_i64())
        .unwrap_or(1)
        .max(1);
    let barrier = buy_barrier_outcome(path, entry, signal.take_profit, signal.stop_loss);
    let exit_price = barrier
        .as_ref()
        .map(|event| event.exit_price)
        .unwrap_or_else(|| final_sell_price(path).expect("non-empty path has final sell price"));
    let actual_net_gp = total_net_gp(
        market_rules,
        recommendation.item_id,
        quantity,
        entry,
        exit_price,
    );
    let actual_return =
        rate((exit_price.as_i64() - entry.as_i64()) as f64 / entry.as_i64() as f64)?;
    let excursions = excursions(path, entry)?;
    let hit_take_profit = barrier
        .as_ref()
        .is_some_and(|event| event.hit_reason == "take_profit");
    let hit_stop_loss = barrier
        .as_ref()
        .is_some_and(|event| event.hit_reason == "stop_loss");
    let label = if hit_take_profit || actual_net_gp > Gp::ZERO {
        OutcomeLabel::Win
    } else if hit_stop_loss || actual_net_gp < Gp::ZERO {
        OutcomeLabel::Loss
    } else {
        OutcomeLabel::BreakEven
    };

    Ok(RecommendationOutcome {
        recommendation_id: recommendation.recommendation_id,
        evaluated_at,
        horizon_secs: signal.horizon_secs,
        actual_return: Some(actual_return),
        actual_net_gp: Some(actual_net_gp),
        direction_correct: Some(actual_return.get() > 0.0),
        hit_take_profit,
        hit_stop_loss,
        max_favourable_excursion: excursions.max_favourable_excursion,
        max_adverse_excursion: excursions.max_adverse_excursion,
        outcome_label: label,
    })
}

fn evaluate_cashout(
    recommendation: &Recommendation,
    signal: &StrategySignal,
    path: &[IntervalPrice],
    horizon_row: Option<&IntervalPrice>,
    market_rules: &MarketRules,
    evaluated_at: DateTime<Utc>,
) -> Result<RecommendationOutcome, MetricsError> {
    let immediate_cashout =
        path.first()
            .and_then(|row| row.avg_low_price)
            .ok_or(MetricsError::MissingPriceData(
                recommendation.recommendation_id,
            ))?;
    let hold_exit =
        horizon_row
            .and_then(|row| row.avg_low_price)
            .ok_or(MetricsError::MissingPriceData(
                recommendation.recommendation_id,
            ))?;
    let avoided_delta_gp =
        market_rules.net_profit_per_unit(recommendation.item_id, hold_exit, immediate_cashout);
    let actual_return = rate(
        (hold_exit.as_i64() - immediate_cashout.as_i64()) as f64
            / immediate_cashout.as_i64() as f64,
    )?;
    let entry_anchor = path
        .first()
        .and_then(|row| row.avg_high_price.or(row.avg_low_price))
        .ok_or(MetricsError::MissingPriceData(
            recommendation.recommendation_id,
        ))?;
    let excursions = excursions(path, entry_anchor)?;
    let label = classify_delta(-actual_return.get());

    Ok(RecommendationOutcome {
        recommendation_id: recommendation.recommendation_id,
        evaluated_at,
        horizon_secs: signal.horizon_secs,
        actual_return: Some(actual_return),
        actual_net_gp: Some(avoided_delta_gp),
        direction_correct: Some(actual_return.get() <= 0.0),
        hit_take_profit: false,
        hit_stop_loss: path_hits_stop(path, signal.stop_loss),
        max_favourable_excursion: excursions.max_favourable_excursion,
        max_adverse_excursion: excursions.max_adverse_excursion,
        outcome_label: label,
    })
}

fn evaluate_hold(
    recommendation: &Recommendation,
    signal: &StrategySignal,
    path: &[IntervalPrice],
    horizon_row: Option<&IntervalPrice>,
    market_rules: &MarketRules,
    evaluated_at: DateTime<Utc>,
) -> Result<RecommendationOutcome, MetricsError> {
    let immediate_cashout =
        path.first()
            .and_then(|row| row.avg_low_price)
            .ok_or(MetricsError::MissingPriceData(
                recommendation.recommendation_id,
            ))?;
    let hold_exit =
        horizon_row
            .and_then(|row| row.avg_low_price)
            .ok_or(MetricsError::MissingPriceData(
                recommendation.recommendation_id,
            ))?;
    let hold_delta_gp =
        market_rules.net_profit_per_unit(recommendation.item_id, immediate_cashout, hold_exit);
    let actual_return = rate(
        (hold_exit.as_i64() - immediate_cashout.as_i64()) as f64
            / immediate_cashout.as_i64() as f64,
    )?;
    let entry_anchor = path
        .first()
        .and_then(|row| row.avg_high_price.or(row.avg_low_price))
        .ok_or(MetricsError::MissingPriceData(
            recommendation.recommendation_id,
        ))?;
    let excursions = excursions(path, entry_anchor)?;
    let label = classify_delta(actual_return.get());

    Ok(RecommendationOutcome {
        recommendation_id: recommendation.recommendation_id,
        evaluated_at,
        horizon_secs: signal.horizon_secs,
        actual_return: Some(actual_return),
        actual_net_gp: Some(hold_delta_gp),
        direction_correct: Some(actual_return.get() >= 0.0),
        hit_take_profit: path_hits_take_profit(path, signal.take_profit),
        hit_stop_loss: path_hits_stop(path, signal.stop_loss),
        max_favourable_excursion: excursions.max_favourable_excursion,
        max_adverse_excursion: excursions.max_adverse_excursion,
        outcome_label: label,
    })
}

fn evaluate_watch(
    recommendation: &Recommendation,
    signal: &StrategySignal,
    path: &[IntervalPrice],
    horizon_row: Option<&IntervalPrice>,
    evaluated_at: DateTime<Utc>,
) -> Result<RecommendationOutcome, MetricsError> {
    let entry =
        path.first()
            .and_then(|row| row.avg_high_price)
            .ok_or(MetricsError::MissingPriceData(
                recommendation.recommendation_id,
            ))?;
    let final_sell =
        horizon_row
            .and_then(|row| row.avg_low_price)
            .ok_or(MetricsError::MissingPriceData(
                recommendation.recommendation_id,
            ))?;
    let potential_return =
        rate((final_sell.as_i64() - entry.as_i64()) as f64 / entry.as_i64() as f64)?;
    let excursions = excursions(path, entry)?;
    let label = if path_hits_take_profit(path, signal.take_profit)
        || path_hits_stop(path, signal.stop_loss)
    {
        classify_delta(potential_return.get())
    } else {
        OutcomeLabel::Expired
    };

    Ok(RecommendationOutcome {
        recommendation_id: recommendation.recommendation_id,
        evaluated_at,
        horizon_secs: signal.horizon_secs,
        actual_return: Some(potential_return),
        actual_net_gp: None,
        direction_correct: None,
        hit_take_profit: path_hits_take_profit(path, signal.take_profit),
        hit_stop_loss: path_hits_stop(path, signal.stop_loss),
        max_favourable_excursion: excursions.max_favourable_excursion,
        max_adverse_excursion: excursions.max_adverse_excursion,
        outcome_label: label,
    })
}

fn evaluate_avoid(
    recommendation: &Recommendation,
    signal: &StrategySignal,
    path: &[IntervalPrice],
    market_rules: &MarketRules,
    evaluated_at: DateTime<Utc>,
) -> Result<RecommendationOutcome, MetricsError> {
    let entry =
        path.first()
            .and_then(|row| row.avg_high_price)
            .ok_or(MetricsError::MissingPriceData(
                recommendation.recommendation_id,
            ))?;
    let barrier = buy_barrier_outcome(path, entry, signal.take_profit, signal.stop_loss);
    let exit_price = barrier
        .as_ref()
        .map(|event| event.exit_price)
        .unwrap_or_else(|| final_sell_price(path).expect("non-empty path has final sell price"));
    let skipped_trade_return =
        rate((exit_price.as_i64() - entry.as_i64()) as f64 / entry.as_i64() as f64)?;
    let skipped_trade_gp =
        market_rules.net_profit_per_unit(recommendation.item_id, entry, exit_price);
    let excursions = excursions(path, entry)?;
    let hit_take_profit = barrier
        .as_ref()
        .is_some_and(|event| event.hit_reason == "take_profit");
    let hit_stop_loss = barrier
        .as_ref()
        .is_some_and(|event| event.hit_reason == "stop_loss");
    let label = if hit_stop_loss || skipped_trade_gp < Gp::ZERO {
        OutcomeLabel::Win
    } else if hit_take_profit || skipped_trade_gp > Gp::ZERO {
        OutcomeLabel::Loss
    } else {
        OutcomeLabel::BreakEven
    };

    Ok(RecommendationOutcome {
        recommendation_id: recommendation.recommendation_id,
        evaluated_at,
        horizon_secs: signal.horizon_secs,
        actual_return: Some(skipped_trade_return),
        actual_net_gp: None,
        direction_correct: Some(skipped_trade_return.get() <= 0.0),
        hit_take_profit,
        hit_stop_loss,
        max_favourable_excursion: excursions.max_favourable_excursion,
        max_adverse_excursion: excursions.max_adverse_excursion,
        outcome_label: label,
    })
}

fn due_horizon(evidence: &RecommendationEvidenceRecord) -> Option<HorizonSecs> {
    evidence
        .recommendation
        .explanation
        .strategy_votes
        .iter()
        .map(|signal| signal.horizon_secs)
        .max_by_key(|value| value.as_i64())
        .or_else(|| {
            evidence
                .linked_predictions
                .iter()
                .map(|record| record.prediction.horizon_secs)
                .max_by_key(|value| value.as_i64())
        })
}

fn select_signal(recommendation: &Recommendation) -> Result<&StrategySignal, MetricsError> {
    let preferred = recommendation
        .explanation
        .strategy_votes
        .iter()
        .find(|signal| signal_matches_action(signal, recommendation.action))
        .or_else(|| recommendation.explanation.strategy_votes.first());

    preferred.ok_or(MetricsError::MissingOutcomeSignal(
        recommendation.recommendation_id,
    ))
}

fn signal_matches_action(signal: &StrategySignal, action: RecommendationAction) -> bool {
    use grand_edge_domain::SignalSide;

    matches!(
        (action, signal.side),
        (RecommendationAction::Buy, SignalSide::Buy)
            | (RecommendationAction::Add, SignalSide::Buy)
            | (RecommendationAction::Hold, SignalSide::Hold)
            | (RecommendationAction::Hold, SignalSide::Buy)
            | (RecommendationAction::Cashout, SignalSide::Sell)
            | (RecommendationAction::Cashout, SignalSide::Cashout)
            | (RecommendationAction::Avoid, SignalSide::Avoid)
            | (RecommendationAction::Avoid, SignalSide::Sell)
            | (RecommendationAction::Watch, SignalSide::Watch)
            | (RecommendationAction::Watch, SignalSide::Buy)
    )
}

fn rule_for_action(action: RecommendationAction) -> ActionOutcomeRule {
    match action {
        RecommendationAction::Buy | RecommendationAction::Add => {
            ActionOutcomeRule::ProfitSeekingBuy
        }
        RecommendationAction::Cashout => ActionOutcomeRule::CashoutOrSell,
        RecommendationAction::Hold => ActionOutcomeRule::HoldPosition,
        RecommendationAction::Watch => ActionOutcomeRule::WatchOnly,
        RecommendationAction::Avoid => ActionOutcomeRule::AvoidRisk,
    }
}

fn sorted_future_rows(as_of: DateTime<Utc>, history: &[IntervalPrice]) -> Vec<IntervalPrice> {
    let mut rows = history
        .iter()
        .filter(|row| row.bucket_start > as_of)
        .cloned()
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.bucket_start);
    rows
}

fn final_sell_price(path: &[IntervalPrice]) -> Option<Gp> {
    path.iter()
        .rev()
        .find_map(|row| row.avg_low_price.or(row.avg_high_price))
}

fn total_net_gp(
    market_rules: &MarketRules,
    item_id: ItemId,
    quantity: i64,
    entry: Gp,
    exit: Gp,
) -> Gp {
    Gp(market_rules
        .net_profit_per_unit(item_id, entry, exit)
        .as_i64()
        * quantity)
}

fn classify_delta(delta: f64) -> OutcomeLabel {
    if delta > BREAK_EVEN_TOLERANCE {
        OutcomeLabel::Win
    } else if delta < -BREAK_EVEN_TOLERANCE {
        OutcomeLabel::Loss
    } else {
        OutcomeLabel::BreakEven
    }
}

fn rate(value: f64) -> Result<Rate, MetricsError> {
    Ok(Rate::new(value)?)
}

fn unevaluable_outcome(
    recommendation_id: RecommendationId,
    horizon_secs: HorizonSecs,
    evaluated_at: DateTime<Utc>,
) -> RecommendationOutcome {
    RecommendationOutcome {
        recommendation_id,
        evaluated_at,
        horizon_secs,
        actual_return: None,
        actual_net_gp: None,
        direction_correct: None,
        hit_take_profit: false,
        hit_stop_loss: false,
        max_favourable_excursion: None,
        max_adverse_excursion: None,
        outcome_label: OutcomeLabel::Unevaluable,
    }
}

#[derive(Debug, Clone, Copy)]
struct BarrierOutcome {
    exit_price: Gp,
    hit_reason: &'static str,
}

fn buy_barrier_outcome(
    path: &[IntervalPrice],
    entry: Gp,
    take_profit: Option<Gp>,
    stop_loss: Option<Gp>,
) -> Option<BarrierOutcome> {
    let _ = entry;
    for row in path {
        if let Some(stop_loss) = stop_loss {
            if row
                .avg_low_price
                .is_some_and(|value| value.as_i64() <= stop_loss.as_i64())
            {
                return Some(BarrierOutcome {
                    exit_price: stop_loss,
                    hit_reason: "stop_loss",
                });
            }
        }
        if let Some(take_profit) = take_profit {
            if row
                .avg_high_price
                .is_some_and(|value| value.as_i64() >= take_profit.as_i64())
            {
                return Some(BarrierOutcome {
                    exit_price: take_profit,
                    hit_reason: "take_profit",
                });
            }
        }
    }

    None
}

fn path_hits_take_profit(path: &[IntervalPrice], take_profit: Option<Gp>) -> bool {
    take_profit.is_some_and(|threshold| {
        path.iter().any(|row| {
            row.avg_high_price
                .is_some_and(|value| value.as_i64() >= threshold.as_i64())
        })
    })
}

fn path_hits_stop(path: &[IntervalPrice], stop_loss: Option<Gp>) -> bool {
    stop_loss.is_some_and(|threshold| {
        path.iter().any(|row| {
            row.avg_low_price
                .is_some_and(|value| value.as_i64() <= threshold.as_i64())
        })
    })
}

#[derive(Debug, Clone, Copy)]
struct Excursions {
    max_favourable_excursion: Option<Rate>,
    max_adverse_excursion: Option<Rate>,
}

fn excursions(path: &[IntervalPrice], entry: Gp) -> Result<Excursions, MetricsError> {
    let mut favourable = None;
    let mut adverse = None;

    for row in path {
        if let Some(high) = row.avg_high_price {
            let value = (high.as_i64() - entry.as_i64()) as f64 / entry.as_i64() as f64;
            favourable = Some(favourable.map_or(value, |current: f64| current.max(value)));
        }
        if let Some(low) = row.avg_low_price {
            let value = (low.as_i64() - entry.as_i64()) as f64 / entry.as_i64() as f64;
            adverse = Some(adverse.map_or(value, |current: f64| current.min(value)));
        }
    }

    Ok(Excursions {
        max_favourable_excursion: favourable.map(rate).transpose()?,
        max_adverse_excursion: adverse.map(rate).transpose()?,
    })
}
