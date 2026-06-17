import type { RecommendationOutcome } from "../../domain/evidence";

function formatPercent(value: number | null) {
  if (value === null) {
    return "Unavailable";
  }
  return `${Math.round(value * 100)}%`;
}

function formatGp(value: number | null) {
  if (value === null) {
    return "Unavailable";
  }
  return `${value} gp`;
}

export function OutcomeSummaryPanel({
  outcome,
  pending,
}: {
  outcome: RecommendationOutcome | null;
  pending: boolean;
}) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Outcome evaluation</p>
      {outcome ? (
        <div className="simulation-summary-grid">
          <div className="action-keynumber">
            <span className="eyebrow">Result</span>
            <strong>{outcome.outcomeLabel.replaceAll("_", " ")}</strong>
          </div>
          <div className="action-keynumber">
            <span className="eyebrow">Actual return</span>
            <strong>{formatPercent(outcome.actualReturn)}</strong>
          </div>
          <div className="action-keynumber">
            <span className="eyebrow">Actual net gp</span>
            <strong>{formatGp(outcome.actualNetGp)}</strong>
          </div>
        </div>
      ) : pending ? (
        <p className="terminal-panel-copy">Outcome is still pending because the review window has not elapsed yet.</p>
      ) : (
        <p className="terminal-panel-copy">Outcome is not available for this recommendation yet.</p>
      )}
    </article>
  );
}
