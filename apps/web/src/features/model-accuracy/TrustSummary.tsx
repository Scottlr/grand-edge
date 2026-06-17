import type { TrustSummaryViewModel } from "../../domain/modelAccuracy";

function valueOrUnavailable(value: number | string | null) {
  if (value === null) {
    return "Unavailable";
  }

  return String(value);
}

export function TrustSummary({ summary }: { summary: TrustSummaryViewModel }) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Can I trust this?</p>
      <div className="simulation-summary-grid">
        <div className="action-keynumber">
          <span className="eyebrow">BUY calls profitable</span>
          <strong>{valueOrUnavailable(summary.buyCallsProfitable)}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">SELL calls protected profit</span>
          <strong>{valueOrUnavailable(summary.sellCallsProtectedProfit)}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Average profit per successful call</span>
          <strong>{summary.averageProfitPerSuccessfulCall === null ? "Unavailable" : `${summary.averageProfitPerSuccessfulCall} gp`}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Best method</span>
          <strong>{valueOrUnavailable(summary.bestMethodThisWeek)}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Weakest method</span>
          <strong>{valueOrUnavailable(summary.weakestMethodThisWeek)}</strong>
        </div>
      </div>
      <p className="terminal-panel-copy">{summary.confidenceHonestySentence ?? "Confidence honesty detail is unavailable right now."}</p>
    </article>
  );
}
