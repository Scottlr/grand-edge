import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import type { IntervalPrice, Position, SimulationRun } from "../api/types";
import { GlossaryProvider } from "../components/learn/GlossaryProvider";
import { buildDisagreementFixture } from "../components/recommendation/recommendationFixtures";
import { recommendationMocks } from "../domain/recommendation";
import {
  ItemIntelligenceView,
  RecommendationExplainerView,
  SimulationReplayView,
  TerminalPortfolioView,
} from "./DetailedViews";
import { intervalPricesToTimePoints } from "../charts/scales";
import { technicalChartTermsHiddenByDefault } from "../charts/chartTypes";

const HISTORY: IntervalPrice[] = [
  {
    itemId: 4151,
    bucketStart: "2026-06-16T10:00:00Z",
    interval: "1h",
    avgHighPrice: 100200,
    highPriceVolume: 140,
    avgLowPrice: 99400,
    lowPriceVolume: 122,
  },
  {
    itemId: 4151,
    bucketStart: "2026-06-16T11:00:00Z",
    interval: "1h",
    avgHighPrice: null,
    highPriceVolume: 120,
    avgLowPrice: 99700,
    lowPriceVolume: 116,
  },
];

const POSITION: Position = {
  positionId: "position-1",
  userId: "user-1",
  itemId: 4151,
  quantity: 7,
  avgBuyPrice: 99800,
  boughtAt: "2026-06-15T12:00:00Z",
  notes: "Bankstanding reserve",
};

const SIMULATION: SimulationRun = {
  runId: "sim-1",
  name: "Abyssal whip replay",
  strategyConfig: {},
  startedAt: "2026-06-16T11:00:00Z",
  finishedAt: "2026-06-16T11:05:00Z",
  status: "completed",
};

function renderWithProviders(node: ReactNode) {
  const queryClient = new QueryClient();
  return renderToStaticMarkup(
    <QueryClientProvider client={queryClient}>
      <GlossaryProvider>{node}</GlossaryProvider>
    </QueryClientProvider>,
  );
}

describe("detailed views", () => {
  it("keeps missing high or low values as gaps instead of fake zeroes", () => {
    const points = intervalPricesToTimePoints(HISTORY);

    expect(points[1]?.mid).toBe(99700);
    expect(points[1]?.high).toBeNull();
  });

  it("renders item headers with API-provided item icons", () => {
    const markup = renderWithProviders(
      <ItemIntelligenceView
        history={HISTORY}
        item={{
          itemId: 4151,
          name: "Abyssal whip",
          examine: "A weapon from the abyss.",
          members: true,
          buyLimit: 70,
          lowAlch: null,
          highAlch: null,
          value: 1,
          icon: recommendationMocks.live.itemIcon,
        }}
        recommendation={recommendationMocks.live}
      />,
    );

    expect(markup).toContain("Abyssal_whip.png");
  });

  it("renders explanation with score decomposition and separated confidences", () => {
    const markup = renderWithProviders(<RecommendationExplainerView recommendation={recommendationMocks.live} />);

    expect(markup).toContain("Prediction confidence");
    expect(markup).toContain("Trade realism");
    expect(markup).toContain("Recommendation confidence");
    expect(markup).toContain("Tax-adjusted edge clears threshold.");
  });

  it("uses watch-first language when execution is weaker than the price view", () => {
    const disagreement = buildDisagreementFixture().recommendation;
    const markup = renderWithProviders(<RecommendationExplainerView recommendation={disagreement} />);

    expect(markup).toContain("WATCH CLOSELY");
    expect(markup).toContain("execution quality is uncertain");
  });

  it("renders the portfolio editor and current positions", () => {
    const markup = renderWithProviders(
      <TerminalPortfolioView positions={[POSITION]} recommendation={recommendationMocks.live} />,
    );

    expect(markup).toContain("Position editor");
    expect(markup).toContain("Abyssal whip");
  });

  it("renders simulation replay history when runs exist", () => {
    const markup = renderWithProviders(
      <SimulationReplayView
        history={HISTORY}
        recommendation={recommendationMocks.live}
        simulations={[SIMULATION]}
      />,
    );

    expect(markup).toContain("Replay 1");
    expect(markup).toContain("Past test trades");
  });

  it("keeps technical chart jargon hidden by default in the detailed item view", () => {
    const markup = renderWithProviders(
      <ItemIntelligenceView
        history={HISTORY}
        item={null}
        recommendation={recommendationMocks.live}
      />,
    ).toLowerCase();

    technicalChartTermsHiddenByDefault.forEach((term) => {
      expect(markup).not.toContain(`>${term}<`);
    });
  });
});
