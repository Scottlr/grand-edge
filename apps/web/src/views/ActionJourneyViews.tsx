import type { Position, Recommendation, SimulationRun } from "../api/types";
import { RecommendationCard } from "../components/cards/RecommendationCard";
import {
  glossaryTermsForRecommendation,
  simpleActionLabel,
} from "../components/recommendation/recommendationFixtures";
import { TooltipTerm } from "../components/learn/TooltipTerm";
import { ActionPageHeader } from "./ActionPageHeader";

function recommendationRiskLabel(recommendation: Recommendation) {
  return recommendation.riskLabel === "low" ||
    recommendation.riskLabel === "medium" ||
    recommendation.riskLabel === "high"
    ? recommendation.riskLabel
    : "unknown";
}

function recommendationSummaryWhy(recommendation: Recommendation | null, fallback: string) {
  if (!recommendation) {
    return fallback;
  }

  return recommendation.primaryReason;
}

function renderRecommendationCard(recommendation: Recommendation | null) {
  if (!recommendation) {
    return <p className="terminal-panel-copy">No matching suggestion is available yet.</p>;
  }

  return (
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
  );
}

function headerActions(labels: string[]) {
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

function headerNumbers(entries: Array<{ label: string; value: string }>) {
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

export function DashboardView({
  recommendations,
  positions,
}: {
  recommendations: Recommendation[];
  positions: Position[];
}) {
  const bestBuy = recommendations.find((entry) => entry.action === "buy" || entry.action === "add") ?? null;
  const bestSell = recommendations.find((entry) => entry.action === "cashout") ?? null;
  const itemsToAvoid = recommendations.filter((entry) => entry.action === "avoid" || entry.action === "watch");

  return (
    <section className="action-view-stack">
      <div className="action-hero-grid">
        <ActionPageHeader
          action={bestBuy ? simpleActionLabel(bestBuy) : "WAIT"}
          confidence={bestBuy?.recommendationConfidence ?? null}
          itemName={bestBuy?.itemName}
          why={recommendationSummaryWhy(
            bestBuy,
            "No strong buys right now. GrandEdge is waiting because current opportunities do not look good enough after tax, spread, and trade realism checks.",
          )}
          keyNumbers={headerNumbers([
            { label: "Expected profit", value: bestBuy?.expectedNetGp === null || bestBuy?.expectedNetGp === undefined ? "Unavailable" : `${bestBuy.expectedNetGp} gp` },
            { label: "Timeframe", value: bestBuy ? `${Math.round(bestBuy.horizonSeconds / 3600)}h` : "Waiting" },
          ])}
          actions={headerActions(["Show why", "Track item"])}
        />
        <ActionPageHeader
          action={bestSell ? simpleActionLabel(bestSell) : "WAIT"}
          confidence={bestSell?.recommendationConfidence ?? null}
          itemName={bestSell?.itemName}
          why={recommendationSummaryWhy(bestSell, "No strong sells right now. GrandEdge is waiting for clearer exit pressure.")}
          keyNumbers={headerNumbers([
            { label: "Your profit", value: bestSell?.expectedNetGp === null || bestSell?.expectedNetGp === undefined ? "Unavailable" : `${bestSell.expectedNetGp} gp` },
            { label: "Portfolio alerts", value: positions.length > 0 ? `${positions.length}` : "0" },
          ])}
          actions={headerActions(["Open portfolio", "See exits"])}
        />
      </div>

      <section className="terminal-panel">
        <p className="eyebrow">What changed</p>
        <h3>Items to avoid or watch</h3>
        <p className="terminal-panel-copy">
          The dashboard leads with action-first language before deeper evidence, charts, or model detail.
        </p>
        <div className="dashboard-card-grid">
          {itemsToAvoid.slice(0, 3).map((entry) => (
            <div className="dashboard-mini-card" key={entry.recommendationId}>
              <strong>{entry.itemName}</strong>
              <span>{simpleActionLabel(entry)}</span>
              <p>{entry.primaryReason}</p>
            </div>
          ))}
          {itemsToAvoid.length === 0 ? <p className="terminal-panel-copy">Nothing urgent to avoid right now.</p> : null}
        </div>
      </section>
    </section>
  );
}

export function BuyView({ recommendation }: { recommendation: Recommendation | null }) {
  return (
    <section className="action-view-stack">
      <ActionPageHeader
        action={recommendation ? simpleActionLabel(recommendation) : "WAIT"}
        confidence={recommendation?.recommendationConfidence ?? null}
        itemName={recommendation?.itemName}
        why={recommendationSummaryWhy(
          recommendation,
          "GrandEdge is waiting because trade realism or after-tax upside is not strong enough yet.",
        )}
        keyNumbers={headerNumbers([
          { label: "Expected profit", value: recommendation?.expectedNetGp === null || recommendation?.expectedNetGp === undefined ? "Unavailable" : `${recommendation.expectedNetGp} gp` },
          { label: "Suggested quantity", value: recommendation?.strategyVotes[0]?.maxQuantity?.toString() ?? "Unavailable" },
          { label: "Timeframe", value: recommendation ? `${Math.round(recommendation.horizonSeconds / 3600)}h` : "Waiting" },
        ])}
        actions={headerActions(["Show why", "Track item", "Run simulation"])}
      />
      {renderRecommendationCard(recommendation)}
    </section>
  );
}

export function SellView({ recommendation }: { recommendation: Recommendation | null }) {
  return (
    <section className="action-view-stack">
      <ActionPageHeader
        action={recommendation ? simpleActionLabel(recommendation) : "WAIT"}
        confidence={recommendation?.recommendationConfidence ?? null}
        itemName={recommendation?.itemName}
        why={recommendationSummaryWhy(recommendation, "No strong sell is active. GrandEdge is waiting for a clearer exit signal.")}
        keyNumbers={headerNumbers([
          { label: "Your profit", value: recommendation?.expectedNetGp === null || recommendation?.expectedNetGp === undefined ? "Unavailable" : `${recommendation.expectedNetGp} gp` },
          { label: "Suggested sell price", value: recommendation?.strategyVotes[0]?.targetExit?.toString() ?? "Unavailable" },
          { label: "Confidence", value: recommendation ? `${Math.round(recommendation.recommendationConfidence * 100)}%` : "Unavailable" },
        ])}
        actions={headerActions(["Show why", "See exits"])}
      />
      {renderRecommendationCard(recommendation)}
    </section>
  );
}

export function PortfolioView({ positions, recommendation }: { positions: Position[]; recommendation: Recommendation | null }) {
  return (
    <section className="action-view-stack">
      <ActionPageHeader
        action={recommendation ? simpleActionLabel(recommendation) : "HOLD"}
        confidence={recommendation?.recommendationConfidence ?? null}
        itemName={recommendation?.itemName}
        why={recommendationSummaryWhy(recommendation, "Your portfolio view starts with hold-or-sell guidance before deeper numbers.")}
        keyNumbers={headerNumbers([
          { label: "Tracked items", value: `${positions.length}` },
          { label: "Items at risk", value: `${positions.length > 0 ? 1 : 0}` },
          { label: "Estimated profit after tax", value: recommendation?.expectedNetGp === null || recommendation?.expectedNetGp === undefined ? "Unavailable" : `${recommendation.expectedNetGp} gp` },
        ])}
        actions={headerActions(["Open hold advice", "Review positions"])}
      />
    </section>
  );
}

export function ItemsView({ recommendation }: { recommendation: Recommendation | null }) {
  return (
    <section className="action-view-stack">
      <ActionPageHeader
        action={recommendation ? simpleActionLabel(recommendation) : "WAIT"}
        confidence={recommendation?.recommendationConfidence ?? null}
        itemName={recommendation?.itemName}
        why={recommendationSummaryWhy(recommendation, "Start with the current action, then open the deeper item context if needed.")}
        keyNumbers={headerNumbers([
          { label: "Expected profit", value: recommendation?.expectedNetGp === null || recommendation?.expectedNetGp === undefined ? "Unavailable" : `${recommendation.expectedNetGp} gp` },
          { label: "Trade realism", value: recommendation?.executionConfidence === null || recommendation?.executionConfidence === undefined ? "Unavailable" : `${Math.round(recommendation.executionConfidence * 100)}%` },
        ])}
        actions={headerActions(["Show why", "View linked items"])}
      />
      {renderRecommendationCard(recommendation)}
    </section>
  );
}

export function LinkedItemsView() {
  return (
    <section className="action-view-stack">
      <ActionPageHeader
        action={"WATCH CLOSELY"}
        confidence={null}
        why={"Use linked items to understand what else may move before diving into graph detail."}
        keyNumbers={headerNumbers([
          { label: "What happens if this moves?", value: "Explore links" },
          { label: "Linked items", value: "Context first" },
        ])}
        actions={headerActions(["View linked items"])}
      />
      <article className="terminal-panel">
        <p className="terminal-panel-copy">
          This page keeps the user-facing label <TooltipTerm term="linkedItem">Linked Items</TooltipTerm> in primary navigation.
        </p>
      </article>
    </section>
  );
}

export function SimulationsView({ simulations }: { simulations: SimulationRun[] }) {
  return (
    <section className="action-view-stack">
      <ActionPageHeader
        action={"WATCH CLOSELY"}
        confidence={null}
        why={"Did this work before? Start with practical replay outcomes before technical traces."}
        keyNumbers={headerNumbers([
          { label: "Runs", value: `${simulations.length}` },
          { label: "Page title", value: "Did this work before?" },
        ])}
        actions={headerActions(["Open simulation history"])}
      />
    </section>
  );
}

export function AccuracyView({ recommendation }: { recommendation: Recommendation | null }) {
  return (
    <section className="action-view-stack">
      <ActionPageHeader
        action={recommendation ? simpleActionLabel(recommendation) : "WAIT"}
        confidence={recommendation?.recommendationConfidence ?? null}
        why={"Can I trust this? Start with confidence honesty and past accuracy before model jargon."}
        keyNumbers={headerNumbers([
          { label: "Past accuracy", value: recommendation?.accuracy?.directionalAccuracy === null || recommendation?.accuracy?.directionalAccuracy === undefined ? "Unavailable" : `${Math.round(recommendation.accuracy.directionalAccuracy * 100)}%` },
          { label: "Confidence honesty", value: recommendation?.accuracy ? "Available" : "Unavailable" },
        ])}
        actions={headerActions(["Show why", "Review confidence honesty"])}
      />
    </section>
  );
}

export function SettingsView({ strategies }: { strategies: Array<{ strategyId: string; enabled: boolean }> }) {
  return (
    <section className="action-view-stack">
      <ActionPageHeader
        action={"HOLD"}
        confidence={null}
        why={"Settings stays secondary. It should not compete with Buy, Sell, or Portfolio in the main journey."}
        keyNumbers={headerNumbers([
          { label: "Methods available", value: `${strategies.length}` },
          { label: "Advanced controls", value: "Settings only" },
        ])}
        actions={headerActions(["Open advanced settings"])}
      />
    </section>
  );
}
