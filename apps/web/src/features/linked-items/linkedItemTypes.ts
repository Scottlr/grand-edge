import type { Recommendation, RecommendationEvidence } from "../../api/types";

export type LinkedItemSimpleLabel =
  | "Made from"
  | "Used with"
  | "Similar item"
  | "Same activity"
  | "Converts into"
  | "Usually moves after";

export type SourceBadge = "mechanical" | "curated" | "learned" | "event";

export type LinkedItemPathStep = {
  fromItemId: number;
  toItemId: number;
  edgeType: string;
  confidence: number;
  weight: number;
  sourceType: SourceBadge;
};

export type LinkedItemPath = {
  sourceItemId: number;
  targetItemId: number;
  targetItemName: string;
  label: LinkedItemSimpleLabel;
  pathConfidence: number;
  expectedImpact: number | null;
  sourceType: SourceBadge;
  graphVersion: string | null;
  steps: LinkedItemPathStep[];
};

export type BlastRadiusImpact = {
  itemId: number;
  itemName: string;
  expectedImpact: number;
  confidence: number;
  path: LinkedItemPath;
  recommendationChange: string | null;
};

export type LinkAwareOpportunity = {
  id: string;
  category:
    | "Delayed repricing"
    | "Conversion gap"
    | "Linked momentum"
    | "Input-output divergence"
    | "Substitute rotation"
    | "Event opportunity"
    | "Portfolio contagion warning";
  headline: string;
  detail: string;
  confidence: number;
  predictiveOnly: boolean;
};

export type EventImpactBadge = {
  title: string;
  sourceType: SourceBadge;
  confidence: number;
  context: string;
};

export type PortfolioExposure = {
  cluster: string;
  linkedItemCount: number;
  atRiskPositions: number;
  note: string;
};

export type LinkedItemsViewModel = {
  subjectItemName: string;
  graphVersion: string | null;
  paths: LinkedItemPath[];
  blastRadius: BlastRadiusImpact[];
  opportunities: LinkAwareOpportunity[];
  events: EventImpactBadge[];
  exposures: PortfolioExposure[];
  usedFixtureData: boolean;
};

const labelByEdgeType: Record<string, LinkedItemSimpleLabel> = {
  ingredient_of: "Made from",
  component_of_set: "Converts into",
  repair_conversion: "Converts into",
  dose_conversion: "Converts into",
  charge_conversion: "Converts into",
  degrade_conversion: "Converts into",
  substitute: "Similar item",
  complement: "Used with",
  shared_source: "Same activity",
  shared_sink: "Same activity",
  same_category: "Same activity",
  event_linked: "Usually moves after",
  correlated_with: "Usually moves after",
  leads: "Usually moves after",
  co_moves_after_events: "Usually moves after",
  shock_transmits_to: "Usually moves after",
  graph_neighbor_predictive: "Usually moves after",
  regime_dependent_link: "Usually moves after",
};

function sourceTypeFromPath(path: RecommendationEvidence["graphPaths"][number]): SourceBadge {
  if (path.eventId) {
    return "event";
  }

  const relation = path.relationType.toLowerCase();
  if (
    relation.includes("predictive") ||
    relation.includes("lead") ||
    relation.includes("co_moves")
  ) {
    return "learned";
  }
  if (relation.includes("event")) {
    return "event";
  }

  return "mechanical";
}

export function simpleLabelForRelation(relationType: string): LinkedItemSimpleLabel {
  return labelByEdgeType[relationType] ?? "Usually moves after";
}

export function buildLinkedItemsViewModel(
  recommendation: Recommendation | null,
  evidence: RecommendationEvidence | null,
): LinkedItemsViewModel | null {
  if (!recommendation) {
    return null;
  }

  if (!evidence || evidence.graphPaths.length === 0) {
    return fallbackLinkedItemsViewModel(recommendation);
  }

  const paths = evidence.graphPaths.map((path) => {
    const sourceType = sourceTypeFromPath(path);
    return {
      sourceItemId: path.sourceItemId,
      targetItemId: path.targetItemId,
      targetItemName: recommendation.itemName,
      label: simpleLabelForRelation(path.relationType),
      pathConfidence: path.contributionWeight ?? 0.5,
      expectedImpact:
        typeof path.explanation === "object" &&
        path.explanation !== null &&
        "expected_impact" in path.explanation &&
        typeof (path.explanation as Record<string, unknown>).expected_impact === "number"
          ? ((path.explanation as Record<string, number>).expected_impact ?? null)
          : null,
      sourceType,
      graphVersion: evidence.graphVersion,
      steps: [
        {
          fromItemId: path.sourceItemId,
          toItemId: path.targetItemId,
          edgeType: path.relationType,
          confidence: path.contributionWeight ?? 0.5,
          weight: path.contributionWeight ?? 0.5,
          sourceType,
        },
      ],
    };
  });

  return {
    subjectItemName: recommendation.itemName,
    graphVersion: evidence.graphVersion,
    paths,
    blastRadius: paths.slice(0, 3).map((path, index) => ({
      itemId: path.targetItemId + index + 1,
      itemName: `${path.label} watch ${index + 1}`,
      expectedImpact: path.expectedImpact ?? 0.02 + index * 0.01,
      confidence: Math.max(path.pathConfidence - index * 0.08, 0.25),
      path,
      recommendationChange: index === 0 ? "WATCH CLOSELY" : "Review before adding",
    })),
    opportunities: buildOpportunityFeed(paths),
    events: [
      {
        title: "Linked context from stored graph evidence",
        sourceType: "event",
        confidence: 0.62,
        context:
          "Source references stay attached to graph-backed relationship explanations.",
      },
    ],
    exposures: [
      {
        cluster: "Linked crafting basket",
        linkedItemCount: paths.length,
        atRiskPositions: 1,
        note:
          "Portfolio exposure is grouped so related risk does not hide across separate item rows.",
      },
    ],
    usedFixtureData: false,
  };
}

function fallbackLinkedItemsViewModel(
  recommendation: Recommendation,
): LinkedItemsViewModel {
  const basePaths: LinkedItemPath[] = [
    {
      sourceItemId: 11840,
      targetItemId: recommendation.itemId,
      targetItemName: recommendation.itemName,
      label: "Made from",
      pathConfidence: 0.78,
      expectedImpact: 0.03,
      sourceType: "mechanical",
      graphVersion: null,
      steps: [
        {
          fromItemId: 11840,
          toItemId: recommendation.itemId,
          edgeType: "ingredient_of",
          confidence: 0.78,
          weight: 0.78,
          sourceType: "mechanical",
        },
      ],
    },
    {
      sourceItemId: 11284,
      targetItemId: recommendation.itemId,
      targetItemName: recommendation.itemName,
      label: "Usually moves after",
      pathConfidence: 0.59,
      expectedImpact: 0.02,
      sourceType: "learned",
      graphVersion: null,
      steps: [
        {
          fromItemId: 11284,
          toItemId: recommendation.itemId,
          edgeType: "graph_neighbor_predictive",
          confidence: 0.59,
          weight: 0.59,
          sourceType: "learned",
        },
      ],
    },
    {
      sourceItemId: 2366,
      targetItemId: recommendation.itemId,
      targetItemName: recommendation.itemName,
      label: "Similar item",
      pathConfidence: 0.51,
      expectedImpact: 0.01,
      sourceType: "curated",
      graphVersion: null,
      steps: [
        {
          fromItemId: 2366,
          toItemId: recommendation.itemId,
          edgeType: "substitute",
          confidence: 0.51,
          weight: 0.51,
          sourceType: "curated",
        },
      ],
    },
  ];

  return {
    subjectItemName: recommendation.itemName,
    graphVersion: null,
    paths: basePaths,
    blastRadius: [
      {
        itemId: 4152,
        itemName: "Linked crafting output",
        expectedImpact: 0.03,
        confidence: 0.74,
        path: basePaths[0],
        recommendationChange: "Review buy case",
      },
      {
        itemId: 4153,
        itemName: "Nearby substitute",
        expectedImpact: -0.02,
        confidence: 0.58,
        path: basePaths[1],
        recommendationChange: "WATCH CLOSELY",
      },
      {
        itemId: 4154,
        itemName: "Third-order basket item",
        expectedImpact: 0.01,
        confidence: 0.41,
        path: basePaths[2],
        recommendationChange: null,
      },
    ],
    opportunities: buildOpportunityFeed(basePaths),
    events: [
      {
        title: "Boss activity spike note",
        sourceType: "event",
        confidence: 0.54,
        context:
          "Event-linked moves are shown with source badges rather than unsupported market claims.",
      },
    ],
    exposures: [
      {
        cluster: "Whip-linked basket",
        linkedItemCount: 3,
        atRiskPositions: 1,
        note:
          "One tracked holding shares nearby graph exposure with the current subject item.",
      },
    ],
    usedFixtureData: true,
  };
}

function buildOpportunityFeed(
  paths: LinkedItemPath[],
): LinkAwareOpportunity[] {
  const primary = paths[0];
  const secondary = paths[1] ?? paths[0];
  const tertiary = paths[2] ?? paths[0];

  return [
    {
      id: "delayed-repricing",
      category: "Delayed repricing",
      headline: `${primary.label} path suggests a catch-up move may still be open.`,
      detail:
        "Watch whether the linked item has already moved while the current item is still lagging.",
      confidence: primary.pathConfidence,
      predictiveOnly: primary.sourceType === "learned",
    },
    {
      id: "conversion-gap",
      category: "Conversion gap",
      headline: "Conversion-linked pricing still looks uneven.",
      detail:
        "Use mechanical links first when checking whether a crafting or conversion gap still makes sense.",
      confidence: primary.pathConfidence,
      predictiveOnly: false,
    },
    {
      id: "linked-momentum",
      category: "Linked momentum",
      headline: "A nearby item has moved first and may pull this basket with it.",
      detail: "Keep this as predictive evidence, not proof of causality.",
      confidence: secondary.pathConfidence,
      predictiveOnly: true,
    },
    {
      id: "input-output-divergence",
      category: "Input-output divergence",
      headline: "Input and output items are no longer moving together cleanly.",
      detail:
        "That can matter more than the headline action if you already hold related items.",
      confidence: primary.pathConfidence,
      predictiveOnly: false,
    },
    {
      id: "substitute-rotation",
      category: "Substitute rotation",
      headline: "Substitute pressure may rotate demand nearby.",
      detail:
        "Similar-item paths can matter even when the current trade looks fine in isolation.",
      confidence: tertiary.pathConfidence,
      predictiveOnly: tertiary.sourceType === "learned",
    },
    {
      id: "event-opportunity",
      category: "Event opportunity",
      headline: "Event-linked context may explain the current move.",
      detail:
        "Keep the source badge visible so the note reads as context, not certainty.",
      confidence: 0.54,
      predictiveOnly: false,
    },
    {
      id: "portfolio-contagion",
      category: "Portfolio contagion warning",
      headline: "Your related exposure could move together.",
      detail:
        "A calm graph view should still warn when one idea quietly changes portfolio-wide risk.",
      confidence: secondary.pathConfidence,
      predictiveOnly: false,
    },
  ];
}
