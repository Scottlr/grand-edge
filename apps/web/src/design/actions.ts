export type UiActionTone = "buy" | "sell" | "wait" | "hold" | "avoid" | "neutral";

export const actionTones: Record<UiActionTone, { cssVar: string; label: string }> = {
  buy: { cssVar: "var(--ge-action-buy)", label: "BUY" },
  sell: { cssVar: "var(--ge-action-sell)", label: "SELL" },
  wait: { cssVar: "var(--ge-action-wait)", label: "WAIT" },
  hold: { cssVar: "var(--ge-action-hold)", label: "HOLD" },
  avoid: { cssVar: "var(--ge-action-avoid)", label: "AVOID" },
  neutral: { cssVar: "var(--ge-text-secondary)", label: "INFO" },
};
