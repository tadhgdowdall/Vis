# Accessibility Rules

This document defines `vis`'s accessibility rules — what we enforce, what the
WCAG mappings are, and what's planned.

## Core Principles

1. **Native HTML first.** If a native element already provides the correct
   semantics, don't require ARIA on top of it.
2. **No ARIA is better than bad ARIA.** Only suggest ARIA when native HTML
   cannot express the required behavior.
3. **Enforce outcomes, not rituals.** Check whether the accessibility need is
   *met*, not whether a specific implementation pattern was followed.
4. **High-confidence only.** Every rule must be deterministic enough for static
   analysis with very few false positives. Heuristic rules come later and are
   opt-in.

---

## Implemented Rules

| Rule | Code | WCAG SC | Level |
|------|------|---------|-------|
| Image missing alt text | `a11y::missing_alt` | [1.1.1 Non-text Content](https://www.w3.org/WAI/WCAG22/Understanding/non-text-content) | A |
| Button missing accessible name | `a11y::button_label` | [4.1.2 Name, Role, Value](https://www.w3.org/WAI/WCAG22/Understanding/name-role-value) | A |
| Clickable div used as button | `a11y::clickable_div` | [2.1.1 Keyboard](https://www.w3.org/WAI/WCAG22/Understanding/keyboard), [4.1.2 Name, Role, Value](https://www.w3.org/WAI/WCAG22/Understanding/name-role-value) | A |
| Form control missing label | `a11y::form_control_label` | [1.3.1 Info and Relationships](https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships), [3.3.2 Labels or Instructions](https://www.w3.org/WAI/WCAG22/Understanding/labels-or-instructions) | A |
| Link missing accessible name | `a11y::link_label` | [2.4.4 Link Purpose (In Context)](https://www.w3.org/WAI/WCAG22/Understanding/link-purpose-in-context), [4.1.2 Name, Role, Value](https://www.w3.org/WAI/WCAG22/Understanding/name-role-value) | A |
| Anchor used as button | `a11y::link_semantics` | [4.1.2 Name, Role, Value](https://www.w3.org/WAI/WCAG22/Understanding/name-role-value) | A |

All 6 rules target WCAG 2.2 Level A — the minimum compliance bar. They are
deterministic, produce very few false positives, and every violation maps to a
concrete, actionable fix.

---

## Roadmap — Rules to Add

Each row is rated on:

- **Priority** — P1 (core compliance gap), P2 (important, some edge cases),
  P3 (nice to have, heuristic)
- **Difficulty** — Easy (straightforward check), Medium (requires context
  tracking), Hard (subjective or needs rendering)
- **False positive risk** — Low (deterministic), Medium (some ambiguity),
  High (heuristic, opt-in recommended)

### Top Priority (P1)

These close critical WCAG Level A gaps with deterministic, low-false-positive
checks:

| # | Rule | WCAG SC | Level | Difficulty | FP Risk |
|---|------|---------|-------|------------|---------|
| 7 | `tabindex` > 0 | 2.4.3 Focus Order | A | Easy | Low |
| 8 | Heading hierarchy skip | 1.3.1 Info and Relationships | A | Easy | Low |
| 9 | `lang` attribute on `<html>` | 3.1.1 Language of Page | A | Easy | Low |
| 10 | `<title>` element present and non-empty | 2.4.2 Page Titled | A | Easy | Low |
| 11 | Autocomplete on form fields | 1.3.5 Identify Input Purpose | AA | Medium | Low |

Details:

**Rule 7: `tabindex` > 0** — Any positive `tabindex` value breaks the natural
focus order. Flag `tabindex="1"`, `tabindex="2"`, etc. `tabindex="0"` and
`tabindex="-1"` are fine.

**Rule 8: Heading hierarchy skip** — `<h1>` followed by `<h3>` without an
intermediate `<h2>` breaks document outline for screen readers. Track the
current heading level and flag jumps > 1. This also catches pages with no `<h1>`.

**Rule 9: `lang` on `<html>`** — Screen readers need a language hint to load
the correct pronunciation rules. `lang` must be a valid BCP 47 tag. A
heuristic: flag missing `lang`, flag obviously invalid values (empty, "x"),
don't try to validate full BCP 47.

**Rule 10: `<title>` element** — Every page needs a non-empty `<title>`. This
is trivially checkable on full HTML documents and component files that render
`<head>`.

**Rule 11: Autocomplete on form fields** — Inputs of type `email`, `name`,
`tel`, `address`, `postal-code`, `country`, `cc-*`, `username`, `password` (new
and current) should have `autocomplete` set. Use the [WCAG Input Purposes
list](https://www.w3.org/TR/WCAG22/#input-purposes). This is deterministic
because inputs have well-known types mapped to specific autocomplete values.
Only flag when both `type` AND context suggest autocomplete is expected. Do NOT
require autocomplete on search fields, generic text inputs, or custom
components where purpose is unclear.

### Priority 2 (P2)

These address important criteria but have more edge cases or require some
heuristics:

| # | Rule | WCAG SC | Level | Difficulty | FP Risk |
|---|------|---------|-------|------------|---------|
| 12 | Keyboard event handler missing on interactive custom elements | 2.1.1 Keyboard | A | Medium | Low |
| 13 | `outline: none` without `:focus-visible` | 2.4.7 Focus Visible | AA | Medium | Low |
| 14 | Missing skip link / bypass block | 2.4.1 Bypass Blocks | A | Medium | Medium |
| 15 | `aria-*` attribute validity | 4.1.2 Name, Role, Value | A | Hard | Low |
| 16 | Invalid `role` values or mismatched parent/child roles | 4.1.2 Name, Role, Value | A | Hard | Low |
| 17 | `role` on semantic native elements | 4.1.2 Name, Role, Value | A | Medium | Low |

Details:

**Rule 12: Keyboard handler** — If a custom element (`<div>`, `<span>`) has
`onClick` but no `onKeyDown`/`onKeyUp` handler, it may be keyboard
inaccessible. Only flag when the element is clearly being used as a button
(click handler + role="button" or interacts with state). Don't flag
onKeyDown-only patterns.

**Rule 13: Focus visible** — `outline: none` or `outline: 0` removes the
browser's default focus indicator. If no `:focus-visible` alternative is
provided, keyboard users can't see where they are. Check both inline styles
and `className` references to Tailwind/shadcn patterns (`outline-none`,
`focus:outline-none`). If a custom focus style exists (box-shadow, ring,
different outline on `:focus-visible`), don't flag.

**Rule 14: Skip link** — Check that the page has at least one skip navigation
link (an `<a>` with `href` pointing to `#main-content` or similar, with text
like "Skip to content" or "Skip navigation"). This only applies to
layout/page component files, not leaf components.

**Rule 15: `aria-*` attribute validity** — Check that `aria-` attribute names
are valid ARIA properties (not typos like `aria-labeledby`). Check that values
are valid for their type (boolean, token list, ID reference). This is
essentially an ARIA schema validator.

**Rule 16: Invalid `role`** — Check that `role` values are valid ARIA roles.
Check that required parent/child relationships are met (e.g., `role="option"`
must be inside `role="listbox"`). Check that `role` isn't applied to elements
that already have that implicit role (`<button role="button">`).

**Rule 17: `role` on native elements** — Flag when a `role` that matches the
native semantics is applied redundantly (`<main role="main">`). Flag when a
`role` contradicts the native semantics (`<button role="heading">`). The
latter breaks the element's built-in keyboard behavior.

### Priority 3 (P3)

Heuristic or subjective checks best suited for opt-in configuration or
informational/warning severity:

| # | Rule | WCAG SC | Level | Difficulty | FP Risk |
|---|------|---------|-------|------------|---------|
| 18 | Link text quality (generic text) | 2.4.4 Link Purpose | A | Hard | High |
| 19 | Placeholder used as label | 3.3.2 Labels or Instructions | A | Medium | Medium |
| 20 | `title` attribute used as sole label | 4.1.2 Name, Role, Value | A | Medium | Low |
| 21 | Color contrast (inline styles) | 1.4.3 Contrast (Minimum) | AA | Hard | High |
| 22 | Form without submit button | 3.2.2 On Input | A | Medium | Medium |
| 23 | Empty heading | 1.3.1 Info and Relationships | A | Easy | Low |

Details:

**Rule 18: Link text quality** — Flag links with generic text: "click here",
"read more", "learn more", "here", "link". These don't communicate destination
out of context. Use a configurable list of generic phrases. Only flag when the
link has no `aria-label` or `aria-labelledby` compensating.

**Rule 19: Placeholder as label** — Inputs that have a `placeholder` but no
associated `<label>`, `aria-label`, or `aria-labelledby`. Placeholder text
disappears on focus and has poor contrast — it's not a sufficient label. This
is essentially a subset of the form_control_label rule that catches the
"well-intentioned but wrong" pattern.

**Rule 20: `title` as label** — The `title` attribute is not reliably
accessible across all assistive technologies. Flag controls that rely solely
on `title` for their accessible name when no label, aria-label, or
aria-labelledby is present.

**Rule 21: Color contrast** — Check inline `style` attributes for color/background
pairs that obviously fail contrast thresholds (4.5:1 for normal text, 3:1 for
large text). This is inherently heuristic without a rendering engine — only
flag the most obvious failures (e.g., `#ccc` on `#fff`). Cannot check
CSS/Tailwind class-based colors without a CSS resolver.

**Rule 22: Form without submit button** — A `<form>` that contains inputs but
no `<button type="submit">` or `<input type="submit">`. Users without
JavaScript may not be able to submit. Don't flag forms with `onSubmit` handlers
since the submission path may be JavaScript-driven.

**Rule 23: Empty heading** — `<h1>` through `<h6>` with no text content or
accessible name. These create gaps in the document outline for screen reader
users.

---

## Out of Scope (Can't Reasonably Check with Static Analysis)

These WCAG criteria require runtime behavior, visual rendering, or audio
analysis — they're inherently out of scope for a static analysis tool:

- **1.2.x Time-based Media** — Requires validating captions, transcripts, audio descriptions
- **1.4.1 Use of Color** — Requires rendering to know if information is conveyed only by color
- **1.4.2 Audio Control** — Requires runtime audio playback
- **2.1.2 No Keyboard Trap** — Requires runtime keyboard navigation testing
- **2.2.x Timing** — Requires runtime behavior
- **2.3.1 Seizure (flashes)** — Requires rendering and frame analysis
- **2.5.x Input Modalities** — Mostly runtime pointer/touch behavior
- **3.2.3 Consistent Navigation** — Requires cross-page comparison
- **3.2.4 Consistent Identification** — Requires cross-page comparison
- **3.3.4 Error Prevention** — Requires understanding form submission context

---

## False Positive Mitigation Strategy

For every rule, the primary design constraint is: **would a real user trust
this tool if it flagged their code?** A single false positive erodes that
trust faster than ten correct detections build it.

Strategies by difficulty:

1. **Deterministic rules (low FP risk):** Ship as errors. These are
   mechanically checkable and never wrong in practice.
2. **Medium FP risk:** Ship as warnings by default. Let users promote to
   errors via `.visrc.toml`.
3. **High FP risk:** Ship as opt-in only. Never on by default. Document the
   known false positive cases.
4. **Configurable ignore:** Every rule should be toggleable in `.visrc.toml`.
   File/glob-based ignores (`components/ui/*`) handle component library false
   positives.

---

## Rule Writing Guidance

Every new rule must answer:

1. What WCAG criterion does this map to?
2. Is it deterministic enough for static analysis?
3. What is the false positive scenario?
4. What is the actionable fix a developer would make?
5. Can the diagnostic be explained in one sentence?

A rule that requires a paragraph of explanation for every violation should
either be simplified or marked opt-in.

---

## References

- [WCAG 2.2 Specification](https://www.w3.org/TR/WCAG22/)
- [Understanding WCAG 2.2](https://www.w3.org/WAI/WCAG22/Understanding/)
- [WAI-ARIA Authoring Practices Guide](https://www.w3.org/WAI/ARIA/apg/)
- [ARIA in HTML Specification](https://www.w3.org/TR/html-aria/)
- [ACT Rules Community Group](https://www.w3.org/WAI/standards-guidelines/act/rules/)
- [axe-core Rule Descriptions](https://github.com/dequelabs/axe-core/blob/develop/doc/rule-descriptions.md)
- [eslint-plugin-jsx-a11y](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y)
