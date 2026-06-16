import type { QueryClient } from "@tanstack/react-query";

import { queryKeys } from "./queryKeys";
import type { LiveEvent } from "./types";

type LiveOptions = {
  eventSourceFactory?: (url: string) => EventSource;
  onEvent?: (event: LiveEvent) => void;
  onStatusChange?: (status: "connecting" | "live" | "closed" | "error") => void;
};

export function applyLiveEventToQueryClient(queryClient: QueryClient, event: LiveEvent) {
  switch (event.type) {
    case "price_updated":
      void queryClient.invalidateQueries({ queryKey: queryKeys.itemsPrefix() });
      break;
    case "recommendation_updated":
      void queryClient.invalidateQueries({ queryKey: queryKeys.recommendationsPrefix() });
      break;
    case "simulation_updated":
      void queryClient.invalidateQueries({ queryKey: queryKeys.simulationsPrefix() });
      break;
    case "strategy_config_updated":
      void queryClient.invalidateQueries({ queryKey: queryKeys.strategies() });
      break;
    default:
      break;
  }
}

export function createLiveConnection(
  queryClient: QueryClient,
  url: string,
  options: LiveOptions = {},
) {
  const source = (options.eventSourceFactory ?? ((eventUrl) => new EventSource(eventUrl)))(url);
  options.onStatusChange?.("connecting");

  source.addEventListener("live_event", (message) => {
    const event = JSON.parse((message as MessageEvent<string>).data) as LiveEvent;
    applyLiveEventToQueryClient(queryClient, event);
    options.onEvent?.(event);
    options.onStatusChange?.("live");
  });

  source.addEventListener("error", () => {
    options.onStatusChange?.("error");
  });

  return {
    close() {
      source.close();
      options.onStatusChange?.("closed");
    },
  };
}
