import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";

import type { RecommendationEvidence } from "../../domain/evidence";
import { recommendationMocks } from "../../domain/recommendation";
import { EvidenceTrailView } from "./EvidenceTrailView";
import { ReasonPerformancePanel } from "./ReasonPerformancePanel";

const evidenceFixture: RecommendationEvidence = {
  recommendationId: recommendationMocks.live.recommendationId,
  itemId: recommendationMocks.live.itemId,
  asOf: recommendationMocks.live.asOf,
  stages: [
    { kind: "market_data", label: "Market data", timestamp: recommendationMocks.live.asOf, status: "present" },
    { kind: "feature_snapshot", label: "Feature snapshot", timestamp: recommendationMocks.live.asOf, status: "present" },
    { kind: "graph_context", label: "Graph context", timestamp: recommendationMocks.live.asOf, status: "present" },
    { kind: "prediction", label: "Prediction", timestamp: recommendationMocks.live.asOf, status: "present" },
    { kind: "recommendation", label: "Recommendation", timestamp: recommendationMocks.live.asOf, status: "present" },
    { kind: "explanation", label: "Explanation", timestamp: recommendationMocks.live.asOf, status: "present" },
    { kind: "outcome_evaluation", label: "Outcome evaluation", timestamp: null, status: "pending" },
  ],
  featureSnapshot: {
    featureSnapshotId: "snapshot-1",
    itemId: recommendationMocks.live.itemId,
    asOf: recommendationMocks.live.asOf,
    featureSetVersion: "features_v1",
    graphVersion: "graph_v1",
    sourceWindowStart: recommendationMocks.live.asOf,
    sourceWindowEnd: recommendationMocks.live.asOf,
    features: { spread_pct: 0.02 },
  },
  predictions: [
    {
      predictionId: "prediction-1",
      featureSnapshotId: "snapshot-1",
      itemId: recommendationMocks.live.itemId,
      asOf: recommendationMocks.live.asOf,
      horizonSecs: 3600,
      modelId: "spread_edge_v1",
      modelVersion: "v1",
      predictedDirection: "up",
      predictedReturn: 0.03,
      confidence: 0.8,
      predictionIntervalLow: null,
      predictionIntervalHigh: null,
      explanation: {},
    },
  ],
  predictionLinks: [
    {
      predictionId: "prediction-1",
      contributionWeight: 1,
      modelId: "spread_edge_v1",
      modelVersion: "v1",
    },
  ],
  recommendation: {
    ...recommendationMocks.live,
    action: "avoid",
    primaryReason: "Positive prediction was invalidated by trade realism and risk checks.",
  },
  graphVersion: "graph_v1",
  graphPaths: [
    {
      sourceItemId: 11840,
      targetItemId: recommendationMocks.live.itemId,
      relationType: "ingredient_of",
      edgeId: "edge-1",
      eventId: null,
      contributionWeight: 0.8,
      explanation: {},
    },
  ],
  graphSources: [
    {
      relationType: "ingredient_of",
      sourceItemId: 11840,
      targetItemId: recommendationMocks.live.itemId,
      contributionWeight: 0.8,
    },
  ],
  explanation: {
    summary: "Positive prediction was invalidated by trade realism and risk checks.",
    reasonAtoms: [
      {
        reasonType: "risk_check",
        reasonKey: "risk:min_confidence",
        label: "Risk threshold",
        direction: "negative",
        weight: 0.7,
        evidence: {},
      },
    ],
    invalidationRules: recommendationMocks.live.invalidationRules,
    graphVersion: "graph_v1",
    graphReasonPathCount: 1,
  },
  outcome: null,
  reasonPerformance: [
    {
      reasonType: "risk_check",
      reasonKey: "risk:min_confidence",
      modelVersion: "v1",
      sampleSize: 12,
      winRate: 0.58,
      avgActualReturn: 0.01,
      avgNetGp: 800,
      calibrationError: 0.09,
    },
  ],
  modelCards: [
    {
      modelId: "spread_edge_v1",
      modelVersion: "v1",
      artifactHash: "artifact://spread_edge_v1/v1",
      featureSchemaHash: "sha256:test",
    },
  ],
  dataState: {
    status: "pending",
    reason: "Outcome horizon has not elapsed yet.",
  },
};

vi.mock("../../api/hooks", () => ({
  useRecommendationEvidence: () => ({
    isLoading: false,
    isError: false,
    data: evidenceFixture,
  }),
}));

describe("evidence trail", () => {
  it("renders stages in order", () => {
    const markup = renderToStaticMarkup(
      <EvidenceTrailView recommendationId={recommendationMocks.live.recommendationId} />,
    );

    expect(markup.indexOf("Market data")).toBeLessThan(markup.indexOf("Feature snapshot"));
    expect(markup.indexOf("Feature snapshot")).toBeLessThan(markup.indexOf("Graph context"));
    expect(markup.indexOf("Graph context")).toBeLessThan(markup.indexOf("Prediction"));
    expect(markup.indexOf("Prediction")).toBeLessThan(markup.indexOf("Recommendation"));
    expect(markup.indexOf("Recommendation")).toBeLessThan(markup.indexOf("Explanation"));
    expect(markup.indexOf("Explanation")).toBeLessThan(markup.indexOf("Outcome evaluation"));
  });

  it("renders graph context when present", () => {
    const markup = renderToStaticMarkup(
      <EvidenceTrailView recommendationId={recommendationMocks.live.recommendationId} />,
    );

    expect(markup).toContain("Graph context");
    expect(markup).toContain("ingredient of");
  });

  it("shows avoid despite positive prediction", () => {
    const markup = renderToStaticMarkup(
      <EvidenceTrailView recommendationId={recommendationMocks.live.recommendationId} />,
    );

    expect(markup).toContain("Positive prediction was invalidated by trade realism and risk checks.");
    expect(markup).toContain("up");
  });

  it("shows sample size in reason performance", () => {
    const markup = renderToStaticMarkup(
      <ReasonPerformancePanel rows={evidenceFixture.reasonPerformance} />,
    );

    expect(markup).toContain("Sample size 12");
  });
});
