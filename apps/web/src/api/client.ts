import type {
  CreateSimulationRequest,
  Interval,
  IntervalPrice,
  Item,
  PatchStrategyRequest,
  Position,
  Recommendation,
  RecommendationAction,
  RecommendationExplanation,
  SimulationRun,
  StrategyStatus,
  UpsertPositionRequest,
} from "./types";

export type ApiClientConfig = {
  baseUrl: string;
  fetchImpl?: typeof fetch;
};

type RequestInitLike = RequestInit & {
  query?: Record<string, string | number | undefined | null>;
};

export class ApiClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: typeof fetch;

  constructor(config: ApiClientConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, "");
    this.fetchImpl = config.fetchImpl ?? fetch;
  }

  async getItems(params: { limit: number; offset: number }): Promise<Item[]> {
    return this.request<Item[]>("/api/items", {
      query: params,
    });
  }

  async getItem(id: number): Promise<Item> {
    return this.request<Item>(`/api/items/${id}`);
  }

  async getItemHistory(
    id: number,
    params: { interval: Interval; limit: number; before?: string },
  ): Promise<IntervalPrice[]> {
    return this.request<IntervalPrice[]>(`/api/items/${id}/history`, {
      query: params,
    });
  }

  async getRecommendations(params?: {
    action?: RecommendationAction;
    limit?: number;
    offset?: number;
  }): Promise<Recommendation[]> {
    return this.request<Recommendation[]>("/api/recommendations", {
      query: params,
    });
  }

  async getRecommendationExplanation(id: string): Promise<RecommendationExplanation> {
    return this.request<RecommendationExplanation>(`/api/recommendations/${id}/explanation`);
  }

  async getStrategies(): Promise<StrategyStatus[]> {
    return this.request<StrategyStatus[]>("/api/strategies");
  }

  async patchStrategy(id: string, body: PatchStrategyRequest): Promise<StrategyStatus> {
    return this.request<StrategyStatus>(`/api/strategies/${id}`, {
      method: "PATCH",
      body: JSON.stringify(body),
      headers: {
        "Content-Type": "application/json",
      },
    });
  }

  async getPositions(): Promise<Position[]> {
    return this.request<Position[]>("/api/users/me/positions");
  }

  async createPosition(body: UpsertPositionRequest): Promise<Position> {
    return this.request<Position>("/api/users/me/positions", {
      method: "POST",
      body: JSON.stringify(body),
      headers: {
        "Content-Type": "application/json",
      },
    });
  }

  async patchPosition(id: string, body: UpsertPositionRequest): Promise<Position> {
    return this.request<Position>(`/api/users/me/positions/${id}`, {
      method: "PATCH",
      body: JSON.stringify(body),
      headers: {
        "Content-Type": "application/json",
      },
    });
  }

  async getSimulations(params?: { limit?: number; offset?: number }): Promise<SimulationRun[]> {
    return this.request<SimulationRun[]>("/api/simulations", {
      query: params,
    });
  }

  async createSimulation(body: CreateSimulationRequest): Promise<SimulationRun> {
    return this.request<SimulationRun>("/api/simulations", {
      method: "POST",
      body: JSON.stringify(body),
      headers: {
        "Content-Type": "application/json",
      },
    });
  }

  liveUrl(): string {
    return this.resolveUrl("/api/live/stream");
  }

  private async request<T>(path: string, init?: RequestInitLike): Promise<T> {
    const url = new URL(this.resolveUrl(path));
    if (init?.query) {
      for (const [key, value] of Object.entries(init.query)) {
        if (value === undefined || value === null || value === "") {
          continue;
        }
        url.searchParams.set(key, String(value));
      }
    }

    const response = await this.fetchImpl(url.toString(), init);
    if (!response.ok) {
      throw new Error(`API request failed: ${response.status} ${response.statusText}`);
    }

    return (await response.json()) as T;
  }

  private resolveUrl(path: string): string {
    return this.baseUrl ? `${this.baseUrl}${path}` : path;
  }
}

export function createApiClient(): ApiClient {
  const baseUrl = import.meta.env.VITE_API_BASE_URL ?? "";
  return new ApiClient({ baseUrl });
}
