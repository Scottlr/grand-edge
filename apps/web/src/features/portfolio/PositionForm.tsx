import type { FormEvent } from "react";

import type { PositionFormValues } from "../../domain/portfolio";

export function PositionForm({
  onSubmit,
  values,
  onChange,
  submitLabel,
}: {
  values: PositionFormValues;
  submitLabel: string;
  onSubmit(event: FormEvent<HTMLFormElement>): void;
  onChange(next: PositionFormValues): void;
}) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Track a holding</p>
      <h3>Tell GrandEdge what you already own</h3>
      <p className="terminal-panel-copy">
        Add an item, quantity, and buy price to receive cashout guidance.
      </p>

      <form className="position-form" onSubmit={onSubmit}>
        <label>
          <span>Item id</span>
          <input
            value={values.itemId}
            onChange={(event) => onChange({ ...values, itemId: Number(event.target.value) || 0 })}
          />
        </label>
        <label>
          <span>Quantity</span>
          <input
            value={values.quantity}
            onChange={(event) => onChange({ ...values, quantity: Number(event.target.value) || 0 })}
          />
        </label>
        <label>
          <span>Average buy price</span>
          <input
            value={values.avgBuyPrice}
            onChange={(event) =>
              onChange({ ...values, avgBuyPrice: Number(event.target.value) || 0 })
            }
          />
        </label>
        <label>
          <span>Target profit (optional GP)</span>
          <input
            value={values.targetProfitGp ?? ""}
            onChange={(event) =>
              onChange({
                ...values,
                targetProfitGp: event.target.value ? Number(event.target.value) : undefined,
              })
            }
          />
        </label>
        <label>
          <span>Risk preference</span>
          <select
            value={values.riskPreference}
            onChange={(event) =>
              onChange({
                ...values,
                riskPreference: event.target.value as PositionFormValues["riskPreference"],
              })
            }
          >
            <option value="conservative">Conservative</option>
            <option value="balanced">Balanced</option>
            <option value="aggressive">Aggressive</option>
          </select>
        </label>
        <label>
          <span>Bought at</span>
          <input
            value={values.boughtAt ?? ""}
            onChange={(event) => onChange({ ...values, boughtAt: event.target.value })}
          />
        </label>
        <label className="position-form-wide">
          <span>Notes</span>
          <textarea
            value={values.notes ?? ""}
            onChange={(event) => onChange({ ...values, notes: event.target.value })}
          />
        </label>
        <button className="terminal-action-button" type="submit">
          {submitLabel}
        </button>
      </form>
    </article>
  );
}
