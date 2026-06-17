import { useId, type ReactNode } from "react";

export function ChartFrame({
  title,
  caption,
  children,
}: {
  title: string;
  caption: string;
  children: ReactNode;
}) {
  const titleId = useId();
  const captionId = useId();

  return (
    <article aria-describedby={captionId} aria-labelledby={titleId} className="terminal-panel chart-frame">
      <div className="terminal-panel-header-inline">
        <div>
          <p className="eyebrow">Chart</p>
          <h3 id={titleId}>{title}</h3>
        </div>
      </div>
      <p className="terminal-panel-copy" id={captionId}>{caption}</p>
      <div className="chart-surface">{children}</div>
    </article>
  );
}

export function ChartUnavailable({ message }: { message: string }) {
  return <p className="terminal-empty-state">{message}</p>;
}
