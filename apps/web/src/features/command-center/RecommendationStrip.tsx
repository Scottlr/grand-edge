import type { Position, Recommendation } from "../../api/types";
import { TopRecommendationCard, type DashboardActionCardKind } from "./TopRecommendationCard";

function firstByAction(recommendations: Recommendation[], actions: Recommendation["action"][]) {
  return recommendations.find((entry) => actions.includes(entry.action)) ?? null;
}

export function RecommendationStrip({
  positions,
  recommendations,
  onSelectRecommendation,
}: {
  positions: Position[];
  recommendations: Recommendation[];
  onSelectRecommendation(recommendationId: string): void;
}) {
  const portfolioItemIds = new Set(positions.map((position) => position.itemId));
  const cards: Array<{ kind: DashboardActionCardKind; recommendation: Recommendation | null }> = [
    { kind: "bestThingToBuy", recommendation: firstByAction(recommendations, ["buy", "add"]) },
    { kind: "bestThingToSell", recommendation: firstByAction(recommendations, ["cashout"]) },
    { kind: "itemsToAvoid", recommendation: firstByAction(recommendations, ["avoid", "watch"]) },
    {
      kind: "portfolioAlerts",
      recommendation: recommendations.find(
        (entry) => portfolioItemIds.has(entry.itemId) && (entry.action === "cashout" || entry.action === "avoid"),
      ) ?? null,
    },
    { kind: "whatChanged", recommendation: recommendations[0] ?? null },
  ];

  return (
    <section className="command-center-strip" aria-label="dashboard action cards">
      {cards.map((card) => (
        <TopRecommendationCard
          key={card.kind}
          kind={card.kind}
          onSelectRecommendation={onSelectRecommendation}
          recommendation={card.recommendation}
        />
      ))}
    </section>
  );
}
