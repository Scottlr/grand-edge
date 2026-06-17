export type ConfidenceState = "weak" | "uncertain" | "usable" | "strong" | "rare";

function normalizeConfidence(confidence: number): number {
  return confidence <= 1 ? confidence * 100 : confidence;
}

export function getConfidenceState(confidence: number): ConfidenceState {
  const normalized = normalizeConfidence(confidence);
  if (normalized < 40) return "weak";
  if (normalized < 55) return "uncertain";
  if (normalized < 70) return "usable";
  if (normalized < 85) return "strong";
  return "rare";
}

export function toConfidencePercent(confidence: number): number {
  return Math.round(normalizeConfidence(confidence));
}
