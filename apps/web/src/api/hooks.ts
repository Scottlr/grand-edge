import {
  QueryClient,
  useMutation,
  useQuery,
  useQueryClient,
  type UseQueryResult,
} from "@tanstack/react-query";

import { createApiClient } from "./client";
import { queryKeys } from "./queryKeys";
import type {
  Interval,
  LoginRequest,
  Position,
  Recommendation,
  RecommendationEvidence,
  RegisterRequest,
  SimulationRun,
  StrategyStatus,
  UpdateRiskProfileRequest,
  UpsertPositionRequest,
} from "./types";

const apiClient = createApiClient();

export function useRecommendations(params?: {
  action?: Recommendation["action"];
  limit?: number;
  offset?: number;
}): UseQueryResult<Recommendation[]> {
  return useQuery({
    queryKey: queryKeys.recommendations(params ?? {}),
    queryFn: () => apiClient.getRecommendations(params),
  });
}

export function useCurrentUser() {
  return useQuery({
    queryKey: ["auth", "me"],
    queryFn: () => apiClient.getCurrentUser(),
    retry: false,
  });
}

export function useRegister() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: RegisterRequest) => apiClient.register(body),
    onSuccess: async (user) => {
      queryClient.setQueryData(["auth", "me"], user);
      await queryClient.invalidateQueries({ queryKey: ["riskProfile"] });
    },
  });
}

export function useLogin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: LoginRequest) => apiClient.login(body),
    onSuccess: async (user) => {
      queryClient.setQueryData(["auth", "me"], user);
      await queryClient.invalidateQueries({ queryKey: ["riskProfile"] });
    },
  });
}

export function useLogout() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => apiClient.logout(),
    onSuccess: async () => {
      queryClient.removeQueries({ queryKey: ["auth", "me"] });
      await queryClient.invalidateQueries({ queryKey: ["riskProfile"] });
    },
  });
}

export function useRiskProfile() {
  return useQuery({
    queryKey: ["riskProfile"],
    queryFn: () => apiClient.getRiskProfile(),
    retry: false,
  });
}

export function useUpdateRiskProfile() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: UpdateRiskProfileRequest) => apiClient.patchRiskProfile(body),
    onSuccess: async (profile) => {
      queryClient.setQueryData(["riskProfile"], profile);
    },
  });
}

export function useRecommendationExplanation(
  recommendationId: string | null,
): UseQueryResult<Recommendation> {
  return useQuery({
    queryKey: recommendationId
      ? queryKeys.recommendationExplanation(recommendationId)
      : ["recommendationExplanation", "idle"],
    queryFn: () => apiClient.getRecommendationExplanation(recommendationId ?? ""),
    enabled: recommendationId !== null,
  });
}

export function useRecommendationEvidence(
  recommendationId: string | null,
): UseQueryResult<RecommendationEvidence> {
  return useQuery({
    queryKey: recommendationId
      ? queryKeys.recommendationEvidence(recommendationId)
      : ["recommendationEvidence", "idle"],
    queryFn: () => apiClient.getRecommendationEvidence(recommendationId ?? ""),
    enabled: recommendationId !== null,
  });
}

export function useItems(params: { limit: number; offset: number }) {
  return useQuery({
    queryKey: queryKeys.items(params),
    queryFn: () => apiClient.getItems(params),
  });
}

export function useItem(itemId: number | null) {
  return useQuery({
    queryKey: itemId !== null ? queryKeys.item(itemId) : ["item", "idle"],
    queryFn: () => apiClient.getItem(itemId ?? 0),
    enabled: itemId !== null,
  });
}

export function useItemHistory(
  itemId: number | null,
  params: { interval: Interval; limit: number; before?: string },
) {
  return useQuery({
    queryKey: itemId !== null ? queryKeys.itemHistory(itemId, params) : ["itemHistory", "idle"],
    queryFn: () => apiClient.getItemHistory(itemId ?? 0, params),
    enabled: itemId !== null,
  });
}

export function useStrategies() {
  return useQuery({
    queryKey: queryKeys.strategies(),
    queryFn: () => apiClient.getStrategies(),
  });
}

export function useToggleStrategy() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ strategyId, enabled }: { strategyId: string; enabled: boolean }) =>
      apiClient.patchStrategy(strategyId, { enabled }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.strategies() });
    },
  });
}

export function usePositions() {
  return useQuery({
    queryKey: queryKeys.positions(),
    queryFn: () => apiClient.getPositions(),
  });
}

export function useCreatePosition() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: UpsertPositionRequest) => apiClient.createPosition(body),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.positions() });
    },
  });
}

export function useUpdatePosition() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, body }: { id: string; body: UpsertPositionRequest }) =>
      apiClient.patchPosition(id, body),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.positions() });
    },
  });
}

export function useSimulations(params: { limit: number; offset: number }) {
  return useQuery({
    queryKey: queryKeys.simulations(params),
    queryFn: () => apiClient.getSimulations(params),
  });
}

export function getApiClient() {
  return apiClient;
}

export function primeWorkspaceQueries(queryClient: QueryClient, payload: {
  recommendations?: Recommendation[];
  strategies?: StrategyStatus[];
  positions?: Position[];
  simulations?: SimulationRun[];
}) {
  if (payload.recommendations) {
    queryClient.setQueryData(queryKeys.recommendations({}), payload.recommendations);
  }
  if (payload.strategies) {
    queryClient.setQueryData(queryKeys.strategies(), payload.strategies);
  }
  if (payload.positions) {
    queryClient.setQueryData(queryKeys.positions(), payload.positions);
  }
  if (payload.simulations) {
    queryClient.setQueryData(queryKeys.simulations({ limit: 10, offset: 0 }), payload.simulations);
  }
}
