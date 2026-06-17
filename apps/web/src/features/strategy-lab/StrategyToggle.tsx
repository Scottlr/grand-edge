export function StrategyToggle({
  checked,
  disabled,
  pending,
  label,
  onChange,
}: {
  checked: boolean;
  disabled: boolean;
  pending: boolean;
  label: string;
  onChange: (next: boolean) => void;
}) {
  return (
    <label className="strategy-lab-toggle">
      <span className="eyebrow">{label}</span>
      <input
        aria-label={label}
        checked={checked}
        disabled={disabled || pending}
        onChange={(event) => onChange(event.target.checked)}
        type="checkbox"
      />
      <span className="strategy-lab-toggle-label">{pending ? "Saving..." : checked ? "Enabled" : "Disabled"}</span>
    </label>
  );
}
