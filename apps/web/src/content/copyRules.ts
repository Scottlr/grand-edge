export const forbiddenDefaultUiTerms = [
  "alpha",
  "edge",
  "liquidity proxy",
  "regime",
  "calibration",
  "drawdown",
  "execution model",
  "confidence interval",
  "mean reversion",
  "momentum oscillator",
  "feature vector",
  "graph propagation",
] as const;

export type CopyRuleViolation = {
  term: string;
  index: number;
};

function escapeRegex(term: string) {
  return term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function findForbiddenDefaultUiTerms(copy: string): CopyRuleViolation[] {
  return forbiddenDefaultUiTerms.flatMap((term) => {
    const pattern = new RegExp(`\\b${escapeRegex(term).replaceAll("\\ ", "\\s+")}\\b`, "i");
    const match = pattern.exec(copy);
    const index = match?.index ?? -1;
    return index >= 0 ? [{ term, index }] : [];
  });
}

export function assertDefaultUiCopy(copy: string): void {
  const violations = findForbiddenDefaultUiTerms(copy);

  if (violations.length > 0) {
    const terms = violations.map((violation) => violation.term).join(", ");
    throw new Error(`Default UI copy contains forbidden technical terms: ${terms}`);
  }
}
