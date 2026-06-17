export function AccuracyStrip({ accuracy }: { accuracy: number | null | undefined }) {
  const percentage = accuracy === null || accuracy === undefined ? 0 : Math.round(accuracy * 100);

  return (
    <div aria-label="Accuracy strip" className="accuracy-strip">
      <span className="accuracy-strip-fill" style={{ width: `${percentage}%` }} />
      <strong>{accuracy === null || accuracy === undefined ? "Unavailable" : `${percentage}% directional accuracy`}</strong>
    </div>
  );
}
