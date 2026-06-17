import { assertDefaultUiCopy } from "./copyRules";

export type AlertKind =
  | "buy"
  | "sell"
  | "wait"
  | "portfolioWarning"
  | "staleData"
  | "linkedItem"
  | "simulation";

export type AlertCopyTemplate = {
  kind: AlertKind;
  title: string;
  body: string;
  whyLabel: string;
  forbiddenTerms?: string[];
};

export const forbiddenTrustCopy = [
  "guaranteed",
  "sure thing",
  "free money",
  "always buy",
  "risk-free",
  "high alpha opportunity",
  "liquidity-adjusted edge degraded",
  "signal invalidation threshold breached",
] as const;

export function findForbiddenTrustCopy(copy: string) {
  const lowered = copy.toLowerCase();
  return forbiddenTrustCopy.filter((term) => lowered.includes(term));
}

export function assertPlainAlertCopy(copy: string) {
  const violations = findForbiddenTrustCopy(copy);
  if (violations.length > 0) {
    throw new Error(
      `Alert copy contains forbidden trust language: ${violations.join(", ")}`,
    );
  }

  assertDefaultUiCopy(copy.replaceAll("GrandEdge", "product"));
}

export const alertTemplates: Record<AlertKind, AlertCopyTemplate> = {
  buy: {
    kind: "buy",
    title: "{itemName} now looks worth buying.",
    body:
      "Price is rising, related items support the move, and expected profit remains positive after tax.",
    whyLabel: "Why:",
  },
  sell: {
    kind: "sell",
    title: "{itemName} has reached a good cashout point.",
    body: "You are in profit and the price signal is weakening.",
    whyLabel: "Why:",
  },
  wait: {
    kind: "wait",
    title: "Wait for a cleaner opening.",
    body:
      "This item may rise, but the trade looks hard to complete at a good price.",
    whyLabel: "Why:",
  },
  portfolioWarning: {
    kind: "portfolioWarning",
    title: "A holding needs a closer look.",
    body:
      "A tracked item is starting to weaken, so GrandEdge wants you to review it before the next move.",
    whyLabel: "Why:",
  },
  staleData: {
    kind: "staleData",
    title: "Market data is old.",
    body:
      "Recommendations are paused because the latest price data is not fresh enough.",
    whyLabel: "Why:",
  },
  linkedItem: {
    kind: "linkedItem",
    title: "A linked item may be opening a new setup.",
    body:
      "A related item moved first, so this item may deserve a second look before the move finishes.",
    whyLabel: "Why:",
  },
  simulation: {
    kind: "simulation",
    title: "A replay just changed the trust picture.",
    body:
      "Recent paper-trade outcomes now look weaker than the live headline, so GrandEdge is staying cautious.",
    whyLabel: "Why:",
  },
};

export function alertTemplatesUsePlainActionLanguage() {
  return Object.values(alertTemplates).every((template) => {
    assertPlainAlertCopy(`${template.title} ${template.body}`);
    return true;
  });
}
