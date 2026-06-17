import type { Position, Recommendation, SimulationRun, StrategyStatus } from "../../api/types";
import { RecommendationStrip } from "./RecommendationStrip";
import { OpportunityTable } from "./OpportunityTable";
import { ModelHealthBar } from "./ModelHealthBar";
import { RecommendationInspector } from "./RecommendationInspector";

function deriveHealth(recommendations: Recommendation[], strategies: StrategyStatus[]) {
  const accuracyEntries = recommendations.map((entry) => entry.accuracy).filter((entry) => entry !== null);
  const recentSimulationAccuracy =
    accuracyEntries.length > 0
      ? accuracyEntries.reduce((sum, entry) => sum + (entry?.directionalAccuracy ?? 0), 0) / accuracyEntries.length
      : null;
  const orderedAccuracy = [...accuracyEntries].sort(
    (left, right) => (right?.directionalAccuracy ?? -1) - (left?.directionalAccuracy ?? -1),
  );

  return {
    liveStrategyCount: strategies.filter((strategy) => strategy.enabled).length,
    recentSimulationAccuracy,
    bestStrategyId: orderedAccuracy[0]?.strategyId ?? null,
    worstStrategyId: orderedAccuracy.at(-1)?.strategyId ?? null,
    marketRegime:
      recommendations.find((entry) => entry.confidenceBreakdown.regimeLabel)?.confidenceBreakdown.regimeLabel ?? null,
  };
}

export function CommandCenterView({
  positions,
  recommendations,
  selectedRecommendationId,
  simulations,
  strategies,
  onSelectRecommendation,
}: {
  positions: Position[];
  recommendations: Recommendation[];
  selectedRecommendationId: string | null;
  simulations: SimulationRun[];
  strategies: StrategyStatus[];
  onSelectRecommendation(recommendationId: string): void;
}) {
  const inspectorRecommendation =
    recommendations.find((entry) => entry.recommendationId === selectedRecommendationId) ?? recommendations[0] ?? null;
  const health = deriveHealth(recommendations, strategies);

  return (
    <section className="command-center-layout">
      <div className="command-center-main">
        <RecommendationStrip
          onSelectRecommendation={onSelectRecommendation}
          positions={positions}
          recommendations={recommendations}
        />
        <OpportunityTable
          onSelectRecommendation={onSelectRecommendation}
          recommendations={recommendations}
          selectedRecommendationId={selectedRecommendationId}
        />
        <ModelHealthBar {...health} />
      </div>
      <RecommendationInspector recommendation={inspectorRecommendation} simulations={simulations} />
    </section>
  );
}
