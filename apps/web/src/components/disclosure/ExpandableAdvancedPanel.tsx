import type { ReactNode } from "react";

import { motion } from "../../design/motion";

export function ExpandableAdvancedPanel({
  title,
  children,
  defaultOpen = false,
}: {
  title: string;
  children: ReactNode;
  defaultOpen?: boolean;
}) {
  return (
    <details className="advanced-panel" open={defaultOpen}>
      <summary
        className="advanced-panel-summary"
        style={{
          transitionDuration: `${motion.panelMs}ms`,
          transitionTimingFunction: motion.easingPanel,
        }}
      >
        {title}
      </summary>
      <div className="advanced-panel-body">{children}</div>
    </details>
  );
}
