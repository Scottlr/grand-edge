import type { PortfolioExposure } from "./linkedItemTypes";

export function PortfolioExposurePanel({
  exposures,
}: {
  exposures: PortfolioExposure[];
}) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Portfolio exposure</p>
      {exposures.length === 0 ? (
        <p className="terminal-panel-copy">
          No related portfolio exposure is being tracked yet.
        </p>
      ) : (
        <div className="terminal-list">
          {exposures.map((exposure) => (
            <div className="terminal-list-row" key={exposure.cluster}>
              <div>
                <strong>{exposure.cluster}</strong>
                <p>{exposure.note}</p>
              </div>
              <div>
                <strong>{exposure.atRiskPositions} at risk</strong>
                <p>{exposure.linkedItemCount} linked items</p>
              </div>
            </div>
          ))}
        </div>
      )}
    </article>
  );
}
