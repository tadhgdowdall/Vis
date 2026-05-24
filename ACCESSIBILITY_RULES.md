# Accessibility Rules

This document defines the accessibility rules `vis` should follow for the prototype and for future rule development.

The project goal is to enforce high-confidence accessibility requirements that are grounded in WCAG and WAI-ARIA guidance, while avoiding over-reporting and cargo-cult ARIA.

## Core Principles

### 1. Native HTML first

Prefer native HTML elements whenever they already provide the correct semantics and keyboard behavior.

Examples:

- Use `<button>` for actions.
- Use `<a href="...">` for navigation.
- Use `<input>`, `<select>`, and `<textarea>` for form controls.

If a native element is used correctly, `vis` should not require extra ARIA.

### 2. No ARIA is better than bad ARIA

ARIA should be used when native HTML cannot express the required behavior, not as a default addition to already-correct markup.

`vis` should avoid rules that effectively force `aria-label`, `role`, or other ARIA attributes onto semantically correct native elements.

### 3. Enforce outcomes, not rituals

Rules should check whether the accessibility requirement is actually met.

Examples:

- A button needs an accessible name.
- A form control needs an accessible name.
- An image needs an appropriate text alternative.

Rules should not require one specific implementation if multiple valid implementations exist.

### 4. High-confidence only

For the MVP, `vis` should focus on deterministic, low-false-positive checks.

Good MVP rules:

- missing `alt` on `img`
- missing accessible name on `button`
- clickable `div` used as a button without native semantics or keyboard support

Lower-confidence rules can come later.

## Standards Baseline

The rules in this document are primarily based on:

- WCAG 2.2 Success Criterion 1.1.1: Non-text Content
- WCAG 2.2 Success Criterion 4.1.2: Name, Role, Value
- WCAG 2.2 Success Criterion 3.3.2: Labels or Instructions
- WAI-ARIA Authoring Practices Guide

References:

- https://www.w3.org/WAI/WCAG22/Understanding/non-text-content
- https://www.w3.org/WAI/WCAG22/Understanding/name-role-value
- https://www.w3.org/WAI/WCAG22/Understanding/labels-or-instructions.html
- https://www.w3.org/WAI/ARIA/apg/practices/read-me-first/
- https://www.w3.org/WAI/ARIA/apg/practices/names-and-descriptions/
- https://www.w3.org/WAI/ARIA/apg/patterns/button/
- https://www.w3.org/WAI/standards-guidelines/act/rules/97a4e1/
- https://www.w3.org/WAI/standards-guidelines/act/rules/e086e5/2022-06-23/

## Rule Set

### Rule: Native button must have an accessible name

Applies to:

- `<button>`
- elements with semantic role `button`

Requirement:

The control must have a non-empty accessible name.

Valid name sources include:

- visible text content inside the button
- `aria-label`
- `aria-labelledby`
- valid native naming mechanisms where applicable

Do not require:

- `aria-label` on a button that already has suitable visible text

Examples that should pass:

```html
<button>Save</button>
<button aria-label="Close">X</button>
<input type="submit">
<input type="image" src="/search.png" alt="Search">
```

Examples that should fail:

```html
<button></button>
<button><span aria-hidden="true"></span></button>
```

Notes:

- This is an accessible-name rule, not an ARIA rule.
- `vis` should report “missing accessible name”, not “missing aria-label”.
- Native button-like inputs also count here. For example, `input type="submit"` and `input type="reset"` have native naming behavior, while `input type="button"` still needs a usable label.

### Rule: Interactive non-native elements must not fake buttons poorly

Applies to:

- elements like `<div>` or `<span>` used as click targets for actions

Requirement:

If a non-native element is used like a button, it must provide equivalent semantics and keyboard behavior, or the author should use a native `<button>` instead.

High-confidence MVP behavior:

- Flag `div` or `span` with click handlers when they are not natively interactive.

Examples that should fail:

```html
<div onClick="save()">Save</div>
<span onClick="openDialog()">Open</span>
```

Preferred fix:

```html
<button type="button">Save</button>
```

Notes:

- Adding `role="button"` alone is not enough.
- A custom button also needs keyboard support and focus behavior.
- This rule follows the principle that native controls are preferred.

### Rule: Informative images must have text alternatives

Applies to:

- `<img>`

Requirement:

Images must have an appropriate text alternative.

Valid cases:

- informative image with meaningful `alt`
- decorative image with `alt=""`

Examples that should pass:

```html
<img src="/chart.png" alt="Quarterly revenue increased by 12 percent" />
<img src="/border-flourish.png" alt="" />
```

Examples that should fail:

```html
<img src="/hero.png">
<img src="/hero.png" alt>
```

Notes:

- For the prototype, `vis` should at minimum require the presence of `alt`.
- Later versions can distinguish better between missing, empty, and low-quality text alternatives.

### Rule: Form controls must have accessible names

Applies to:

- `<input>` except cases where a type has special naming behavior
- `<select>`
- `<textarea>`

Requirement:

Each form control must have a non-empty accessible name.

Valid sources can include:

- associated `<label>`
- `aria-label`
- `aria-labelledby`
- other valid native naming mechanisms

Additional guidance:

- A programmatic name alone is not always enough for good UX.
- For data-entry fields, visible labels are often required to satisfy WCAG 3.3.2 for all users, not only assistive technology users.

Notes:

- The MVP may defer this rule until label association is parsed reliably.
- When implemented, the rule should distinguish between:
  - missing accessible name
  - accessible name exists, but no visible label for data-entry UX

### Rule: Links should use links, not buttons

Applies to:

- action/navigation elements

Requirement:

Navigation should use `<a href="...">`.
Actions should use `<button>`.

Notes:

- This is important semantically, but may be better as a later rule because intent is not always statically obvious.

### Rule: Links must have accessible names

Applies to:

- `<a href="...">`

Requirement:

Links must have a non-empty accessible name that communicates destination or purpose.

Valid name sources can include:

- visible link text
- `aria-label`
- `aria-labelledby`
- meaningful `alt` text when the link contains only an image

Examples that should pass:

```html
<a href="/reports">View reports</a>
<a href="/reports"><img src="/reports.png" alt="View reports"></a>
```

Examples that should fail:

```html
<a href="/reports"><img src="/reports.png"></a>
<a href="/reports"></a>
```

Notes:

- This aligns with WCAG name requirements and common failures for image-only links.
- For image-only links, the image alt text should describe the link purpose, not just the picture.

## What `vis` Should Not Enforce

The tool should not require:

- `aria-label` on native buttons that already have visible text
- ARIA roles on native elements that already expose the correct semantics
- ARIA added “just in case”

The tool should be careful about:

- treating placeholder text as equivalent to a proper visible label for form UX
- assuming every clickable custom element can be validated without runtime behavior checks
- forcing one naming mechanism when several standards-compliant options exist

## MVP Enforcement Priority

### Implement now

- `img` missing `alt`
- `button` missing accessible name
- clickable `div` and `span`

### Implement once parsing improves

- form controls missing associated labels or accessible names
- `aria-labelledby` support
- icon-only button name detection

### Later, after the core pipeline is stable

- heading hierarchy
- tabindex misuse
- invalid ARIA usage
- link purpose rules
- landmark and region checks

## Rule Writing Guidance

Every new rule should answer these questions:

1. What WCAG or WAI guidance does this map to?
2. Is the requirement deterministic enough for static analysis?
3. Are we checking for the actual accessibility outcome?
4. Are we preferring native semantics over ARIA?
5. Can the rule be explained in a short, actionable diagnostic?

If a rule mainly enforces style preferences rather than accessibility outcomes, it should not be part of the core ruleset.
