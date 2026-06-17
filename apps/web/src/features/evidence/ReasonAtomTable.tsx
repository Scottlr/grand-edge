import type { ReasonAtom } from "../../domain/evidence";

export function ReasonAtomTable({ reasons }: { reasons: ReasonAtom[] }) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Reason checks</p>
      {reasons.length === 0 ? (
        <p className="terminal-empty-state">No structured reason atoms were stored for this recommendation.</p>
      ) : (
        <div className="terminal-list">
          {reasons.map((reason) => (
            <div className="terminal-list-row" key={reason.reasonKey}>
              <div>
                <strong>{reason.label}</strong>
                <p>{reason.reasonType.replaceAll("_", " ")}</p>
              </div>
              <div>
                <strong>{reason.direction}</strong>
                <p>Weight {Math.round(reason.weight * 100)}%</p>
              </div>
            </div>
          ))}
        </div>
      )}
    </article>
  );
}
