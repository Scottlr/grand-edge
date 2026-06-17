import type { Recommendation, SimulationRun } from "../../api/types";
import { RecommendationCard } from "../../components/cards/RecommendationCard";
import { glossaryTermsForRecommendation, simpleActionLabel } from "../../components/recommendation/recommendationFixtures";

export function RecommendationInspector({
  recommendation,
  simulations,
}: {
  recommendation: Recommendation | null;
  simulations: SimulationRun[];
}) {
  if (!recommendation) {
    return (
      <aside aria-label="Recommendation inspector" className="terminal-panel command-center-inspector" tabIndex={0}>
        <p className="eyebrow">Inspector</p>
        <h3>Select a recommendation</h3>
        <p className="terminal-panel-copy">
          Pick a row or top action card to see prediction confidence, trade realism, recommendation confidence, and invalidation rules.
        </p>
      </aside>
    );
  }

  const executionLimited =
    recommendation.predictionConfidence !== null &&
    recommendation.executionConfidence !== null &&
    recommendation.predictionConfidence - recommendation.executionConfidence >= 0.2;

  return (
    <aside aria-label="Recommendation inspector" className="terminal-panel command-center-inspector" tabIndex={0}>
      <div className="terminal-panel-header-inline">
        <div>
          <p className="eyebrow">Inspector</p>
          <h3>{recommendation.itemName}</h3>
        </div>
        <span className="terminal-mono">{simpleActionLabel(recommendation)}</span>
      </div>
      <p className="terminal-panel-copy">
        {executionLimited
          ? "Price likely moves in the right direction, but trade realism is weaker. GrandEdge keeps this in watch-first territory instead of sounding like a confident buy."
          : recommendation.primaryReason}
      </p>
      <div className="command-center-confidence-grid">
        <div className="action-keynumber">
          <span className="eyebrow">Prediction confidence</span>
          <strong>{recommendation.predictionConfidence === null ? "Unavailable" : `${Math.round(recommendation.predictionConfidence * 100)}%`}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Trade realism</span>
          <strong>{recommendation.executionConfidence === null ? "Unavailable" : `${Math.round(recommendation.executionConfidence * 100)}%`}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Recommendation confidence</span>
          <strong>{Math.round(recommendation.recommendationConfidence * 100)}%</strong>
        </div>
      </div>
      <RecommendationCard
        action={simpleActionLabel(recommendation)}
        confidence={recommendation.recommendationConfidence}
        confidenceBreakdown={recommendation.confidenceBreakdown}
        dataState={recommendation.dataState}
        expectedNetGp={recommendation.expectedNetGp}
        expectedRoi={recommendation.expectedRoi}
        horizonLabel={`${Math.round(recommendation.horizonSeconds / 3600)}h window`}
        invalidationRules={recommendation.invalidationRules}
        itemName={recommendation.itemName}
        learnTermIds={glossaryTermsForRecommendation(recommendation)}
        modelAgreement={recommendation.modelAgreement}
        primaryReason={recommendation.primaryReason}
        reasons={recommendation.reasons}
        riskLabel={
          recommendation.riskLabel === "low" ||
          recommendation.riskLabel === "medium" ||
          recommendation.riskLabel === "high"
            ? recommendation.riskLabel
            : "unknown"
        }
        strategyVotes={recommendation.strategyVotes}
      />
      <div className="command-center-simulation-history">
        <p className="eyebrow">Simulation history</p>
        {simulations.length > 0 ? (
          <ul className="command-center-simulation-list">
            {simulations.slice(0, 3).map((run) => (
              <li key={run.runId}>
                <strong>{run.name}</strong>
                <span>{run.status}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="terminal-panel-copy">No simulation history is available yet.</p>
        )}
      </div>
    </aside>
  );
}
