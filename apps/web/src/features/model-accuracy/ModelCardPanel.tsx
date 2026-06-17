import type { ModelCardRef } from "../../domain/evidence";

export function ModelCardPanel({ modelCards }: { modelCards: ModelCardRef[] }) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Model card references</p>
      {modelCards.length === 0 ? (
        <p className="terminal-panel-copy">No artifact-linked model card reference is available for this recommendation yet.</p>
      ) : (
        <div className="terminal-list">
          {modelCards.map((card) => (
            <div className="terminal-list-row" key={`${card.modelId}:${card.modelVersion}`}>
              <div>
                <strong>{card.modelId}</strong>
                <p>{card.modelVersion}</p>
              </div>
              <div>
                <strong>{card.featureSchemaHash ?? "Schema hash unavailable"}</strong>
                <p>{card.artifactHash ?? "Artifact hash unavailable"}</p>
              </div>
            </div>
          ))}
        </div>
      )}
    </article>
  );
}
