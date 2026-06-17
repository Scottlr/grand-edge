import type { PaperBetView, SimulationModeLabel } from "../../domain/simulation";

export type SimulationReplayFixture = {
  verdict: string;
  similarPastCalls: number;
  madeProfit: number;
  averageProfitGp: number | null;
  worstDropBeforeRecovery: number | null;
  bets: PaperBetView[];
};

function buildBet(overrides: Partial<PaperBetView>): PaperBetView {
  return {
    betId: "bet-1",
    strategyId: "spread_edge_v1",
    itemId: 4151,
    itemName: "Abyssal whip",
    entryTime: "2026-06-16T10:00:00Z",
    entryPrice: 100000,
    quantity: 8,
    targetExit: 104000,
    stopLoss: 99000,
    exitTime: "2026-06-16T12:00:00Z",
    exitPrice: 103600,
    taxPaid: 1040,
    expectedNetGp: 1400,
    realizedProfitGp: 1120,
    realizedRoi: 0.014,
    maxDrawdown: 0.06,
    hitReason: "target_exit",
    confidenceAtEntry: 0.78,
    modeLabel: "Normal test",
    slippageEstimateGp: 120,
    ...overrides,
  };
}

export const simulationReplayFixtures: Record<
  "realized_win" | "realized_loss" | "open" | "skipped" | "insufficient_history",
  SimulationReplayFixture
> = {
  realized_win: {
    verdict: "Similar past calls usually ended profitably after tax.",
    similarPastCalls: 14,
    madeProfit: 10,
    averageProfitGp: 1280,
    worstDropBeforeRecovery: 0.08,
    bets: [
      buildBet({ betId: "bet-win-1", modeLabel: "Safe test" }),
      buildBet({ betId: "bet-win-2", modeLabel: "Normal test", realizedProfitGp: 1680, realizedRoi: 0.019 }),
    ],
  },
  realized_loss: {
    verdict: "Similar past calls often struggled before fees and slippage were applied.",
    similarPastCalls: 9,
    madeProfit: 3,
    averageProfitGp: -640,
    worstDropBeforeRecovery: 0.12,
    bets: [
      buildBet({
        betId: "bet-loss-1",
        modeLabel: "Best-case test",
        exitPrice: 98950,
        realizedProfitGp: -880,
        realizedRoi: -0.011,
        hitReason: "stop_loss",
      }),
    ],
  },
  open: {
    verdict: "A few similar calls are still open, so the verdict is still developing.",
    similarPastCalls: 6,
    madeProfit: 2,
    averageProfitGp: null,
    worstDropBeforeRecovery: 0.1,
    bets: [
      buildBet({
        betId: "bet-open-1",
        exitTime: null,
        exitPrice: null,
        realizedProfitGp: null,
        realizedRoi: null,
        hitReason: "open",
      }),
    ],
  },
  skipped: {
    verdict: "Some similar calls were skipped because price history was too thin to replay safely.",
    similarPastCalls: 4,
    madeProfit: 1,
    averageProfitGp: 300,
    worstDropBeforeRecovery: null,
    bets: [
      buildBet({
        betId: "bet-skip-1",
        exitTime: null,
        exitPrice: null,
        realizedProfitGp: null,
        realizedRoi: null,
        hitReason: "skipped",
        maxDrawdown: null,
      }),
    ],
  },
  insufficient_history: {
    verdict: "GrandEdge found similar calls, but not enough shared price history to draw the overlay safely.",
    similarPastCalls: 3,
    madeProfit: 1,
    averageProfitGp: 420,
    worstDropBeforeRecovery: null,
    bets: [
      buildBet({
        betId: "bet-short-history-1",
        modeLabel: "Safe test",
      }),
    ],
  },
};

export function simulationModeLabels(): SimulationModeLabel[] {
  return ["Safe test", "Normal test", "Best-case test"];
}
