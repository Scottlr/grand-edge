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

export function findForbiddenDefaultUiTerms(copy: string): CopyRuleViolation[] {
  const lowered = copy.toLowerCase();

  return forbiddenDefaultUiTerms.flatMap((term) => {
    const index = lowered.indexOf(term);
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
