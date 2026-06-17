import type { InvalidationRuleDto } from "../../domain/recommendation";

export function InvalidationRules({ rules }: { rules: InvalidationRuleDto[] }) {
  if (rules.length === 0) {
    return <p className="evidence-empty">No clear danger points are available yet.</p>;
  }

  return (
    <ul className="invalidation-rules">
      {rules.map((rule) => (
        <li key={`${rule.metric}-${rule.operator}-${rule.threshold}`}>
          <strong>{rule.reason}</strong>
          <span>
            {rule.metric} {rule.operator} {rule.threshold}
            {rule.currentValue ? ` (now ${rule.currentValue})` : ""}
          </span>
        </li>
      ))}
    </ul>
  );
}
