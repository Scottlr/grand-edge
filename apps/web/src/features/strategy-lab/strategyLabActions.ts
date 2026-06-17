export function createStrategyToggleHandler({
  knownStrategyIds,
  onToggle,
}: {
  knownStrategyIds: string[];
  onToggle: (strategyId: string, enabled: boolean) => Promise<unknown> | unknown;
}) {
  return async (strategyId: string, enabled: boolean) => {
    if (!knownStrategyIds.includes(strategyId)) {
      throw new Error(`Unknown strategy id: ${strategyId}`);
    }

    await onToggle(strategyId, enabled);
  };
}
