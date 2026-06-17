export const emptyStates = {
  noPortfolioItems: {
    title: "Track your first holding",
    message: "Add an item, quantity, and buy price to receive cashout guidance.",
  },
  noBuyRecommendations: {
    title: "No strong buys right now",
    message:
      "GrandEdge is waiting because current opportunities do not look good enough after tax, spread, and trade realism checks.",
  },
  noSellRecommendations: {
    title: "No urgent sells",
    message: "Your tracked items do not currently have a strong sell signal.",
  },
  staleData: {
    title: "Market data is old",
    message:
      "Recommendations are paused because the latest price data is not fresh enough.",
  },
  missingAccuracy: {
    title: "Past accuracy is still filling in",
    message:
      "GrandEdge does not have enough recent history to show a fair accuracy read yet, so confidence honesty stays cautious.",
  },
} as const;

export function emptyStatesTeachNextAction() {
  return (
    emptyStates.noPortfolioItems.message.includes("Add an item") &&
    emptyStates.noBuyRecommendations.message.includes("GrandEdge is waiting") &&
    emptyStates.noSellRecommendations.message.includes("sell signal") &&
    emptyStates.staleData.message.includes("paused") &&
    emptyStates.missingAccuracy.message.includes("enough recent history")
  );
}
