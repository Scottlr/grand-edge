import type { HoldingGuidance } from "../../domain/portfolio";

function formatGp(value: number | null) {
  if (value === null) {
    return "Unavailable";
  }
  return `${value} gp`;
}

function formatPercent(value: number | null) {
  if (value === null) {
    return "Unavailable";
  }
  return `${Math.round(value * 100)}%`;
}

export function HoldingActionCard({
  guidance,
}: {
  guidance: HoldingGuidance;
}) {
  return (
    <article className="terminal-panel portfolio-holding-card">
      <div className="portfolio-holding-head">
        <div>
          <p className="eyebrow">Suggested action</p>
          <h3>{guidance.itemName}</h3>
          <p className="terminal-panel-copy">{guidance.headline}</p>
        </div>
        <span className={`terminal-tone terminal-tone-${guidance.tone}`}>{guidance.action}</span>
      </div>

      <div className="portfolio-holding-metrics">
        <div className="action-keynumber">
          <span className="eyebrow">Quantity</span>
          <strong>{guidance.quantity}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Average buy price</span>
          <strong>{formatGp(guidance.avgBuyPrice)}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Current low</span>
          <strong>{formatGp(guidance.currentLow)}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Current high</span>
          <strong>{formatGp(guidance.currentHigh)}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Profit after tax</span>
          <strong>{formatGp(guidance.unrealizedProfitAfterTax)}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Cashout price</span>
          <strong>{formatGp(guidance.cashoutPrice)}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Danger point</span>
          <strong>{formatGp(guidance.stopLoss)}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Confidence</span>
          <strong>{formatPercent(guidance.confidence)}</strong>
        </div>
      </div>

      <p className="portfolio-holding-reason">{guidance.reason}</p>
      {guidance.notes ? <p className="terminal-mono">Notes: {guidance.notes}</p> : null}
    </article>
  );
}
