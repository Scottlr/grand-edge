import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";

import { GlossaryProvider } from "../../components/learn/GlossaryProvider";
import { toStrategyLabViewModel } from "../../domain/strategy";
import { StrategyDetailPanel } from "./StrategyDetailPanel";
import { createStrategyToggleHandler } from "./strategyLabActions";
import { strategyLabFixtureStatuses, strategyLabFixtures } from "./strategyLabFixtures";
import { StrategyLabView } from "./StrategyLabView";
import { StrategyTable } from "./StrategyTable";
import { StrategyToggle } from "./StrategyToggle";

describe("strategy lab", () => {
  it("calls patch mutation through the toggle handler", async () => {
    const onToggle = vi.fn(async () => undefined);
    const handler = createStrategyToggleHandler({
      knownStrategyIds: strategyLabFixtureStatuses.map((entry) => entry.strategyId),
      onToggle,
    });

    await handler("spread_edge_v1", false);

    expect(onToggle).toHaveBeenCalledWith("spread_edge_v1", false);
  });

  it("rejects unknown strategy toggle requests", async () => {
    const handler = createStrategyToggleHandler({
      knownStrategyIds: ["spread_edge_v1"],
      onToggle: async () => undefined,
    });

    await expect(handler("unknown_strategy", true)).rejects.toThrow("Unknown strategy id");
  });

  it("renders degraded status distinctly in the strategy table", () => {
    const rows = toStrategyLabViewModel(strategyLabFixtureStatuses, strategyLabFixtures).rows;
    const markup = renderToStaticMarkup(
      <StrategyTable
        onSelect={() => undefined}
        onToggle={() => undefined}
        pendingStrategyId={null}
        rows={rows}
        selectedStrategyId="momentum_v1"
      />,
    );

    expect(markup).toContain("Degraded");
  });

  it("renders the last 10 paper bets in the detail panel", () => {
    const row = toStrategyLabViewModel(strategyLabFixtureStatuses, strategyLabFixtures).rows[0]!;
    const detail = strategyLabFixtures[row.strategyId]!.detail;
    const markup = renderToStaticMarkup(<StrategyDetailPanel detail={detail} row={row} />);

    expect(markup).toContain("Last 10 paper bets");
    expect(markup).toContain("Abyssal whip");
    expect(markup).toContain("Amylase crystal");
  });

  it("renders null metrics as not enough data yet rather than zero", () => {
    const rows = toStrategyLabViewModel(strategyLabFixtureStatuses, strategyLabFixtures).rows;
    const markup = renderToStaticMarkup(
      <StrategyTable
        onSelect={() => undefined}
        onToggle={() => undefined}
        pendingStrategyId={null}
        rows={rows}
        selectedStrategyId="mean_reversion_v1"
      />,
    );

    expect(markup).toContain("Not enough data yet");
    expect(markup).not.toContain(">0%<");
  });

  it("hides raw model fields until advanced detail is expanded", () => {
    const row = toStrategyLabViewModel(strategyLabFixtureStatuses, strategyLabFixtures).rows[0]!;
    const detail = strategyLabFixtures[row.strategyId]!.detail;
    const markup = renderToStaticMarkup(<StrategyDetailPanel detail={detail} row={row} />);

    expect(markup).toContain("Advanced method detail");
    expect(markup).not.toContain("artifact/v1");
    expect(markup).not.toContain("processNoise");
  });

  it("disables the toggle while a mutation is pending", () => {
    const markup = renderToStaticMarkup(
      <StrategyToggle
        checked
        disabled={false}
        label="Toggle Spread Edge"
        onChange={() => undefined}
        pending
      />,
    );

    expect(markup).toContain("disabled");
    expect(markup).toContain("Saving...");
  });

  it("keeps strategy lab out of beginner-first default primary navigation copy", () => {
    const queryClient = new QueryClient();
    const markup = renderToStaticMarkup(
      <QueryClientProvider client={queryClient}>
        <GlossaryProvider>
          <StrategyLabView
            dataState="live"
            strategies={strategyLabFixtureStatuses}
            toggleStrategy={async () => undefined}
          />
        </GlossaryProvider>
      </QueryClientProvider>,
    );

    expect(markup).toContain("This page is intentionally advanced.");
    expect(markup).not.toContain("Feature Store");
    expect(markup).not.toContain("Graph Engine");
  });
});
