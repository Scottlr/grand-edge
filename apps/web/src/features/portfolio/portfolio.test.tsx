import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";

import { recommendationMocks } from "../../domain/recommendation";
import type { Position } from "../../api/types";
import { GlossaryProvider } from "../../components/learn/GlossaryProvider";
import { buildHoldingGuidance } from "./portfolioFixtures";
import { PortfolioView } from "./PortfolioView";

vi.mock("../../api/hooks", () => ({
  useCreatePosition: () => ({ mutate: vi.fn() }),
  useUpdatePosition: () => ({ mutate: vi.fn() }),
  useRiskProfile: () => ({ data: null }),
}));

const POSITION: Position = {
  positionId: "position-1",
  userId: "user-1",
  itemId: 4151,
  quantity: 7,
  avgBuyPrice: 99800,
  boughtAt: "2026-06-15T12:00:00Z",
  notes: "Bankstanding reserve",
};

function render(node: ReactNode) {
  const queryClient = new QueryClient();
  return renderToStaticMarkup(
    <QueryClientProvider client={queryClient}>
      <GlossaryProvider>{node}</GlossaryProvider>
    </QueryClientProvider>,
  );
}

describe("portfolio feature", () => {
  it("renders the first-holding empty state", () => {
    const markup = render(<PortfolioView positions={[]} recommendations={[]} />);

    expect(markup).toContain("Track your first holding");
    expect(markup).toContain("Add an item, quantity, and buy price to receive cashout guidance.");
  });

  it("shows after-tax profit on holding cards", () => {
    const markup = render(
      <PortfolioView positions={[POSITION]} recommendations={[recommendationMocks.live]} />,
    );

    expect(markup).toContain("Profit after tax");
    expect(markup).toContain("1400 gp");
  });

  it("uses beginner-safe action labels for holdings", () => {
    const actions = buildHoldingGuidance([POSITION], [
      { ...recommendationMocks.live, action: "cashout" },
      { ...recommendationMocks.live, recommendationId: "rec-2", action: "avoid" },
      { ...recommendationMocks.live, recommendationId: "rec-3", action: "watch" },
      { ...recommendationMocks.live, recommendationId: "rec-4", action: "hold" },
      { ...recommendationMocks.live, recommendationId: "rec-5", action: "add" },
    ]).map((entry) => entry.action);

    actions.forEach((action) => {
      expect([
        "SELL ALL",
        "SELL SOME",
        "HOLD",
        "DO NOT ADD",
        "WATCH CLOSELY",
      ]).toContain(action);
    });
  });

  it("does not use forbidden rebalance language as a primary action", () => {
    const markup = render(
      <PortfolioView positions={[POSITION]} recommendations={[recommendationMocks.live]} />,
    ).toLowerCase();

    expect(markup).not.toContain("rebalance");
    expect(markup).not.toContain("reduce exposure");
    expect(markup).not.toContain("liquidate");
    expect(markup).not.toContain("de-risk");
  });
});
