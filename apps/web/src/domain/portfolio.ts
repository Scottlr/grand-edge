import type { RiskProfile } from "./auth";
import type { RecommendationDto as Recommendation } from "./recommendation";

export type Position = {
  positionId: string;
  userId: string;
  itemId: number;
  quantity: number;
  avgBuyPrice: number;
  boughtAt: string | null;
  notes: string | null;
};

export type UpsertPositionRequest = {
  itemId: number;
  quantity: number;
  avgBuyPrice: number;
  boughtAt?: string | null;
  notes?: string | null;
};

export type PositionFormValues = {
  itemId: number;
  quantity: number;
  avgBuyPrice: number;
  boughtAt?: string;
  notes?: string;
  riskPreference: "conservative" | "balanced" | "aggressive";
  targetProfitGp?: number;
};

export type HoldingAction =
  | "SELL ALL"
  | "SELL SOME"
  | "HOLD"
  | "DO NOT ADD"
  | "WATCH CLOSELY";

export type HoldingGuidance = {
  action: HoldingAction;
  tone: "sell" | "hold" | "wait" | "avoid";
  headline: string;
  reason: string;
  itemId: number;
  itemName: string;
  quantity: number;
  avgBuyPrice: number;
  currentLow: number | null;
  currentHigh: number | null;
  unrealizedProfitAfterTax: number | null;
  cashoutPrice: number | null;
  stopLoss: number | null;
  confidence: number | null;
  recommendation: Recommendation | null;
  notes: string | null;
};

export type PortfolioSummary = {
  trackedItemCount: number;
  itemsToSell: number;
  itemsToHold: number;
  itemsAtRisk: number;
  estimatedProfitAfterTax: number | null;
};

export function riskPreferenceFromProfile(
  profile: RiskProfile | null | undefined,
): PositionFormValues["riskPreference"] {
  if (!profile) {
    return "balanced";
  }
  if (profile.minConfidence >= 0.7 || profile.maxPortfolioDrawdown <= 0.1) {
    return "conservative";
  }
  if (profile.minConfidence <= 0.45 || profile.maxPortfolioDrawdown >= 0.25) {
    return "aggressive";
  }
  return "balanced";
}

export function toPositionRequest(values: PositionFormValues): UpsertPositionRequest {
  return {
    itemId: values.itemId,
    quantity: values.quantity,
    avgBuyPrice: values.avgBuyPrice,
    boughtAt: values.boughtAt || null,
    notes: values.notes || null,
  };
}
