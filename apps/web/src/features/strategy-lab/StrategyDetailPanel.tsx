import { ExpandableAdvancedPanel } from "../../components/disclosure/ExpandableAdvancedPanel";
import type { StrategyLabDetail, StrategyLabRow } from "../../domain/strategy";

export function StrategyDetailPanel({
  detail,
  row,
}: {
  detail: StrategyLabDetail | null;
  row: StrategyLabRow | null;
}) {
  if (!detail || !row) {
    return (
      <article className="terminal-panel">
        <p className="eyebrow">Method detail</p>
        <p className="terminal-empty-state">Select a method to review its controls and recent paper-bet history.</p>
      </article>
    );
  }

  return (
    <article className="terminal-panel strategy-lab-detail">
      <p className="eyebrow">Method detail</p>
      <h3>{detail.displayName}</h3>
      <p className="terminal-panel-copy">{detail.summary}</p>

      <div className="simulation-summary-grid">
        <div className="action-keynumber">
          <span className="eyebrow">What it looks for</span>
          <strong>{detail.whatItLooksFor}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Current weight</span>
          <strong>{detail.currentWeightLabel ?? "Not enough data yet"}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Recent performance</span>
          <strong>{detail.recentPerformanceLabel ?? "Not enough data yet"}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Current confidence</span>
          <strong>{detail.currentConfidenceLabel ?? "Not enough data yet"}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Best regime</span>
          <strong>{detail.bestRegime ?? "Unavailable"}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Worst regime</span>
          <strong>{detail.worstRegime ?? "Unavailable"}</strong>
        </div>
      </div>

      <section className="terminal-panel strategy-lab-subpanel">
        <p className="eyebrow">Last 10 paper bets</p>
        <div className="strategy-lab-paper-bets">
          {detail.paperBets.length > 0 ? (
            detail.paperBets.map((bet) => (
              <div className="terminal-list-row" key={`${detail.strategyId}-${bet.itemName}-${bet.outcomeLabel}`}>
                <div>
                  <strong>{bet.itemName}</strong>
                  <p>{bet.actionLabel} • {bet.outcomeLabel}</p>
                </div>
                <div>
                  <strong>{bet.netGp === null ? "No trade" : `${bet.netGp} gp`}</strong>
                  <p>{bet.confidence === null ? "No confidence logged" : `${Math.round(bet.confidence * 100)}% confidence`}</p>
                </div>
              </div>
            ))
          ) : (
            <p className="terminal-empty-state">No recent paper bets are available for this method.</p>
          )}
        </div>
      </section>

      <section className="terminal-panel strategy-lab-subpanel">
        <p className="eyebrow">Current configuration</p>
        <ul className="invalidation-rules">
          {detail.configSummary.map((line) => (
            <li key={line}>{line}</li>
          ))}
        </ul>
      </section>

      <ExpandableAdvancedPanel title="Advanced method detail">
        <dl className="advanced-definition-list">
          <dt>Strategy id</dt>
          <dd>{detail.advanced.strategyId}</dd>
          <dt>Model version</dt>
          <dd>{detail.advanced.modelVersion ?? "Unavailable"}</dd>
          <dt>Status note</dt>
          <dd>{detail.advanced.statusNote ?? "None"}</dd>
          <dt>Config JSON</dt>
          <dd>
            <pre className="strategy-lab-config-json">{detail.advanced.configJson}</pre>
          </dd>
        </dl>
      </ExpandableAdvancedPanel>
    </article>
  );
}
