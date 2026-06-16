import { describe, expect, it } from "vitest";

import {
  normalizeRiskLabel,
  recommendationMocks,
  recommendationMocksCoverAllDataStates,
  toRecommendationViewModel,
} from "./recommendation";

describe("recommendation domain", () => {
  it("covers every data state in mock fixtures", () => {
    expect(recommendationMocksCoverAllDataStates()).toBe(true);
  });

  it("maps recommendation dto into a compact view model", () => {
    const viewModel = toRecommendationViewModel(recommendationMocks.live);

    expect(viewModel.itemName).toBe("Abyssal whip");
    expect(viewModel.action).toBe("buy");
    expect(viewModel.strategyVotes).toHaveLength(1);
  });

  it("normalizes unknown risk labels safely", () => {
    expect(normalizeRiskLabel("custom")).toBe("unknown");
    expect(normalizeRiskLabel("low")).toBe("low");
  });
});
