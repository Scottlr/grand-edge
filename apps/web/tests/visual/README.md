# Frontend Visual QA

Grand Edge uses documented browser QA fallback for T036 rather than adding a new e2e dependency layer.

## Required local checks

1. Run `npm --prefix apps/web run build`.
2. Run `npm --prefix apps/web run preview`.
3. Open `http://127.0.0.1:4173/`.
4. Verify these viewports:
   - `1440x900`
   - `1024x768`
   - `390x844`

## Required journey checks

- Beginner: Dashboard -> BUY card -> Show why -> Learn: Confidence -> Track item
- Intermediate: Buy -> Item -> Linked Items -> What happens if this moves? -> Did this work before?
- Advanced: Accuracy -> Open advanced detail -> Strategy Lab -> simulation mode comparison

## Required state checks

- Loading
- Live
- Stale
- Degraded
- Empty
- Error

## Accessibility checks

- Keyboard focus reaches nav, dashboard cards, opportunity rows, inspector, strategy toggles, and portfolio form fields.
- Focus ring remains visible.
- Reduced-motion OS preference removes nonessential motion.
