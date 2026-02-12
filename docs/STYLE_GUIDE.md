# Otter Budget Tracker – UI Style Guide (v0.1, Markdown)

Scope: dark-mode only, Vue + Vuetify, Chart.js. This guide documents **current** patterns from your screenshots + the provided Vuetify theme. Desktop layout is **described, not locked**.

---

## 1) Design principles

- **Dark, cosmic, low-noise UI**: dark surfaces, subtle borders/separators, bright accent for “paid / action”.
- **Cards over screens**: content is grouped into rounded cards with minimal elevation.
- **Data clarity**: budget vs paid vs remaining is always visually distinct (neutral vs accent vs success/error).
- **One primary accent**: pink/magenta drives emphasis (primary, paid bars, primary CTAs).

---

## 2) Theme tokens (source of truth)

Use exactly these Vuetify theme colors:

```ts
primary:    #E040A0
secondary:  #8890A8
accent:     #E040A0
success:    #5AD8A0
warning:    #FFB74D
error:      #FF5070
background: #0B0D1A
surface:    #121630
on-primary: #FFFFFF
on-secondary: #E8EAF0
on-background: #E8EAF0
on-surface: #E8EAF0
surface-variant: #121630
```

### Semantic usage (hard rules)

- **Primary / Accent (`#E040A0`)**: CTAs, “Paid” series, active nav item, key highlights.
- **Success (`#5AD8A0`)**: “Remaining” value and positive states.
- **Error (`#FF5070`)**: overspent/over-budget states, destructive actions.
- **Secondary (`#8890A8`)**: muted labels, dividers, grid lines, inactive UI.
- **Background (`#0B0D1A`)**: app backdrop.
- **Surface (`#121630`)**: cards, sheets, tables, nav background.

---

## 3) Layout & spacing

### Responsive strategy (current)

- **Phone-first**: single-column stacked sections, bottom navigation.
- **Desktop**: content can expand wide (your current layout shows multi-column chart area + full-width tables). Don’t hard-cap max width yet; keep sensible horizontal gutters.

### Spacing scale (inferred → become tokens)

Adopt an 8-based scale with a few helpers:

- `space-1 = 4px`
- `space-2 = 8px`
- `space-3 = 12px`
- `space-4 = 16px`
- `space-5 = 20px`
- `space-6 = 24px`
- `space-8 = 32px`
- `space-10 = 40px`

**Defaults**

- Screen padding: `16px` (phone), `24–32px` (desktop)
- Card internal padding: `16–24px`
- Card-to-card gap: `16–24px`

---

## 4) Shape, elevation, borders

### Corner radius (hard rule)

- **Use `rounded="lg"` everywhere by default** (cards, buttons, sheets, inputs where applicable).
- Vuetify `rounded="lg"` maps to **16px** radius (treat that as the standard).

### Elevation (hard rule)

- **No heavy shadows**. Prefer **0 elevation** (matches your defaults).
- Separation comes from **surface contrast** + **subtle borders**.

### Dividers / borders (pattern)

Use thin separators with low-opacity secondary:

- Border/divider color: `secondary` at ~**10–20% alpha**
- Row separators in tables: same treatment (no harsh lines)

---

## 5) Background & “cosmic” treatment

- App background is `background` with a subtle star/noise effect.
- Keep decorative elements **non-interactive** and low contrast (avoid competing with data).
- If you implement it as CSS, keep opacity very low and avoid large bright clusters.

---

## 6) Typography (intentionally loose)

Since you’re on Vuetify defaults, keep it simple and consistent:

**Conventions**

- **Section headers**: uppercase, letter-spaced, secondary/muted (e.g., “CHARTS”, “CATEGORIES”, “TRANSACTIONS”).
- **Key numbers**: larger size, high contrast; color indicates meaning (primary/success/error).
- **Table values**: regular weight; only colorize for states (error when overspent).

---

## 7) Navigation

### Bottom navigation (current pattern)

- Background: `surface`
- Active item: icon + label in `primary`
- Inactive items: `secondary` (or on-surface with lowered opacity)
- Height: keep touch-friendly (target **≥ 56px**)
- On desktop: keep as-is for now (future: side rail is a possible evolution, not part of v0.1)

---

## 8) Cards & sections

### Stat tiles (top summary)

Pattern: 3 tiles showing **Total Budgeted / Total Paid / Remaining**.

- Container: card on `surface`
- Label: muted, uppercase
- Value:
  - Budgeted: `primary` (pink)
  - Paid: `on-surface` (neutral) or slightly muted
  - Remaining: `success` (green)

### Section card headers

- Left: icon + uppercase title
- Right: optional chevron/collapse affordance
- Header row has slightly higher contrast than body text, but still muted.

---

## 9) Tables & lists

### Categories table

Columns: Category | Paid/Budgeted | Due day (and optionally actions)

- Header row: uppercase, muted (`secondary`)
- Rows: no zebra striping; rely on separators + hover/active states.
- Alignment:
  - Category: left
  - Amounts: right (or aligned by decimal if you ever implement it)
  - Due day: centered/right

### Status coloring (hard rules)

- **Over budget** (paid > budgeted):
  - Amount text: `error`
  - Row background: `error` at ~**10–15% alpha** (a subtle red tint, like your screenshot)
- Normal:
  - Text: `on-surface`
  - Muted secondary info: `secondary`

### Row interactions

- Desktop hover: subtle surface-variant tint (low contrast)
- Selected/focused row: same family (don’t introduce bright outlines)

---

## 10) Buttons

### Primary CTA

- Filled button with `color="primary"`
- Text: `on-primary` (white)
- Rounded: `lg`

### Secondary / Cancel

- Tonal/outlined neutral button (e.g., `variant="tonal"` or `outlined`) on `surface`
- Text: `on-surface` or `secondary`
- Rounded: `lg`

### Destructive actions

- Use `error` for delete/trash icons and destructive confirmations.
- Keep destructive actions visually smaller than primary CTAs (icon button or text button).

---

## 11) Inputs & forms (overlay sheet pattern)

This is a **core global pattern**: “When something is editable it opens an overlay form”.

### Overlay behavior (hard rules)

- Opens as a **bottom sheet** overlay.
- Default height: content-driven; can expand to full screen when long.
- When expanded:
  - Sheet scrolls internally (**independent scrolling**).
  - Background content position is preserved; closing returns you to the previous scroll position.
- Dismissal:
  - **Cancel / Save** buttons
  - Tap outside = **Cancel**
  - `Esc` = **Cancel**
  - Swipe-down (mobile) = **Cancel**
  - No “unsaved changes” prompt (by design).

### Overlay styling

- Surface: `surface`
- Top corners: rounded `lg` (often visually reads like larger top radius; keep consistent)
- Padding: `24px` top for title area, `16–24px` body
- Title: large, left-aligned (“Add Budget Entry”, “Add Transaction”, etc.)

### Inputs (hard rules)

- Use your defaults everywhere:
  - `variant="outlined"`
  - `density="compact"`
- Labels: muted (`secondary`)
- Field spacing: `16–24px` vertical gap
- Dropdown chevrons/icons: muted (`secondary`)

### Form actions row

- Bottom-right aligned on larger screens; on small screens can still align right but keep touch targets large.
- Buttons: **Cancel (neutral)** + **Save (primary)**.

---

## 12) Charts (Chart.js)

### Color mapping (hard rules)

- “Paid” series: `primary` (`#E040A0`)
- “Budgeted” series: neutral (use `secondary` with reduced alpha)
- “Remaining” (in donut legend or values): `secondary` or muted on-surface
- Progress value text (e.g., “31%”): `primary`

### Grid lines / axes

- Axis labels: muted (`secondary`)
- Grid lines: `secondary` at ~10–15% alpha
- Avoid strong tick marks; keep the chart “floating” and subtle like the screenshots.

### Donut chart

- Paid slice: `primary`
- Remaining slice: darker neutral (surface/secondary blend)
- Center label: percent in `primary`

---

## 13) States & feedback

- **Success**: use `success` sparingly (mostly “Remaining”).
- **Warning**: reserved for near-due / upcoming attention states (not heavily shown yet).
- **Error**: overspent and destructive actions.
- Focus states: keep accessible but not neon—prefer subtle outlines using `secondary`/`primary` at low alpha.

---

## 14) Vuetify defaults (codified)

Keep these as global defaults:

- `VCard`: `rounded="lg"`, `elevation=0`
- `VBtn`: `rounded="lg"`
- `VTextField/VSelect/VAutocomplete`: `outlined`, `compact`

---

## 15) “Do / Don’t” quick rules for agents

**Do**

- Use semantic tokens: primary for CTAs/paid, success for remaining, error for overspent.
- Keep everything rounded `lg`.
- Keep separators subtle and avoid big shadows.
- Use the overlay sheet pattern for editing everywhere.

**Don’t**

- Introduce new accent colors.
- Add heavy elevation/shadows.
- Mix input variants/densities across screens.
- Make tap-outside / Esc behave differently from Cancel.
