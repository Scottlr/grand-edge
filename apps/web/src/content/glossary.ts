export type GlossaryTermId =
  | "confidence"
  | "expectedProfit"
  | "profitAfterTax"
  | "spread"
  | "buyLimit"
  | "suggestedQuantity"
  | "executionConfidence"
  | "predictionConfidence"
  | "dataQuality"
  | "modelAccuracy"
  | "simulation"
  | "linkedItem"
  | "catchUp"
  | "blastRadius"
  | "forecastRange"
  | "dangerPoint"
  | "volatility"
  | "momentum"
  | "meanReversion"
  | "calibration"
  | "liquidity"
  | "observedVolume"
  | "paperTrade";

export type GlossaryEntry = {
  id: GlossaryTermId;
  label: string;
  quick: string;
  learnTitle: string;
  learnBody: string[];
  example?: string;
  whyItMatters?: string[];
  advanced?: string[];
};

type GlossaryRecord = Record<GlossaryTermId, GlossaryEntry>;

export const glossaryEntries: GlossaryRecord = {
  confidence: {
    id: "confidence",
    label: "Confidence",
    quick: "How strongly GrandEdge trusts this suggestion right now.",
    learnTitle: "Confidence",
    learnBody: [
      "Confidence is a summary of how convincing the current evidence looks.",
      "It combines the price view, trade realism, recent honesty, and data quality instead of pretending one signal is enough.",
    ],
    example: "A BUY with strong agreement and fresh data can carry higher confidence than a BUY with weaker trade realism.",
    whyItMatters: ["Confidence helps you separate a strong action from a tentative one."],
    advanced: ["Confidence is not guaranteed profit. It is an honesty signal about evidence strength."],
  },
  expectedProfit: {
    id: "expectedProfit",
    label: "Expected profit",
    quick: "The estimated GP gain if the idea works as planned.",
    learnTitle: "Expected profit",
    learnBody: [
      "Expected profit is the estimated upside after the system considers likely entry and exit points.",
      "It is a forecast, not a promise, and it can shrink if trade realism is weak.",
    ],
    example: "A suggestion might show 1,400 GP expected profit before extra uncertainty trims it further.",
    whyItMatters: ["It keeps the recommendation focused on likely value, not just price direction."],
    advanced: ["This value should be read beside confidence, risk, and trade realism."],
  },
  profitAfterTax: {
    id: "profitAfterTax",
    label: "Profit after tax",
    quick: "Estimated GP left after Grand Exchange tax is taken out.",
    learnTitle: "Profit after tax",
    learnBody: [
      "GrandEdge treats tax as a real trading cost.",
      "A price move can look good before tax and still fail the final action once tax is applied.",
    ],
    example: "A small upside can disappear after tax, turning a possible BUY into WAIT.",
    whyItMatters: ["It prevents the UI from celebrating profit that cannot actually be kept."],
    advanced: ["Tax logic stays versioned in shared market rules."],
  },
  spread: {
    id: "spread",
    label: "Spread",
    quick: "The gap between the buy-side and sell-side price.",
    learnTitle: "Spread",
    learnBody: [
      "Spread is the gap between the current high-side and low-side price.",
      "A bigger spread can mean more possible profit, but it can also mean the item is harder to trade.",
    ],
    example: "If an item buys for 1,000 GP and sells for 950 GP, the spread is 50 GP.",
    whyItMatters: ["Wide spreads can create opportunity, but they also raise execution risk."],
    advanced: ["spread_pct = (high - low) / midpoint"],
  },
  buyLimit: {
    id: "buyLimit",
    label: "Buy limit",
    quick: "The game cap on how many of an item you can buy within a window.",
    learnTitle: "Buy limit",
    learnBody: [
      "Some items cannot be bought in unlimited quantities right away.",
      "GrandEdge keeps this in mind so a suggestion does not assume impossible sizing.",
    ],
    example: "A strong BUY might still suggest a smaller quantity if the buy limit is tight.",
    whyItMatters: ["It keeps suggested quantity grounded in actual GE rules."],
    advanced: ["Buy-limit windows are versioned market-rule configuration."],
  },
  suggestedQuantity: {
    id: "suggestedQuantity",
    label: "Suggested quantity",
    quick: "A conservative amount that looks realistic to trade.",
    learnTitle: "Suggested quantity",
    learnBody: [
      "Suggested quantity is the amount GrandEdge thinks is practical, not the largest amount imaginable.",
      "It is shaped by trade realism, rules, and risk rather than raw optimism.",
    ],
    example: "An item may look attractive, but the system can still suggest buying only a few units.",
    whyItMatters: ["It turns a recommendation into something actionable."],
    advanced: ["Sizing may be limited by execution confidence, capacity estimates, and risk rules."],
  },
  executionConfidence: {
    id: "executionConfidence",
    label: "Trade realism",
    quick: "How likely it is that you can actually buy or sell at a useful price.",
    learnTitle: "Trade realism",
    learnBody: [
      "GrandEdge estimates this because the OSRS Wiki API does not show the full Grand Exchange order book.",
      "Low trade realism can turn a possible BUY into WAIT.",
    ],
    example: "An item can have a positive price view but still rate poorly for trade realism if fills look uncertain.",
    whyItMatters: ["It protects users from acting on ideas that may be too hard to execute."],
    advanced: ["Also called execution confidence in advanced panels."],
  },
  predictionConfidence: {
    id: "predictionConfidence",
    label: "Price-view confidence",
    quick: "How strongly the methods agree on the likely price move.",
    learnTitle: "Price-view confidence",
    learnBody: [
      "This is about the price view itself, before trading friction is considered.",
      "A strong price view does not automatically mean the final action should be BUY.",
    ],
    example: "GrandEdge can like the price direction but still wait if trade realism is weak.",
    whyItMatters: ["It separates market view from execution reality."],
    advanced: ["This is intentionally shown separately from recommendation confidence."],
  },
  dataQuality: {
    id: "dataQuality",
    label: "Data quality",
    quick: "Whether the current prices and evidence are fresh enough to trust.",
    learnTitle: "Data quality",
    learnBody: [
      "Fresh data supports stronger advice.",
      "Stale or incomplete data should lower trust or pause recommendations entirely.",
    ],
    example: "If prices stop updating, GrandEdge should say that clearly instead of sounding confident.",
    whyItMatters: ["Trust depends on the system admitting when its inputs are weak."],
    advanced: ["Data state can be live, stale, degraded, empty, or error."],
  },
  modelAccuracy: {
    id: "modelAccuracy",
    label: "Past accuracy",
    quick: "How the method has performed on earlier similar calls.",
    learnTitle: "Past accuracy",
    learnBody: [
      "Past accuracy shows how honest the system has been recently.",
      "It cannot guarantee the next outcome, but it helps calibrate trust.",
    ],
    example: "A method with decent recent results may deserve more trust than one that has been missing often.",
    whyItMatters: ["Users should be able to ask whether this has worked before."],
    advanced: ["Advanced panels may also show calibration, sample size, and drawdown."],
  },
  simulation: {
    id: "simulation",
    label: "Simulation",
    quick: "A paper-trading replay that shows what would have happened without risking GP.",
    learnTitle: "Simulation",
    learnBody: [
      "Simulations replay prior ideas with conservative assumptions.",
      "They help answer whether the system's suggestions held up after tax and execution limits.",
    ],
    example: "A simulation can show whether a prior WATCH would have been safer than an aggressive BUY.",
    whyItMatters: ["It gives the product a memory of what actually happened."],
    advanced: ["Paper-trading fills are intentionally conservative, not perfect-case."],
  },
  linkedItem: {
    id: "linkedItem",
    label: "Linked item",
    quick: "Another item that may move with this one.",
    learnTitle: "Linked item",
    learnBody: [
      "Some items react to related ingredients, outputs, or substitutes.",
      "GrandEdge uses linked items as context, not as automatic proof.",
    ],
    example: "If one ingredient moves sharply, a related crafted item may react later.",
    whyItMatters: ["Linked items help explain market context without forcing graph jargon on the main UI."],
    advanced: ["Advanced panels may describe relation type, direction, confidence, and path."],
  },
  catchUp: {
    id: "catchUp",
    label: "Catch-up move",
    quick: "A move where one item may react after a related item already moved.",
    learnTitle: "Catch-up move",
    learnBody: [
      "Sometimes one item lags behind a related move and may catch up later.",
      "GrandEdge treats this as a possibility, not certainty.",
    ],
    example: "A linked ingredient might move first, with the finished item reacting later.",
    whyItMatters: ["It helps explain delayed opportunities."],
    advanced: ["This can be tied to lead-lag or graph path evidence in advanced views."],
  },
  blastRadius: {
    id: "blastRadius",
    label: "What else may be affected",
    quick: "The nearby items that may react if this one moves.",
    learnTitle: "What else may be affected",
    learnBody: [
      "A large move in one item can ripple into related items.",
      "GrandEdge uses this to warn about knock-on effects rather than pretending each item is isolated.",
    ],
    example: "A sudden ingredient shock may alter several nearby crafting items.",
    whyItMatters: ["It keeps risk and opportunity connected across related items."],
    advanced: ["Advanced panels may show path depth, confidence, and affected neighbors."],
  },
  forecastRange: {
    id: "forecastRange",
    label: "Likely price range",
    quick: "The rough band where the price may land, not one exact number.",
    learnTitle: "Likely price range",
    learnBody: [
      "A healthy forecast shows uncertainty instead of a single perfect target.",
      "When no honest range is available, the UI should say so.",
    ],
    example: "Instead of promising 104k exactly, the system may show a likely range around that level.",
    whyItMatters: ["It stops the product from sounding more precise than it is."],
    advanced: ["Also called a confidence interval in advanced explanations."],
  },
  dangerPoint: {
    id: "dangerPoint",
    label: "Danger point",
    quick: "The level where the idea likely stops being worth following.",
    learnTitle: "Danger point",
    learnBody: [
      "A danger point is where the idea has likely gone wrong or become too risky.",
      "This gives users a concrete reason to step back instead of hoping.",
    ],
    example: "If price falls through the danger point, GrandEdge may prefer WAIT or CASHOUT.",
    whyItMatters: ["It turns risk into a clear action boundary."],
    advanced: ["Advanced views may map this to stop-loss logic."],
  },
  volatility: {
    id: "volatility",
    label: "Price choppiness",
    quick: "How sharply the price has been swinging around.",
    learnTitle: "Price choppiness",
    learnBody: [
      "Higher choppiness means the path can be rough even when the overall idea looks good.",
      "GrandEdge should show this as context, not drama.",
    ],
    example: "Two items can have similar profit estimates but very different choppiness.",
    whyItMatters: ["It helps users understand how rough the ride may be."],
    advanced: ["Advanced panels may still call this volatility."],
  },
  momentum: {
    id: "momentum",
    label: "Strong move",
    quick: "A sign that price has been moving firmly in one direction.",
    learnTitle: "Strong move",
    learnBody: [
      "Strong moves can support follow-through, but they can also cool off.",
      "GrandEdge treats this as one input, not a complete argument.",
    ],
    example: "A rising item with strong move evidence may still fail if trade realism is weak.",
    whyItMatters: ["It explains when recent movement is helping the case."],
    advanced: ["Advanced panels may refer to this as momentum."],
  },
  meanReversion: {
    id: "meanReversion",
    label: "Return to normal",
    quick: "A sign that price may drift back toward its usual range.",
    learnTitle: "Return to normal",
    learnBody: [
      "After an unusual move, some items may settle closer to their recent norm.",
      "GrandEdge uses this carefully because unusual conditions can last longer than expected.",
    ],
    example: "A sudden spike may fade instead of continuing straight up.",
    whyItMatters: ["It helps users understand why a recent move may not last."],
    advanced: ["Advanced panels may still refer to this as mean reversion."],
  },
  calibration: {
    id: "calibration",
    label: "Confidence honesty",
    quick: "Whether the system's confidence has matched reality over time.",
    learnTitle: "Confidence honesty",
    learnBody: [
      "Good confidence honesty means high-confidence calls were usually stronger than low-confidence calls.",
      "This matters because a confident tone is only useful if it has been earned.",
    ],
    example: "If 80% confidence calls rarely work, confidence honesty is poor.",
    whyItMatters: ["It tells users whether the confidence meter deserves trust."],
    advanced: ["Advanced panels may use the term calibration."],
  },
  liquidity: {
    id: "liquidity",
    label: "Trading ease",
    quick: "How easy the item seems to trade in practice.",
    learnTitle: "Trading ease",
    learnBody: [
      "GrandEdge estimates trading ease from observed market activity, not a full order book.",
      "Low trading ease can make a good-looking idea less usable.",
    ],
    example: "A thinly traded item may look profitable but be hard to move quickly.",
    whyItMatters: ["It keeps the product honest about real-world friction."],
    advanced: ["Advanced panels may still discuss liquidity proxies."],
  },
  observedVolume: {
    id: "observedVolume",
    label: "Observed activity",
    quick: "The recent amount of trading activity seen in the available data.",
    learnTitle: "Observed activity",
    learnBody: [
      "Observed activity is a clue about how busy the item has been.",
      "It is helpful, but it is not the same thing as full market depth.",
    ],
    example: "Busy recent trading can support stronger trade realism, but it is still only a proxy.",
    whyItMatters: ["It gives context for whether a suggestion seems practical to trade."],
    advanced: ["Advanced panels may display observed volume and z-scores."],
  },
  paperTrade: {
    id: "paperTrade",
    label: "Practice trade",
    quick: "A simulated trade tracked for learning instead of real GP.",
    learnTitle: "Practice trade",
    learnBody: [
      "Practice trades let GrandEdge evaluate ideas without risking a bank.",
      "They create an audit trail for what happened after a recommendation.",
    ],
    example: "A practice trade can show whether a prior BUY would have survived tax and timing friction.",
    whyItMatters: ["It helps the system learn and explain itself safely."],
    advanced: ["Advanced views may connect practice trades to replay and outcome metrics."],
  },
};

export function getGlossaryEntry(term: GlossaryTermId): GlossaryEntry {
  return glossaryEntries[term];
}

export function glossaryContainsAllRequiredTerms(): boolean {
  const requiredTerms = Object.keys(glossaryEntries) as GlossaryTermId[];

  return requiredTerms.every((term) => {
    const entry = glossaryEntries[term];
    return entry.quick.length > 0 && entry.learnBody.length > 0 && (entry.advanced?.length ?? 0) > 0;
  });
}
