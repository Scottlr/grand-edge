export type Position = {
  positionId: string;
  userId: string;
  itemId: number;
  quantity: number;
  avgBuyPrice: number;
  boughtAt: string | null;
  notes: string | null;
};

export type UpsertPositionRequest = {
  itemId: number;
  quantity: number;
  avgBuyPrice: number;
  boughtAt?: string | null;
  notes?: string | null;
};
