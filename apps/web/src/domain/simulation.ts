export type SimulationRun = {
  runId: string;
  name: string;
  strategyConfig: unknown;
  startedAt: string;
  finishedAt: string | null;
  status: string;
};

export type CreateSimulationRequest = {
  name: string;
  strategyConfig?: unknown;
};
