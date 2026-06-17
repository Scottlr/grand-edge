export type ModelHealthBarProps = {
  liveStrategyCount: number;
  recentSimulationAccuracy: number | null;
  bestStrategyId: string | null;
  worstStrategyId: string | null;
  marketRegime: string | null;
};

export function ModelHealthBar({
  bestStrategyId,
  liveStrategyCount,
  marketRegime,
  recentSimulationAccuracy,
  worstStrategyId,
}: ModelHealthBarProps) {
  return (
    <article className="terminal-panel command-center-healthbar">
      <div className="terminal-panel-header-inline">
        <div>
          <p className="eyebrow">Model health</p>
          <h3>Did this work before?</h3>
        </div>
        <p className="terminal-panel-copy">
          Keep method health visible, but plain. Technical labels stay in advanced panels later.
        </p>
      </div>
      <div className="command-center-health-grid">
        <div className="action-keynumber">
          <span className="eyebrow">Live methods</span>
          <strong>{liveStrategyCount}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Recent accuracy</span>
          <strong>{recentSimulationAccuracy === null ? "Unavailable" : `${Math.round(recentSimulationAccuracy * 100)}%`}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Best method</span>
          <strong>{bestStrategyId ?? "Unavailable"}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Weakest method</span>
          <strong>{worstStrategyId ?? "Unavailable"}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Market mood</span>
          <strong>{marketRegime ?? "Unavailable"}</strong>
        </div>
      </div>
    </article>
  );
}
