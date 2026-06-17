import type { Position, Recommendation } from "../../api/types";
import type {
  HoldingAction,
  HoldingGuidance,
  PortfolioSummary,
} from "../../domain/portfolio";

function actionTone(action: HoldingAction): HoldingGuidance["tone"] {
  switch (action) {
    case "SELL ALL":
    case "SELL SOME":
      return "sell";
    case "HOLD":
      return "hold";
    case "DO NOT ADD":
      return "avoid";
    case "WATCH CLOSELY":
    default:
      return "wait";
  }
}

export function recommendationToHoldingAction(
  recommendation: Recommendation | null,
  hasPosition: boolean,
): HoldingAction {
  if (!recommendation) {
    return hasPosition ? "HOLD" : "WATCH CLOSELY";
  }

  switch (recommendation.action) {
    case "cashout":
      return "SELL ALL";
    case "add":
      return hasPosition ? "SELL SOME" : "WATCH CLOSELY";
    case "hold":
      return "HOLD";
    case "avoid":
      return hasPosition ? "DO NOT ADD" : "WATCH CLOSELY";
    case "watch":
      return "WATCH CLOSELY";
    case "buy":
    default:
      return hasPosition ? "HOLD" : "WATCH CLOSELY";
  }
}

function itemNameForPosition(position: Position, recommendation: Recommendation | null): string {
  if (recommendation?.itemId === position.itemId) {
    return recommendation.itemName;
  }
  return `Item ${position.itemId}`;
}

function cashoutPrice(recommendation: Recommendation | null): number | null {
  return recommendation?.strategyVotes[0]?.targetExit ?? null;
}

function currentLow(recommendation: Recommendation | null): number | null {
  return recommendation?.strategyVotes[0]?.targetEntry ?? null;
}

function currentHigh(recommendation: Recommendation | null): number | null {
  return recommendation?.strategyVotes[0]?.targetExit ?? null;
}

function holdingReason(action: HoldingAction, recommendation: Recommendation | null): string {
  if (recommendation?.primaryReason) {
    return recommendation.primaryReason;
  }

  switch (action) {
    case "SELL ALL":
      return "The current view favors taking profit instead of waiting for more upside.";
    case "SELL SOME":
      return "The current view is positive, but scaling back size keeps the trade more realistic.";
    case "DO NOT ADD":
      return "The trade does not look attractive enough after tax and execution checks.";
    case "WATCH CLOSELY":
      return "The item may move, but the trade path is not clear enough yet.";
    case "HOLD":
    default:
      return "Nothing is wrong enough to exit, and nothing is strong enough to add more right now.";
  }
}

export function buildHoldingGuidance(
  positions: Position[],
  recommendations: Recommendation[],
): HoldingGuidance[] {
  return positions.map((position) => {
    const recommendation =
      recommendations.find((entry) => entry.itemId === position.itemId) ?? null;
    const action = recommendationToHoldingAction(recommendation, true);

    return {
      action,
      tone: actionTone(action),
      headline:
        action === "SELL ALL"
          ? "Good cashout point"
          : action === "SELL SOME"
            ? "Trim your size"
            : action === "DO NOT ADD"
              ? "Do not add more"
              : action === "WATCH CLOSELY"
                ? "Watch this holding"
                : "Keep holding",
      reason: holdingReason(action, recommendation),
      itemId: position.itemId,
      itemName: itemNameForPosition(position, recommendation),
      quantity: position.quantity,
      avgBuyPrice: position.avgBuyPrice,
      currentLow: currentLow(recommendation),
      currentHigh: currentHigh(recommendation),
      unrealizedProfitAfterTax: recommendation?.expectedNetGp ?? null,
      cashoutPrice: cashoutPrice(recommendation),
      stopLoss: recommendation?.strategyVotes[0]?.stopLoss ?? null,
      confidence: recommendation?.recommendationConfidence ?? null,
      recommendation,
      notes: position.notes,
    };
  });
}

export function buildPortfolioSummary(guidance: HoldingGuidance[]): PortfolioSummary {
  const itemsToSell = guidance.filter((entry) => entry.action === "SELL ALL").length;
  const itemsToHold = guidance.filter((entry) => entry.action === "HOLD").length;
  const itemsAtRisk = guidance.filter((entry) =>
    entry.action === "WATCH CLOSELY" || entry.action === "DO NOT ADD" || entry.action === "SELL SOME"
  ).length;
  const profitEntries = guidance
    .map((entry) => entry.unrealizedProfitAfterTax)
    .filter((entry): entry is number => entry !== null);

  return {
    trackedItemCount: guidance.length,
    itemsToSell,
    itemsToHold,
    itemsAtRisk,
    estimatedProfitAfterTax:
      profitEntries.length > 0
        ? profitEntries.reduce((sum, value) => sum + value, 0)
        : null,
  };
}

export function firstPortfolioRecommendation(
  recommendations: Recommendation[],
  positions: Position[],
): Recommendation | null {
  const positionIds = new Set(positions.map((entry) => entry.itemId));
  return (
    recommendations.find((entry) => positionIds.has(entry.itemId)) ??
    recommendations.find((entry) => entry.action === "cashout") ??
    null
  );
}
