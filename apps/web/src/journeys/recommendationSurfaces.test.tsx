import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderToStaticMarkup } from "react-dom/server";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

import { GlossaryProvider } from "../components/learn/GlossaryProvider";
import { LearnModal } from "../components/learn/LearnModal";
import {
  allRecommendationSurfacesPassChecklist,
  recommendationSurfaceChecklists,
} from "../content/recommendationSurfaces";
import { recommendationMocks } from "../domain/recommendation";
import { LinkedItemsView } from "../features/linked-items/LinkedItemsView";
import { ModelAccuracyView } from "../features/model-accuracy/ModelAccuracyView";
import { PortfolioView } from "../features/portfolio/PortfolioView";
import { SimulationReplayView } from "../features/simulations/SimulationReplayView";
import {
  ItemIntelligenceView,
} from "../views/DetailedViews";
import { BuyView } from "../views/ActionJourneyViews";

vi.mock("../api/hooks", () => ({
  useCreatePosition: () => ({ mutate: vi.fn() }),
  useUpdatePosition: () => ({ mutate: vi.fn() }),
  useRiskProfile: () => ({ data: null }),
  useRecommendationEvidence: () => ({
    data: null,
    isLoading: false,
    isError: false,
  }),
}));

function render(node: ReactNode) {
  const queryClient = new QueryClient();
  return renderToStaticMarkup(
    <QueryClientProvider client={queryClient}>
      <GlossaryProvider>{node}</GlossaryProvider>
    </QueryClientProvider>,
  );
}

describe("recommendation surfaces", () => {
  it("covers all required surfaces with the shared checklist", () => {
    expect(recommendationSurfaceChecklists.map((entry) => entry.surface)).toEqual([
      "dashboard",
      "buy",
      "sell",
      "portfolio",
      "item",
      "linkedItems",
      "simulations",
      "accuracy",
    ]);
    expect(allRecommendationSurfacesPassChecklist()).toBe(true);
  });

  it("documents skipped surfaces explicitly", () => {
    expect(
      recommendationSurfaceChecklists.filter((entry) => entry.skipReason),
    ).toEqual([]);
  });

  it("beginner journey can open show why confidence learn modal and track item", () => {
    const markup = render(<BuyView recommendation={recommendationMocks.live} />);
    const learnMarkup = render(<LearnModal onOpenChange={() => undefined} open term="confidence" />);

    expect(markup).toContain("Show why");
    expect(markup).toContain("Learn: Confidence");
    expect(markup).toContain("Track item");
    expect(learnMarkup).toContain("Confidence learn panel");
  });

  it("intermediate journey can reach linked items and simulation", () => {
    const itemMarkup = render(
      <ItemIntelligenceView
        history={[]}
        item={null}
        recommendation={recommendationMocks.live}
      />,
    );
    const linkedMarkup = render(
      <LinkedItemsView recommendation={recommendationMocks.live} />,
    );
    const simulationMarkup = render(
      <SimulationReplayView
        history={[]}
        recommendation={recommendationMocks.live}
        simulations={[]}
      />,
    );

    expect(itemMarkup).toContain("Show linked items");
    expect(linkedMarkup).toContain("What happens if this moves?");
    expect(simulationMarkup).toContain("Did this work before?");
  });

  it("advanced journey can open accuracy advanced data", () => {
    const accuracyMarkup = render(
      <ModelAccuracyView
        initialAdvancedOpen
        recommendation={recommendationMocks.live}
      />,
    );

    expect(accuracyMarkup).toContain("Brier score");
    expect(accuracyMarkup).toContain("Model card references");
  });

  it("portfolio surface keeps shared empty-state guidance", () => {
    const markup = render(<PortfolioView positions={[]} recommendations={[]} />);

    expect(markup).toContain("Track your first holding");
    expect(markup).toContain("Add an item, quantity, and buy price to receive cashout guidance.");
  });
});
