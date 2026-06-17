export type RecommendationSurfaceId =
  | "dashboard"
  | "buy"
  | "sell"
  | "portfolio"
  | "item"
  | "linkedItems"
  | "simulations"
  | "accuracy";

export type RecommendationSurfaceChecklist = {
  surface: RecommendationSurfaceId;
  hasSimpleAction: boolean;
  hasOneSentenceWhy: boolean;
  hasConfidence: boolean;
  hasKeyNumbers: boolean;
  hasShowWhy: boolean;
  hasLearnAffordance: boolean;
  hasAdvancedExpansion: boolean;
  skipReason?: string;
};

export const recommendationSurfaceChecklists: RecommendationSurfaceChecklist[] = [
  {
    surface: "dashboard",
    hasSimpleAction: true,
    hasOneSentenceWhy: true,
    hasConfidence: true,
    hasKeyNumbers: true,
    hasShowWhy: true,
    hasLearnAffordance: true,
    hasAdvancedExpansion: true,
  },
  {
    surface: "buy",
    hasSimpleAction: true,
    hasOneSentenceWhy: true,
    hasConfidence: true,
    hasKeyNumbers: true,
    hasShowWhy: true,
    hasLearnAffordance: true,
    hasAdvancedExpansion: true,
  },
  {
    surface: "sell",
    hasSimpleAction: true,
    hasOneSentenceWhy: true,
    hasConfidence: true,
    hasKeyNumbers: true,
    hasShowWhy: true,
    hasLearnAffordance: true,
    hasAdvancedExpansion: true,
  },
  {
    surface: "portfolio",
    hasSimpleAction: true,
    hasOneSentenceWhy: true,
    hasConfidence: true,
    hasKeyNumbers: true,
    hasShowWhy: true,
    hasLearnAffordance: true,
    hasAdvancedExpansion: true,
  },
  {
    surface: "item",
    hasSimpleAction: true,
    hasOneSentenceWhy: true,
    hasConfidence: true,
    hasKeyNumbers: true,
    hasShowWhy: true,
    hasLearnAffordance: true,
    hasAdvancedExpansion: true,
  },
  {
    surface: "linkedItems",
    hasSimpleAction: true,
    hasOneSentenceWhy: true,
    hasConfidence: true,
    hasKeyNumbers: true,
    hasShowWhy: true,
    hasLearnAffordance: true,
    hasAdvancedExpansion: true,
  },
  {
    surface: "simulations",
    hasSimpleAction: true,
    hasOneSentenceWhy: true,
    hasConfidence: true,
    hasKeyNumbers: true,
    hasShowWhy: true,
    hasLearnAffordance: true,
    hasAdvancedExpansion: true,
  },
  {
    surface: "accuracy",
    hasSimpleAction: true,
    hasOneSentenceWhy: true,
    hasConfidence: true,
    hasKeyNumbers: true,
    hasShowWhy: true,
    hasLearnAffordance: true,
    hasAdvancedExpansion: true,
  },
];

export function allRecommendationSurfacesPassChecklist() {
  return recommendationSurfaceChecklists.every(
    (surface) =>
      surface.hasSimpleAction &&
      surface.hasOneSentenceWhy &&
      surface.hasConfidence &&
      surface.hasKeyNumbers &&
      surface.hasShowWhy &&
      surface.hasLearnAffordance &&
      surface.hasAdvancedExpansion,
  );
}
