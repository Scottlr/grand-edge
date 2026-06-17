import type { EventImpactBadge } from "./linkedItemTypes";

function percent(value: number) {
  return `${Math.round(value * 100)}%`;
}

export function EventImpactPanel({ events }: { events: EventImpactBadge[] }) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Event impact</p>
      {events.length === 0 ? (
        <p className="terminal-panel-copy">
          No event-linked context is available for this item yet.
        </p>
      ) : (
        <div className="terminal-list">
          {events.map((event) => (
            <div className="terminal-list-row" key={event.title}>
              <div>
                <strong>{event.title}</strong>
                <p>{event.context}</p>
              </div>
              <div>
                <strong
                  className={`linked-source-badge linked-source-${event.sourceType}`}
                >
                  {event.sourceType}
                </strong>
                <p>{percent(event.confidence)}</p>
              </div>
            </div>
          ))}
        </div>
      )}
    </article>
  );
}
