import type { StrategyVoteDto } from "../domain/strategy";
import { ModelVoteTimeline } from "./ModelVoteTimeline";

export function ModelVoteStackChart({ votes }: { votes: StrategyVoteDto[] }) {
  return <ModelVoteTimeline votes={votes} />;
}
