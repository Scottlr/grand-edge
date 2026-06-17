import type { Recommendation } from "../../api/types";
import { TooltipTerm } from "../../components/learn/TooltipTerm";
import { DataStatePanel } from "../../components/state/DataStatePanel";
import { emptyStates } from "../../content/emptyStates";
import { simpleActionLabel } from "../../components/recommendation/recommendationFixtures";

export type OpportunityTableProps = {
  recommendations: Recommendation[];
  selectedRecommendationId: string | null;
  onSelectRecommendation(recommendationId: string): void;
};

function dataStateMessage(recommendations: Recommendation[]) {
  const first = recommendations[0];
  if (!first) {
    return `${emptyStates.noBuyRecommendations.title}. ${emptyStates.noBuyRecommendations.message}`;
  }
  if (first.dataState === "stale") {
    return "Data is stale. Recommendations are paused until fresh prices arrive.";
  }
  if (first.dataState === "degraded") {
    return "Recommendation evidence is degraded. GrandEdge is keeping the advice cautious until the missing pieces recover.";
  }
  if (first.dataState === "error") {
    return "Recommendation updates failed. GrandEdge is showing the error honestly instead of inventing advice.";
  }
  return null;
}

export function OpportunityTable({
  recommendations,
  selectedRecommendationId,
  onSelectRecommendation,
}: OpportunityTableProps) {
  const stateMessage = dataStateMessage(recommendations);

  if (stateMessage) {
    return (
      <DataStatePanel
        state={recommendations[0]?.dataState ?? "empty"}
        title="Current actions"
        message={stateMessage}
      />
    );
  }

  return (
    <article className="terminal-panel">
      <div className="terminal-panel-header-inline">
        <div>
          <p className="eyebrow">Opportunity table</p>
          <h3>Current actions</h3>
        </div>
        <p className="terminal-panel-copy">
          Rows lead with action, one-sentence why, and expected profit. Deeper confidence stays one click away.
        </p>
      </div>
      <div className="command-center-table-wrap">
        <table className="command-center-table">
          <thead>
            <tr>
              <th>Rank</th>
              <th>Item</th>
              <th>Action</th>
              <th>Why</th>
              <th>
                <TooltipTerm term="confidence">Confidence</TooltipTerm>
              </th>
              <th>
                <TooltipTerm term="expectedProfit">Expected profit</TooltipTerm>
              </th>
              <th>
                <TooltipTerm term="suggestedQuantity">Suggested quantity</TooltipTerm>
              </th>
              <th>Timeframe</th>
            </tr>
          </thead>
          <tbody>
            {recommendations.map((recommendation, index) => {
              const isSelected = recommendation.recommendationId === selectedRecommendationId;
              return (
                <tr
                  aria-selected={isSelected}
                  className={isSelected ? "command-center-row-selected" : undefined}
                  key={recommendation.recommendationId}
                  onKeyDown={(event) => {
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      onSelectRecommendation(recommendation.recommendationId);
                    }
                  }}
                  onClick={() => onSelectRecommendation(recommendation.recommendationId)}
                  role="button"
                  tabIndex={0}
                >
                  <td>{index + 1}</td>
                  <td>
                    <div className="command-center-item-cell">
                      {recommendation.itemIcon?.cdnUrl ? (
                        <img alt={recommendation.itemName} className="command-center-item-icon" src={recommendation.itemIcon.cdnUrl} />
                      ) : (
                        <span className="command-center-item-fallback" aria-hidden="true">
                          {recommendation.itemName.slice(0, 2).toUpperCase()}
                        </span>
                      )}
                      <span>{recommendation.itemName}</span>
                    </div>
                  </td>
                  <td>{simpleActionLabel(recommendation)}</td>
                  <td>{recommendation.primaryReason}</td>
                  <td>{Math.round(recommendation.recommendationConfidence * 100)}%</td>
                  <td>{recommendation.expectedNetGp === null ? "Unavailable" : `${recommendation.expectedNetGp} gp`}</td>
                  <td>{recommendation.strategyVotes[0]?.maxQuantity?.toString() ?? "Unavailable"}</td>
                  <td>{Math.round(recommendation.horizonSeconds / 3600)}h</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </article>
  );
}
