import { QueryClient } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";

import { applyLiveEventToQueryClient } from "./live";

describe("applyLiveEventToQueryClient", () => {
  it("invalidates recommendation caches on recommendation updates", () => {
    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries").mockResolvedValue(undefined);

    applyLiveEventToQueryClient(queryClient, {
      type: "recommendation_updated",
      recommendation_id: "rec-1",
      item_id: 4151,
      action: "buy",
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["recommendations"] });
  });

  it("invalidates strategy caches on strategy configuration updates", () => {
    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries").mockResolvedValue(undefined);

    applyLiveEventToQueryClient(queryClient, {
      type: "strategy_config_updated",
      strategy_id: "kalman",
      enabled: true,
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["strategies"] });
  });
});
