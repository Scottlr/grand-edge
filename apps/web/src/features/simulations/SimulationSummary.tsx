import type { SimulationReplayFixture } from "./simulationFixtures";

export function SimulationSummary({ fixture }: { fixture: SimulationReplayFixture }) {
  return (
    <article className="terminal-panel simulation-summary">
      <p className="eyebrow">Did this work before?</p>
      <div className="simulation-summary-grid">
        <div className="action-keynumber">
          <span className="eyebrow">Similar past calls</span>
          <strong>{fixture.similarPastCalls}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Made profit</span>
          <strong>{fixture.madeProfit}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Average profit</span>
          <strong>{fixture.averageProfitGp === null ? "Unavailable" : `${fixture.averageProfitGp} gp`}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Worst drop before recovery</span>
          <strong>
            {fixture.worstDropBeforeRecovery === null
              ? "Unavailable"
              : `${Math.round(fixture.worstDropBeforeRecovery * 100)}%`}
          </strong>
        </div>
      </div>
      <p className="terminal-panel-copy">{fixture.verdict}</p>
    </article>
  );
}
