import type { GlossaryTermId } from "../../content/glossary";
import {
  normalizeRiskLabel,
  recommendationMocks,
  type DataState,
  type RecommendationAction,
  type RecommendationDto,
  type RiskLabel,
} from "../../domain/recommendation";

export type SimpleActionLabel =
  | "BUY"
  | "SELL"
  | "WAIT"
  | "HOLD"
  | "SELL ALL"
  | "SELL SOME"
  | "DO NOT ADD"
  | "WATCH CLOSELY";

export function simpleActionLabel(recommendation: RecommendationDto): SimpleActionLabel {
  switch (recommendation.action) {
    case "buy":
      return "BUY";
    case "add":
      return "SELL SOME";
    case "cashout":
      return "SELL";
    case "hold":
      return "HOLD";
    case "avoid":
      return recommendation.executionConfidence !== null && recommendation.executionConfidence > 0.55
        ? "DO NOT ADD"
        : "WAIT";
    case "watch":
    default:
      return "WATCH CLOSELY";
  }
}

export function glossaryTermsForRecommendation(recommendation: RecommendationDto): GlossaryTermId[] {
  const terms = new Set<GlossaryTermId>(["confidence", "expectedProfit", "executionConfidence"]);

  if (recommendation.invalidationRules.length > 0) {
    terms.add("dangerPoint");
  }

  if (recommendation.strategyVotes.some((vote) => vote.execution?.observedVolume !== null)) {
    terms.add("observedVolume");
  }

  if (recommendation.expectedRoi !== null) {
    terms.add("profitAfterTax");
  }

  return [...terms];
}

export type RecommendationCardFixture = {
  recommendation: RecommendationDto;
  actionLabel: SimpleActionLabel;
  learnTermIds: GlossaryTermId[];
  riskLabel: RiskLabel;
  horizonLabel: string;
};

export function buildRecommendationCardFixture(dataState: DataState): RecommendationCardFixture {
  const recommendation = recommendationMocks[dataState];

  return {
    recommendation,
    actionLabel: simpleActionLabel(recommendation),
    learnTermIds: glossaryTermsForRecommendation(recommendation),
    riskLabel: normalizeRiskLabel(recommendation.riskLabel),
    horizonLabel: `${Math.round(recommendation.horizonSeconds / 3600)}h window`,
  };
}

export function buildDisagreementFixture(): RecommendationCardFixture {
  const recommendation: RecommendationDto = {
    ...recommendationMocks.live,
    recommendationId: "rec-disagreement",
    action: "watch",
    primaryReason: "The price view looks positive, but trade realism is weaker than the price signal.",
    reasons: [
      "Price view looks positive.",
      "Trade realism is weaker than the price signal.",
      "GrandEdge prefers watch-first language when the path looks harder to trade.",
    ],
    confidenceBreakdown: {
      ...recommendationMocks.live.confidenceBreakdown,
      recommendationConfidence: 0.58,
      predictionConfidence: 0.82,
      executionConfidence: 0.39,
      modelAgreementLabel: "mixed agreement",
      executionQualityLabel: "weak",
    },
    predictionConfidence: 0.82,
    executionConfidence: 0.39,
    recommendationConfidence: 0.58,
    strategyVotes: [
      recommendationMocks.live.strategyVotes[0],
      {
        ...recommendationMocks.live.strategyVotes[0],
        strategyId: "mean_reversion_v1",
        side: "watch",
        confidence: 0.52,
      },
    ],
  };

  return {
    recommendation,
    actionLabel: simpleActionLabel(recommendation),
    learnTermIds: ["confidence", "executionConfidence", "modelAccuracy", "linkedItem"],
    riskLabel: normalizeRiskLabel(recommendation.riskLabel),
    horizonLabel: "1h window",
  };
}

export function actionToneForRecommendation(action: RecommendationAction): "buy" | "sell" | "wait" | "hold" | "avoid" {
  switch (action) {
    case "buy":
    case "add":
      return "buy";
    case "cashout":
      return "sell";
    case "hold":
      return "hold";
    case "avoid":
      return "avoid";
    case "watch":
    default:
      return "wait";
  }
}
