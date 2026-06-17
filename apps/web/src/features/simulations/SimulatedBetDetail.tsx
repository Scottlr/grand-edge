import type { PaperBetView } from "../../domain/simulation";

export function SimulatedBetDetail({ bet }: { bet: PaperBetView | null }) {
  if (!bet) {
    return (
      <article className="terminal-panel">
        <p className="terminal-empty-state">No replay detail is available yet.</p>
      </article>
    );
  }

  return (
    <article className="terminal-panel simulation-detail">
      <p className="eyebrow">Selected test detail</p>
      <div className="detailed-score-grid">
        <div>
          <span className="eyebrow">Strategy</span>
          <strong>{bet.strategyId}</strong>
        </div>
        <div>
          <span className="eyebrow">Expected result</span>
          <strong>{bet.expectedNetGp === null ? "Unavailable" : `${bet.expectedNetGp} gp`}</strong>
        </div>
        <div>
          <span className="eyebrow">Actual result</span>
          <strong>{bet.realizedProfitGp === null ? "Open or skipped" : `${bet.realizedProfitGp} gp`}</strong>
        </div>
        <div>
          <span className="eyebrow">Tax paid</span>
          <strong>{bet.taxPaid} gp</strong>
        </div>
        <div>
          <span className="eyebrow">Slippage estimate</span>
          <strong>{bet.slippageEstimateGp === null ? "Unavailable" : `${bet.slippageEstimateGp} gp`}</strong>
        </div>
        <div>
          <span className="eyebrow">Hit reason</span>
          <strong>{bet.hitReason}</strong>
        </div>
        <div>
          <span className="eyebrow">Confidence at entry</span>
          <strong>{bet.confidenceAtEntry === null ? "Unavailable" : `${Math.round(bet.confidenceAtEntry * 100)}%`}</strong>
        </div>
      </div>
    </article>
  );
}
