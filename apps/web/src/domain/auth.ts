export type ExecutionMode =
  | "conservative_instant"
  | "passive_estimated"
  | "haircut_passive"
  | "worst_case"
  | "user_position_replay";

export type AuthenticatedUser = {
  userId: string;
  email: string;
  displayName: string | null;
};

export type LoginRequest = {
  email: string;
  password: string;
};

export type RegisterRequest = {
  email: string;
  password: string;
  displayName?: string;
};

export type RiskProfile = {
  userId: string;
  maxGpPerItem: number;
  maxPortfolioDrawdown: number;
  minExpectedRoi: number;
  minConfidence: number;
  participationRate: number;
  preferredExecutionMode: ExecutionMode;
  updatedAt: string;
};

export type UpdateRiskProfileRequest = {
  maxGpPerItem: number;
  maxPortfolioDrawdown: number;
  minExpectedRoi: number;
  minConfidence: number;
  participationRate: number;
  preferredExecutionMode: ExecutionMode;
};
