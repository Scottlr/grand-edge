import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import type { IntervalPrice, SimulationRun } from "../../api/types";
import { GlossaryProvider } from "../../components/learn/GlossaryProvider";
import { recommendationMocks } from "../../domain/recommendation";
import { simulationModeAdvancedLabels } from "../../domain/simulation";
import { SimulationReplayView } from "./SimulationReplayView";

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
];

const COMPLETED_RUN: SimulationRun = {
  runId: "sim-1",
  name: "Abyssal whip replay",
  strategyConfig: {},
  startedAt: "2026-06-16T11:00:00Z",
  finishedAt: "2026-06-16T11:05:00Z",
  status: "completed",
};

const FAILED_RUN: SimulationRun = {
  ...COMPLETED_RUN,
  runId: "sim-2",
  status: "failed",
};

function renderWithProviders(node: ReactNode) {
  const queryClient = new QueryClient();
  return renderToStaticMarkup(
    <QueryClientProvider client={queryClient}>
      <GlossaryProvider>{node}</GlossaryProvider>
    </QueryClientProvider>,
  );
}

describe("simulation replay feature", () => {
  it("uses the did this work before heading and plain mode labels", () => {
    const markup = renderWithProviders(
      <SimulationReplayView
        history={HISTORY}
        recommendation={recommendationMocks.live}
        simulations={[COMPLETED_RUN]}
      />,
    );

    expect(markup).toContain("Did this work before?");
    expect(markup).toContain("Safe test");
    expect(markup).toContain("Normal test");
    expect(markup).toContain("Best-case test");
  });

  it("hides advanced execution-mode jargon by default", () => {
    const markup = renderWithProviders(
      <SimulationReplayView
        history={HISTORY}
        recommendation={recommendationMocks.live}
        simulations={[COMPLETED_RUN]}
      />,
    );

    Object.values(simulationModeAdvancedLabels).forEach((label) => {
      expect(markup).not.toContain(label);
    });
  });

  it("shows tax paid and hit reason in the selected bet detail", () => {
    const markup = renderWithProviders(
      <SimulationReplayView
        history={HISTORY}
        recommendation={recommendationMocks.live}
        simulations={[FAILED_RUN]}
      />,
    );

    expect(markup).toContain("Tax paid");
    expect(markup).toContain("1040 gp");
    expect(markup).toContain("Hit reason");
    expect(markup).toContain("stop_loss");
  });

  it("handles insufficient history honestly while keeping bet detail cards", () => {
    const markup = renderWithProviders(
      <SimulationReplayView
        history={[]}
        recommendation={recommendationMocks.live}
        simulations={[COMPLETED_RUN]}
      />,
    );

    expect(markup).toContain("Not enough shared price history to draw the overlay safely.");
    expect(markup).toContain("Selected test detail");
  });

  it("distinguishes open and skipped states in replay outputs", () => {
    const openMarkup = renderWithProviders(
      <SimulationReplayView
        history={[]}
        recommendation={recommendationMocks.live}
        simulations={[]}
      />,
    );
    const skippedMarkup = renderWithProviders(
      <SimulationReplayView
        history={HISTORY}
        recommendation={recommendationMocks.live}
        simulations={[{ ...COMPLETED_RUN, status: "skipped" }]}
      />,
    );

    expect(openMarkup).toContain("Open");
    expect(skippedMarkup).toContain("Skipped");
  });
});
