import { useMemo, useState } from "react";

import type { IntervalPrice, Recommendation, SimulationRun } from "../../api/types";
import { AccuracyStrip } from "../../charts/AccuracyStrip";
import { ChartFrame, ChartUnavailable } from "../../charts/ChartFrame";
import { DrawdownGraph } from "../../charts/DrawdownGraph";
import { SimulationReplayGraph } from "../../charts/SimulationReplayGraph";
import { chartFixtureDrawdown } from "../../charts/chartFixtures";
import { recommendationMarkersFromVote, timePointsToPricePoints, intervalPricesToTimePoints } from "../../charts/scales";
import { simulationModeAdvancedLabels } from "../../domain/simulation";
import { ActionPageHeader } from "../../views/ActionPageHeader";
import { PaperBetTimeline } from "./PaperBetTimeline";
import { SimulatedBetDetail } from "./SimulatedBetDetail";
import { SimulationSummary } from "./SimulationSummary";
import { WinLossDistribution } from "./WinLossDistribution";
import { simulationReplayFixtures } from "./simulationFixtures";

function formatGp(value: number | null | undefined) {
  if (value === null || value === undefined) {
    return "Unavailable";
  }

  return `${value} gp`;
}

function formatPercent(value: number | null | undefined) {
  if (value === null || value === undefined) {
    return "Unavailable";
  }

  return `${Math.round(value * 100)}%`;
}

function detailActions(labels: string[]) {
  return (
    <>
      {labels.map((label) => (
        <button className="terminal-action-button" key={label} type="button">
          {label}
        </button>
      ))}
    </>
  );
}

function detailNumbers(entries: Array<{ label: string; value: string }>) {
  return (
    <>
      {entries.map((entry) => (
        <div className="action-keynumber" key={entry.label}>
          <span className="eyebrow">{entry.label}</span>
          <strong>{entry.value}</strong>
        </div>
      ))}
    </>
  );
}

function modeKeyForRunCount(simulations: SimulationRun[], history: IntervalPrice[]): keyof typeof simulationReplayFixtures {
  if (simulations.length === 0) {
    return "open";
  }
  if (history.length === 0) {
    return "insufficient_history";
  }
  if (simulations[0]?.status === "skipped") {
    return "skipped";
  }
  if (simulations[0]?.status === "failed") {
    return "realized_loss";
  }

  return "realized_win";
}

export function SimulationReplayView({
  history,
  recommendation,
  simulations,
}: {
  history: IntervalPrice[];
  recommendation: Recommendation | null;
  simulations: SimulationRun[];
}) {
  const points = useMemo(() => timePointsToPricePoints(intervalPricesToTimePoints(history)), [history]);
  const fixture = simulationReplayFixtures[modeKeyForRunCount(simulations, history)];
  const [selectedBetId, setSelectedBetId] = useState<string | null>(fixture.bets[0]?.betId ?? null);
  const selectedBet = fixture.bets.find((bet) => bet.betId === selectedBetId) ?? fixture.bets[0] ?? null;

  return (
    <section className="detailed-view-stack">
      <ActionPageHeader
        action={recommendation ? "WATCH CLOSELY" : "WATCH CLOSELY"}
        confidence={recommendation?.recommendationConfidence ?? null}
        itemName={recommendation?.itemName}
        why={"Did this work before? This replay view keeps the market path beside simulation outcomes."}
        keyNumbers={detailNumbers([
          { label: "Replay runs", value: `${fixture.bets.length}` },
          { label: "Expected profit", value: formatGp(recommendation?.expectedNetGp) },
          { label: "Past accuracy", value: formatPercent(recommendation?.accuracy?.directionalAccuracy) },
        ])}
        actions={detailActions(["Run new replay", "Compare exits", "Inspect fills"])}
      />

      <SimulationSummary fixture={fixture} />

      <div className="terminal-grid">
        <div className="detailed-view-stack">
          <PaperBetTimeline bets={fixture.bets} />
          <WinLossDistribution bets={fixture.bets} />
          <ChartFrame caption="The underlying item path stays visible beside replay outcomes." title="Current price path">
            {points.length > 0 ? (
              <SimulationReplayGraph
                markers={recommendationMarkersFromVote(recommendation?.strategyVotes[0])}
                points={points}
                replayLabels={fixture.bets.map((bet) => `${bet.modeLabel} · ${bet.hitReason}`)}
              />
            ) : (
              <ChartUnavailable message="Not enough shared price history to draw the overlay safely." />
            )}
          </ChartFrame>
          <ChartFrame caption="Worst temporary drop stays visible even when a replay later recovers." title="Worst temporary drop">
            <DrawdownGraph points={chartFixtureDrawdown} />
          </ChartFrame>
          <ChartFrame caption="Confidence at entry remains visible beside replay outcomes." title="Recent hit rate">
            <AccuracyStrip
              accuracy={recommendation?.accuracy?.directionalAccuracy}
              pending={fixture.bets.some((bet) => bet.hitReason === "open")}
              skipped={fixture.bets.some((bet) => bet.hitReason === "skipped")}
            />
          </ChartFrame>
        </div>

        <div className="detailed-view-stack">
          <SimulatedBetDetail bet={selectedBet} />
          <article className="terminal-panel">
            <p className="eyebrow">Mode labels</p>
            <div className="simulation-mode-grid">
              {Object.entries(simulationModeAdvancedLabels).map(([plainLabel]) => (
                <button
                  className={`terminal-action-button ${selectedBet?.modeLabel === plainLabel ? "terminal-nav-button-active" : ""}`}
                  key={plainLabel}
                  type="button"
                  onClick={() => {
                    const matching = fixture.bets.find((bet) => bet.modeLabel === plainLabel) ?? fixture.bets[0] ?? null;
                    setSelectedBetId(matching?.betId ?? null);
                  }}
                >
                  {plainLabel}
                </button>
              ))}
            </div>
            <p className="terminal-panel-copy">
              Default labels stay plain: Safe test, Normal test, and Best-case test.
            </p>
          </article>
        </div>
      </div>
    </section>
  );
}
