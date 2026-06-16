# Grand Edge Design Foundation

This folder owns reusable frontend primitives for the Grand Edge terminal.

- `tokens.css` defines the visual source of truth for surfaces, borders, text, accent colors, action tones, radii, and elevation.
- `typography.ts` holds the UI and mono font stacks plus the baseline letter-spacing rule.
- `motion.ts` centralizes restrained timing and easing constants for later shells, cards, charts, and refresh states.
- `layout.ts` defines shell-scale dimensions such as the top bar, nav rail, inspector width, and max workspace width.
- `actions.ts` maps BUY, SELL, WAIT, HOLD, and AVOID to stable design tokens.
- `accessibility.ts` captures minimum interactive size and focus-ring sizing constants.

Rules for later frontend tasks:

- Reuse `--ge-*` tokens instead of introducing one-off hex values.
- Keep motion purposeful and quiet. No bounce, glow, or decorative animation.
- Use mono only for prices, timestamps, model IDs, and dense operational metadata.
- Keep component-specific constants near the component; do not turn this folder into a dumping ground.
