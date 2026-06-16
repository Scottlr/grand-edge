export const queryKeys = {
  itemsPrefix: () => ["items"] as const,
  items: (params: unknown) => ["items", params] as const,
  item: (id: number) => ["item", id] as const,
  itemHistory: (id: number, params: unknown) => ["itemHistory", id, params] as const,
  recommendationsPrefix: () => ["recommendations"] as const,
  recommendations: (params: unknown) => ["recommendations", params] as const,
  recommendationExplanation: (id: string) => ["recommendationExplanation", id] as const,
  strategies: () => ["strategies"] as const,
  positions: () => ["positions"] as const,
  simulationsPrefix: () => ["simulations"] as const,
  simulations: (params: unknown) => ["simulations", params] as const,
} as const;
