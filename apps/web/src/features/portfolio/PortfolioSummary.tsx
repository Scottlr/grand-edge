import type { PortfolioSummary as PortfolioSummaryModel } from "../../domain/portfolio";

function formatGp(value: number | null) {
  if (value === null) {
    return "Unavailable";
  }
  return `${value} gp`;
}

export function PortfolioSummary({
  summary,
}: {
  summary: PortfolioSummaryModel;
}) {
  return (
    <article className="terminal-panel portfolio-summary-grid">
      <div className="action-keynumber">
        <span className="eyebrow">Tracked items</span>
        <strong>{summary.trackedItemCount}</strong>
      </div>
      <div className="action-keynumber">
        <span className="eyebrow">Items to sell</span>
        <strong>{summary.itemsToSell}</strong>
      </div>
      <div className="action-keynumber">
        <span className="eyebrow">Items to hold</span>
        <strong>{summary.itemsToHold}</strong>
      </div>
      <div className="action-keynumber">
        <span className="eyebrow">Items at risk</span>
        <strong>{summary.itemsAtRisk}</strong>
      </div>
      <div className="action-keynumber portfolio-summary-profit">
        <span className="eyebrow">Estimated profit after tax</span>
        <strong>{formatGp(summary.estimatedProfitAfterTax)}</strong>
      </div>
    </article>
  );
}
