import type { EvidenceStage } from "../../domain/evidence";

export function EvidenceTimeline({ stages }: { stages: EvidenceStage[] }) {
  return (
    <ol className="evidence-timeline" aria-label="Evidence stages">
      {stages.map((stage) => (
        <li className={`evidence-stage evidence-stage-${stage.status}`} key={stage.kind}>
          <div>
            <strong>{stage.label}</strong>
            <p>{stage.status.replaceAll("_", " ")}</p>
          </div>
          <span>{stage.timestamp ? new Date(stage.timestamp).toLocaleString("en-GB") : "Not available"}</span>
        </li>
      ))}
    </ol>
  );
}
