export function AccuracyStrip({
  accuracy,
  pending = false,
  skipped = false,
}: {
  accuracy: number | null | undefined;
  pending?: boolean;
  skipped?: boolean;
}) {
  const percentage = accuracy === null || accuracy === undefined ? 0 : Math.round(accuracy * 100);
  const statusLabel = skipped ? "Skipped outcome" : pending ? "Pending outcome" : "Hit rate";

  return (
    <div aria-label="Accuracy strip" className="accuracy-strip">
      <span className="accuracy-strip-fill" style={{ width: `${percentage}%` }} />
      <strong>{accuracy === null || accuracy === undefined ? "Unavailable" : `${percentage}% directional accuracy`}</strong>
      <span className="terminal-mono">{statusLabel}</span>
    </div>
  );
}
