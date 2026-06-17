import { useState, type FormEvent } from "react";

import { useCreatePosition, useUpdatePosition } from "../api/hooks";
import type { IntervalPrice, Item, Position, Recommendation, SimulationRun, UpsertPositionRequest } from "../api/types";
import { RecommendationCard } from "../components/cards/RecommendationCard";
import { glossaryTermsForRecommendation, simpleActionLabel } from "../components/recommendation/recommendationFixtures";
import { AccuracyStrip } from "../charts/AccuracyStrip";
import { BetReplayTrack } from "../charts/BetReplayTrack";
import { ChartFrame, ChartUnavailable } from "../charts/ChartFrame";
import { DrawdownGraph } from "../charts/DrawdownGraph";
import { LiquidityHeatstream } from "../charts/LiquidityHeatstream";
import { ModelVoteStackChart } from "../charts/ModelVoteStackChart";
import { OverlayControls } from "../charts/OverlayControls";
import { PriceEdgeRibbon } from "../charts/PriceEdgeRibbon";
import { RegimeTimeline } from "../charts/RegimeTimeline";
import { SimulationReplayGraph } from "../charts/SimulationReplayGraph";
import { SpreadRiver } from "../charts/SpreadRiver";
import { EvidenceTrailView } from "../features/evidence/EvidenceTrailView";
import { chartFixtureDrawdown } from "../charts/chartFixtures";
import { intervalPricesToTimePoints, recommendationMarkersFromVote, timePointsToPricePoints } from "../charts/scales";
import { ActionPageHeader } from "./ActionPageHeader";

function recommendationRiskLabel(recommendation: Recommendation) {
  return recommendation.riskLabel === "low" ||
    recommendation.riskLabel === "medium" ||
    recommendation.riskLabel === "high"
    ? recommendation.riskLabel
    : "unknown";
}

function formatPercent(value: number | null | undefined) {
  if (value === null || value === undefined) {
    return "Unavailable";
  }

  return `${Math.round(value * 100)}%`;
}

function formatGp(value: number | null | undefined) {
  if (value === null || value === undefined) {
    return "Unavailable";
  }

  return `${value} gp`;
}

function itemIdentity(item: Item | null, recommendation: Recommendation | null) {
  if (item?.icon?.cdnUrl) {
    return <img alt={`${item.name} icon`} className="detailed-item-icon" src={item.icon.cdnUrl} />;
  }
  if (recommendation?.itemIcon?.cdnUrl) {
    return <img alt={`${recommendation.itemName} icon`} className="detailed-item-icon" src={recommendation.itemIcon.cdnUrl} />;
  }

  const label = item?.name ?? recommendation?.itemName ?? "Item";
  return <span className="detailed-item-fallback">{label.slice(0, 2).toUpperCase()}</span>;
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

export function ItemIntelligenceView({
  item,
  recommendation,
  history,
}: {
  item: Item | null;
  recommendation: Recommendation | null;
  history: IntervalPrice[];
}) {
  const points = intervalPricesToTimePoints(history);
  const pricePoints = timePointsToPricePoints(points);
  const itemName = item?.name ?? recommendation?.itemName;

  return (
    <section className="detailed-view-stack">
      <ActionPageHeader
        action={recommendation ? simpleActionLabel(recommendation) : "WAIT"}
        confidence={recommendation?.recommendationConfidence ?? null}
        itemName={itemName}
        why={recommendation?.primaryReason ?? "This item view starts with the current call, then shows the market context behind it."}
        keyNumbers={detailNumbers([
          { label: "Expected profit", value: formatGp(recommendation?.expectedNetGp) },
          { label: "Trade realism", value: formatPercent(recommendation?.executionConfidence) },
          { label: "Past accuracy", value: formatPercent(recommendation?.accuracy?.directionalAccuracy) },
        ])}
        actions={detailActions(["Show linked items", "Run replay", "Review votes"])}
      />

      <article className="terminal-panel detailed-item-header">
        <div className="detailed-item-identity">
          {itemIdentity(item, recommendation)}
          <div>
            <p className="eyebrow">Item intelligence</p>
            <h3>{itemName ?? "No item selected"}</h3>
            <p className="terminal-panel-copy">
              Read price shape, spread, liquidity, and model agreement together before acting.
            </p>
          </div>
        </div>
      </article>

      <div className="terminal-grid">
        <ChartFrame caption="Current price, likely price range, and suggested action levels from stored interval history." title="Current price path">
          {pricePoints.length > 0 ? (
            <>
              <PriceEdgeRibbon markers={recommendation?.strategyVotes[0] ?? null} points={pricePoints} />
              <OverlayControls />
            </>
          ) : (
            <ChartUnavailable message="No interval history yet." />
          )}
        </ChartFrame>
        <ChartFrame caption="Spread tightens and widens over time without faking missing values." title="Spread">
          {pricePoints.length > 0 ? <SpreadRiver points={pricePoints} /> : <ChartUnavailable message="Spread history is unavailable." />}
        </ChartFrame>
        <ChartFrame caption="Higher opacity means more observed activity in the stored snapshots." title="Trading ease">
          {pricePoints.length > 0 ? <LiquidityHeatstream points={pricePoints} /> : <ChartUnavailable message="Volume snapshots are unavailable." />}
        </ChartFrame>
        <ChartFrame caption="Each method view stays visible instead of collapsing into one opaque score." title="Advanced method views">
          {recommendation ? <ModelVoteStackChart votes={recommendation.strategyVotes} /> : <ChartUnavailable message="No strategy votes yet." />}
        </ChartFrame>
        <ChartFrame caption="Data quality, trade realism, and agreement labels set the current market mood." title="Market mood">
          <RegimeTimeline recommendation={recommendation} />
        </ChartFrame>
        <ChartFrame caption="Recent directional accuracy stays visible beside the current recommendation." title="Recent hit rate">
          <AccuracyStrip accuracy={recommendation?.accuracy?.directionalAccuracy} />
        </ChartFrame>
        <ChartFrame caption="The likely price range stays honest about uncertainty instead of pretending one exact target." title="Likely price range">
          <ChartUnavailable message="No likely price range is available yet." />
        </ChartFrame>
      </div>
    </section>
  );
}

export function RecommendationExplainerView({ recommendation }: { recommendation: Recommendation | null }) {
  const watchFirst =
    recommendation?.predictionConfidence !== null &&
    recommendation?.predictionConfidence !== undefined &&
    recommendation?.executionConfidence !== null &&
    recommendation?.executionConfidence !== undefined &&
    recommendation.predictionConfidence - recommendation.executionConfidence >= 0.2;

  const why = watchFirst
    ? "Price likely moves up, but execution quality is uncertain. Recommendation: WATCH, not BUY."
    : recommendation?.primaryReason ?? "This explainer keeps the why, the confidence, and the trade reality in one place.";

  return (
    <section className="detailed-view-stack">
      <ActionPageHeader
        action={watchFirst ? "WATCH CLOSELY" : recommendation ? simpleActionLabel(recommendation) : "WAIT"}
        confidence={recommendation?.recommendationConfidence ?? null}
        itemName={recommendation?.itemName}
        why={why}
        keyNumbers={detailNumbers([
          { label: "Prediction confidence", value: formatPercent(recommendation?.predictionConfidence) },
          { label: "Trade realism", value: formatPercent(recommendation?.executionConfidence) },
          { label: "Recommendation confidence", value: formatPercent(recommendation?.recommendationConfidence) },
        ])}
        actions={detailActions(["Show votes", "Open linked items", "Review invalidation rules"])}
      />

      {recommendation ? (
        <>
          <section className="terminal-panel">
            <p className="eyebrow">Score decomposition</p>
            <div className="detailed-score-grid">
              <div>
                <span className="eyebrow">Expected profit</span>
                <strong>{formatGp(recommendation.expectedNetGp)}</strong>
              </div>
              <div>
                <span className="eyebrow">Expected ROI</span>
                <strong>{formatPercent(recommendation.expectedRoi)}</strong>
              </div>
              <div>
                <span className="eyebrow">Suggested quantity</span>
                <strong>{recommendation.strategyVotes[0]?.maxQuantity ?? "Unavailable"}</strong>
              </div>
              <div>
                <span className="eyebrow">Target exit</span>
                <strong>{formatGp(recommendation.strategyVotes[0]?.targetExit)}</strong>
              </div>
              <div>
                <span className="eyebrow">Danger point</span>
                <strong>{formatGp(recommendation.strategyVotes[0]?.stopLoss)}</strong>
              </div>
              <div>
                <span className="eyebrow">Past accuracy</span>
                <strong>{formatPercent(recommendation.accuracy?.directionalAccuracy)}</strong>
              </div>
            </div>
          </section>
          <RecommendationCard
            action={simpleActionLabel(recommendation)}
            confidence={recommendation.recommendationConfidence}
            confidenceBreakdown={recommendation.confidenceBreakdown}
            dataState={recommendation.dataState}
            expectedNetGp={recommendation.expectedNetGp}
            expectedRoi={recommendation.expectedRoi}
            horizonLabel={`${Math.round(recommendation.horizonSeconds / 3600)}h window`}
            invalidationRules={recommendation.invalidationRules}
            itemName={recommendation.itemName}
            learnTermIds={glossaryTermsForRecommendation(recommendation)}
            modelAgreement={recommendation.modelAgreement}
            primaryReason={recommendation.primaryReason}
            reasons={recommendation.reasons}
            riskLabel={recommendationRiskLabel(recommendation)}
            strategyVotes={recommendation.strategyVotes}
          />
          <EvidenceTrailView compact recommendationId={recommendation.recommendationId} />
        </>
      ) : (
        <article className="terminal-panel">
          <p className="terminal-empty-state">No recommendation explanation is available yet.</p>
        </article>
      )}
    </section>
  );
}

function initialPositionForm(recommendation: Recommendation | null): UpsertPositionRequest {
  return {
    itemId: recommendation?.itemId ?? 0,
    quantity: recommendation?.strategyVotes[0]?.maxQuantity ?? 1,
    avgBuyPrice: recommendation?.strategyVotes[0]?.targetEntry ?? 0,
    boughtAt: recommendation?.asOf ?? "",
    notes: "",
  };
}

export function TerminalPortfolioView({
  positions,
  recommendation,
}: {
  positions: Position[];
  recommendation: Recommendation | null;
}) {
  const [form, setForm] = useState<UpsertPositionRequest>(() => initialPositionForm(recommendation));
  const createPosition = useCreatePosition();
  const updatePosition = useUpdatePosition();

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (positions.length > 0) {
      void updatePosition.mutateAsync({ id: positions[0].positionId, body: form });
      return;
    }
    void createPosition.mutateAsync(form);
  }

  return (
    <section className="detailed-view-stack">
      <ActionPageHeader
        action={recommendation ? simpleActionLabel(recommendation) : "HOLD"}
        confidence={recommendation?.recommendationConfidence ?? null}
        itemName={recommendation?.itemName}
        why={recommendation?.primaryReason ?? "Portfolio view starts with what you hold, what it cost, and what the system thinks now."}
        keyNumbers={detailNumbers([
          { label: "Tracked positions", value: `${positions.length}` },
          { label: "Suggested exit", value: formatGp(recommendation?.strategyVotes[0]?.targetExit) },
          { label: "Trade realism", value: formatPercent(recommendation?.executionConfidence) },
        ])}
        actions={detailActions(["Review exits", "Update position", "Check replay"])}
      />

      <article className="terminal-panel">
        <p className="eyebrow">Position editor</p>
        <form className="position-form" onSubmit={handleSubmit}>
          <label>
            <span>Item id</span>
            <input value={form.itemId} onChange={(event) => setForm((current) => ({ ...current, itemId: Number(event.target.value) || 0 }))} />
          </label>
          <label>
            <span>Quantity</span>
            <input value={form.quantity} onChange={(event) => setForm((current) => ({ ...current, quantity: Number(event.target.value) || 0 }))} />
          </label>
          <label>
            <span>Average buy price</span>
            <input value={form.avgBuyPrice} onChange={(event) => setForm((current) => ({ ...current, avgBuyPrice: Number(event.target.value) || 0 }))} />
          </label>
          <label>
            <span>Bought at</span>
            <input className="position-form-wide" value={form.boughtAt ?? ""} onChange={(event) => setForm((current) => ({ ...current, boughtAt: event.target.value }))} />
          </label>
          <label>
            <span>Notes</span>
            <textarea className="position-form-wide" value={form.notes ?? ""} onChange={(event) => setForm((current) => ({ ...current, notes: event.target.value }))} />
          </label>
          <button className="terminal-action-button" type="submit">
            {positions.length > 0 ? "Update first position" : "Create position"}
          </button>
        </form>
      </article>

      <article className="terminal-panel">
        <p className="eyebrow">Current positions</p>
        {positions.length === 0 ? (
          <p className="terminal-empty-state">No positions are tracked yet.</p>
        ) : (
          <div className="terminal-list">
            {positions.map((position) => (
              <div className="terminal-list-row" key={position.positionId}>
                <div>
                  <strong>{recommendation?.itemId === position.itemId ? recommendation.itemName : `Item ${position.itemId}`}</strong>
                  <p>{position.notes ?? "No notes"}</p>
                </div>
                <div>
                  <strong>{position.quantity} units</strong>
                  <p>{position.avgBuyPrice} gp average buy</p>
                </div>
              </div>
            ))}
          </div>
        )}
      </article>
    </section>
  );
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
  const points = intervalPricesToTimePoints(history);

  return (
    <section className="detailed-view-stack">
      <ActionPageHeader
        action={recommendation ? simpleActionLabel(recommendation) : "WATCH CLOSELY"}
        confidence={recommendation?.recommendationConfidence ?? null}
        itemName={recommendation?.itemName}
        why={"Did this work before? This replay view keeps the market path beside simulation outcomes."}
        keyNumbers={detailNumbers([
          { label: "Replay runs", value: `${simulations.length}` },
          { label: "Expected profit", value: formatGp(recommendation?.expectedNetGp) },
          { label: "Past accuracy", value: formatPercent(recommendation?.accuracy?.directionalAccuracy) },
        ])}
        actions={detailActions(["Run new replay", "Compare exits", "Inspect fills"])}
      />

      <div className="terminal-grid">
        <ChartFrame caption="The underlying item path stays visible beside replay outcomes." title="Current price path">
          {points.length > 0 ? (
            <SimulationReplayGraph
              markers={recommendationMarkersFromVote(recommendation?.strategyVotes[0])}
              points={timePointsToPricePoints(points)}
              replayLabels={simulations.map((run, index) => `Replay ${index + 1}: ${run.status}`)}
            />
          ) : (
            <ChartUnavailable message="No price path is available for this replay yet." />
          )}
        </ChartFrame>
        <ChartFrame caption="Replay outcomes stay readable as distinct runs instead of one hidden average." title="Past test trades">
          <BetReplayTrack recommendation={recommendation} simulations={simulations} />
        </ChartFrame>
        <ChartFrame caption="Worst temporary drop stays visible even when a replay later recovers." title="Worst temporary drop">
          <DrawdownGraph points={chartFixtureDrawdown} />
        </ChartFrame>
      </div>
    </section>
  );
}
