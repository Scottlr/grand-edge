import type { PaperBetView } from "../../domain/simulation";

function stateLabel(bet: PaperBetView) {
  switch (bet.hitReason) {
    case "open":
      return "Open";
    case "skipped":
      return "Skipped";
    case "stop_loss":
      return "Realized loss";
    case "target_exit":
    case "manual_cashout":
    case "horizon_expired":
    default:
      return bet.realizedProfitGp !== null && bet.realizedProfitGp < 0 ? "Realized loss" : "Realized win";
  }
}

export function PaperBetTimeline({ bets }: { bets: PaperBetView[] }) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Past test trades</p>
      <div className="simulation-bet-list">
        {bets.map((bet) => (
          <div className="simulation-bet-row" key={bet.betId}>
            <div>
              <strong>{bet.itemName}</strong>
              <p>{bet.entryTime}</p>
            </div>
            <div>
              <strong>{bet.modeLabel}</strong>
              <p>{stateLabel(bet)}</p>
            </div>
            <div>
              <strong>{bet.realizedProfitGp === null ? "Unrealized" : `${bet.realizedProfitGp} gp`}</strong>
              <p>{bet.hitReason}</p>
            </div>
          </div>
        ))}
      </div>
    </article>
  );
}
