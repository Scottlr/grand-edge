import type { PaperBetView } from "../../domain/simulation";

export function WinLossDistribution({ bets }: { bets: PaperBetView[] }) {
  const wins = bets.filter((bet) => bet.realizedProfitGp !== null && bet.realizedProfitGp > 0).length;
  const losses = bets.filter((bet) => bet.realizedProfitGp !== null && bet.realizedProfitGp < 0).length;
  const open = bets.filter((bet) => bet.hitReason === "open").length;
  const skipped = bets.filter((bet) => bet.hitReason === "skipped").length;

  return (
    <article className="terminal-panel">
      <p className="eyebrow">Win/loss distribution</p>
      <div className="simulation-distribution-grid">
        <div className="action-keynumber">
          <span className="eyebrow">Realized wins</span>
          <strong>{wins}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Realized losses</span>
          <strong>{losses}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Open</span>
          <strong>{open}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Skipped</span>
          <strong>{skipped}</strong>
        </div>
      </div>
    </article>
  );
}
