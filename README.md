# Vis

Vis is a compiler-style accessibility analyzer for web source code.

It scans HTML, JSX, TSX, and JavaScript files before production and reports
high-confidence accessibility issues with deterministic diagnostics.

```sh
vis check .
```

The goal is to make accessibility feedback feel closer to Rust, TypeScript,
ESLint, or Biome: fast, actionable, and suitable for local development and CI.

## Status

Vis is early-stage software. The current focus is static analysis for native
HTML semantics and React-style JSX/TSX. The analyzer intentionally favors
high-confidence checks over broad heuristic coverage.

## Supported Inputs

- `.html`
- `.jsx`
- `.tsx`
- `.js` files that contain JSX

Common generated and dependency directories are skipped automatically, including
`node_modules`, `.git`, `dist`, `build`, `.next`, `target`, and `coverage`.

## Usage

Run against the current directory:

```sh
cargo run -q -- check .
```

Run against specific files or directories:

```sh
cargo run -q -- check src examples/react/App.tsx
```

Emit machine-readable JSON:

```sh
cargo run -q -- check . --json
```

Example diagnostic:

```text
error[a11y::missing_alt]
 --> src/App.tsx:12:5

Image missing alt text.

  <img src="/hero.png" />
  ^^^

help: Add alt text, or alt="" if the image is decorative.
```

## Exit Codes

- `0`: no error-level diagnostics were found
- `1`: at least one error-level diagnostic, parse error, or filesystem error was found
- `2`: command usage or configuration error

Warnings and info diagnostics are printed, but they do not fail the command by
default.

## Configuration

Vis looks for `.visrc.toml` in the current directory or an ancestor directory.

```toml
exclude = [
  "dist/**",
  "src/generated/**",
  "examples/fixtures/**"
]

[components]
Button = "button"
NavLink = "a"
Image = "img"
TextField = "input"

[rules]
"a11y::empty_heading" = "warn"
"a11y::autocomplete" = "off"
```

Component mappings tell Vis how project-specific components should be treated
semantically. For example, mapping `NavLink` to `a` lets link rules inspect
`<NavLink to="/reports">Reports</NavLink>`.

Rule severities are:

- `error`
- `warn` or `warning`
- `info`
- `off`

Exclude patterns support exact paths, directory prefixes such as `dist/**`, and
simple `*` / `?` wildcards.

## Implemented Rules

Current rules include:

- image missing `alt`
- heading hierarchy skips
- missing `<html lang>`
- missing or empty page `<title>`
- link missing accessible name
- anchor used as a button
- positive `tabindex`
- clickable non-native elements
- button missing accessible name
- form control missing label
- autocomplete checks for known field purposes
- empty heading
- placeholder used as sole label
- form without submit control
- `title` used as sole label
- invalid `aria-*` attribute names
- invalid `role` values
- redundant native roles
- custom control missing keyboard handler

See [ACCESSIBILITY_RULES.md](ACCESSIBILITY_RULES.md) for WCAG mappings and the
rule roadmap.

## Development

Run the full local verification set:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test
```

Try the included examples:

```sh
cargo run -q -- check examples/html/pass.html examples/react/pass.jsx examples/react/pass.tsx examples/react/components-pass.tsx
cargo run -q -- check examples
```

## Current Limitations

Vis is a static analyzer. It does not execute browser code, compute layout, or
evaluate runtime state. That means it currently does not attempt full color
contrast analysis, keyboard trap detection, timing behavior, dynamic focus
management, or other checks that require rendering/runtime observation.

Framework support is also intentionally narrow. React-style JSX/TSX is the
first serious target; Next.js-specific semantics such as `next/link` and
`next/image` should be handled through framework adapters as the project
matures.

## Roadmap

Near-term priorities:

- harden CLI behavior for CI and editor integrations
- expand integration tests around JSON output, config, and fixtures
- introduce clearer React and Next.js adapter boundaries
- improve component prop mapping for design systems
- keep new rules deterministic and low-noise by default

Longer term, Vis aims to become a framework-aware accessibility compiler that
can power CI gates, editor diagnostics, and design-system validation.
