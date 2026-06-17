import type { GlossaryTermId } from "./glossary";

export const defaultUiReplacements = {
  liquidityProxy: "How easy it may be to trade",
  executionConfidence: "Trade realism",
  confidenceInterval: "Likely price range",
  drawdown: "Worst temporary drop",
  regime: "Market mood",
  momentum: "Price is moving strongly",
  meanReversion: "Price may return to normal",
  calibration: "Whether confidence has been honest",
  graphRelationship: "Linked item",
  blastRadius: "What else may be affected",
  model: "Method",
  feature: "Input signal",
  prediction: "Price view",
  recommendation: "Suggested action",
} as const;

export const glossaryLabelByTerm: Record<GlossaryTermId, string> = {
  confidence: "Confidence",
  expectedProfit: "Expected profit",
  profitAfterTax: "Profit after tax",
  spread: "Spread",
  buyLimit: "Buy limit",
  suggestedQuantity: "Suggested quantity",
  executionConfidence: "Trade realism",
  predictionConfidence: "Price-view confidence",
  dataQuality: "Data quality",
  modelAccuracy: "Past accuracy",
  simulation: "Simulation",
  linkedItem: "Linked item",
  catchUp: "Catch-up move",
  blastRadius: "What else may be affected",
  forecastRange: "Likely price range",
  dangerPoint: "Danger point",
  volatility: "Price choppiness",
  momentum: "Strong move",
  meanReversion: "Return to normal",
  calibration: "Confidence honesty",
  liquidity: "Trading ease",
  observedVolume: "Observed activity",
  paperTrade: "Practice trade",
};

export function replaceDefaultUiCopy(copy: string): string {
  return Object.entries(defaultUiReplacements).reduce((nextCopy, [technical, plain]) => {
    const pattern = new RegExp(`\\b${technical}\\b`, "gi");
    return nextCopy.replace(pattern, plain);
  }, copy);
}
