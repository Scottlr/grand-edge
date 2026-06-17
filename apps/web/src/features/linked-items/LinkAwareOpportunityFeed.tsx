import type { LinkAwareOpportunity } from "./linkedItemTypes";

function percent(value: number) {
  return `${Math.round(value * 100)}%`;
}

export function LinkAwareOpportunityFeed({
  opportunities,
}: {
  opportunities: LinkAwareOpportunity[];
}) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Link-aware opportunity feed</p>
      <div className="terminal-list">
        {opportunities.map((opportunity) => (
          <div className="terminal-list-row" key={opportunity.id}>
            <div>
              <strong>{opportunity.category}</strong>
              <p>{opportunity.headline}</p>
              <p className="linked-path-caveat">
                {opportunity.predictiveOnly
                  ? "Predictive evidence only. This does not claim causality."
                  : opportunity.detail}
              </p>
            </div>
            <div>
              <strong>{percent(opportunity.confidence)}</strong>
              <p>{opportunity.predictiveOnly ? "Predictive" : "Context-backed"}</p>
            </div>
          </div>
        ))}
      </div>
    </article>
  );
}
