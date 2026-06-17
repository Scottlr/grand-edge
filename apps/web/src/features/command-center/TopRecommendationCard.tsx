import type { Recommendation } from "../../api/types";
import { actionToneForRecommendation, simpleActionLabel } from "../../components/recommendation/recommendationFixtures";

export type DashboardActionCardKind =
  | "bestThingToBuy"
  | "bestThingToSell"
  | "itemsToAvoid"
  | "portfolioAlerts"
  | "whatChanged";

export function TopRecommendationCard({
  kind,
  recommendation,
  onSelectRecommendation,
}: {
  kind: DashboardActionCardKind;
  recommendation: Recommendation | null;
  onSelectRecommendation(recommendationId: string): void;
}) {
  const titleByKind: Record<DashboardActionCardKind, string> = {
    bestThingToBuy: "Best thing to buy",
    bestThingToSell: "Best thing to sell",
    itemsToAvoid: "Items to avoid or wait",
    portfolioAlerts: "Your portfolio alerts",
    whatChanged: "What changed",
  };

  const summaryByKind: Record<DashboardActionCardKind, string> = {
    bestThingToBuy: "Current top action after taxes, spread, and trade realism checks.",
    bestThingToSell: "Current strongest exit or de-risking call.",
    itemsToAvoid: "Items where the safer move is wait, avoid, or watch closely.",
    portfolioAlerts: "Current holdings that need attention first.",
    whatChanged: "The most notable recommendation shift on this refresh.",
  };

  const tone = recommendation ? actionToneForRecommendation(recommendation.action) : "wait";

  return (
    <article className={`command-center-topcard command-center-topcard-${tone}`}>
      <div className="command-center-topcard-head">
        <span className={`terminal-tone terminal-tone-${tone}`}>{titleByKind[kind]}</span>
        <span className="terminal-mono">
          {recommendation ? `${Math.round(recommendation.recommendationConfidence * 100)}% confidence` : "Awaiting data"}
        </span>
      </div>
      <div className="command-center-topcard-body">
        {recommendation?.itemIcon?.cdnUrl ? (
          <img alt={recommendation.itemName} className="command-center-item-icon" src={recommendation.itemIcon.cdnUrl} />
        ) : (
          <span className="command-center-item-fallback" aria-hidden="true">
            {recommendation?.itemName.slice(0, 2).toUpperCase() ?? "GE"}
          </span>
        )}
        <div>
          <h3>{recommendation?.itemName ?? "No matching recommendation"}</h3>
          <p>{recommendation?.primaryReason ?? summaryByKind[kind]}</p>
        </div>
      </div>
      <div className="command-center-topcard-metrics">
        <div>
          <span className="eyebrow">Action</span>
          <strong>{recommendation ? simpleActionLabel(recommendation) : "WAIT"}</strong>
        </div>
        <div>
          <span className="eyebrow">Expected profit</span>
          <strong>{recommendation?.expectedNetGp === null || recommendation?.expectedNetGp === undefined ? "Unavailable" : `${recommendation.expectedNetGp} gp`}</strong>
        </div>
      </div>
      <button
        className="terminal-action-button"
        disabled={!recommendation}
        onClick={() => recommendation && onSelectRecommendation(recommendation.recommendationId)}
        type="button"
      >
        {recommendation ? "Open inspector" : "Waiting for signal"}
      </button>
    </article>
  );
}
