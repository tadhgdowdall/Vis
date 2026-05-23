# Vis

## The Idea

Vis is a compiler-style accessibility analysis tool for the web.

The goal is to treat accessibility issues the same way modern compilers treat code errors:

- deterministic
- actionable
- developer-friendly
- caught before production

Instead of accessibility being:

- an afterthought
- a browser audit
- a Lighthouse score
- a manual QA process

Vis makes accessibility part of the development pipeline itself.

The experience should feel closer to:

- Rust Compiler
- TypeScript
- ESLint
- Biome

than to traditional accessibility scanners.

## Core Concept

Vis takes source files:

- HTML
- JSX
- TSX
- eventually Vue/Svelte/etc.

and converts them into a semantic accessibility representation.

Instead of analyzing syntax directly, Vis analyzes meaning.

Example:

```tsx
<div onClick={save}>
  Save
</div>
```

becomes something like:

```text
Interactive element
- keyboard inaccessible
- non-focusable
- role missing
```

Then rules evaluate that semantic structure and emit compiler-style diagnostics.

## The Long-Term Vision

Eventually Vis becomes:

- a framework-agnostic accessibility compiler
- a CI/CD accessibility gate
- a VSCode extension
- a semantic UI correctness engine
- a design-system validator
- potentially an accessibility "type system" for UI

The ideal developer workflow:

```sh
vis check .
```

And the build fails if accessibility guarantees are violated.

## Parsing Strategy

Vis should use a hybrid parsing architecture rather than trying to force every framework through one generic parser.

The core idea is:

- use production-grade parsers for the source languages themselves
- add framework adapters for framework-specific file formats and semantics
- normalize everything into the same accessibility IR before rules run

This keeps the compiler pipeline consistent while still being realistic about how modern frontend stacks differ.

### Planned Parsing Layers

#### Core Parsers

- HTML: a browser-grade HTML parser
- JavaScript / TypeScript / JSX / TSX: a production-grade Rust parser

#### Framework Adapters

- React: parse JSX/TSX and map native elements plus common component patterns into IR
- Next.js: build on the React path first, then add framework-aware semantics for things like `next/link`, `next/image`, and other Next-specific abstractions
- Vue: parse SFC structure, then analyze template and script sections separately
- Svelte: use a dedicated adapter for `.svelte` files
- Astro: use a dedicated adapter for `.astro` files

### Near-Term Priority

The first production-grade target should be:

- React
- Next.js

That means the next real parser milestone is not “support every framework a little bit.”

It is:

- replace the current prototype JSX/TSX parser with a real parser
- keep HTML on a real standards-compliant path
- introduce framework adapter boundaries early
- make React and Next.js the first serious end-to-end analysis target

### Why This Approach

This is the most practical route to production-grade analysis.

It avoids:

- relying on a simplistic hand-rolled parser long-term
- pretending Vue, Svelte, Astro, and React all have the same source model
- coupling rules directly to framework syntax

It supports:

- better correctness on real codebases
- cleaner expansion across frameworks
- a stable semantic IR that rules can depend on

## Why Rust

Rust is a strong fit because Vis is fundamentally:

- a parser
- a compiler
- a static analysis engine
- a diagnostics system

Rust gives:

- speed
- safety
- excellent CLI tooling
- strong parser ecosystem
- compiler-grade diagnostics
- future WASM support

The architecture naturally maps to compiler design patterns.

## MVP Scope

The MVP is intentionally narrow.

It is not:

- full WCAG compliance
- runtime browser simulation
- a visual accessibility testing suite

The MVP is:

A CLI tool that scans HTML/JSX/TSX and reports high-confidence accessibility violations with compiler-style output.

## Initial MVP Features

### Supported Input

- HTML
- JSX
- TSX

### Initial Rules

- img missing alt
- clickable divs
- missing button labels
- form inputs missing labels
- invalid heading hierarchy
- tabindex misuse

### Output Style

```text
error[a11y::missing_alt]
 --> src/App.tsx:12:5

Image missing alt text

help: add alt="" for decorative images
```

The UX is a major part of the product.

## Core Technical Architecture

Vis works in stages:

```text
Source files
    ↓
Source-specific parser
    ↓
Framework adapter
    ↓
Accessibility IR
    ↓
Rule Engine
    ↓
Diagnostics
    ↓
CLI / CI Output
```

The most important part is the Accessibility IR (Intermediate Representation).

This semantic layer abstracts away framework syntax and allows rules to operate consistently across technologies.

Example semantic node:

```rust
A11yNode {
    role: Button,
    interactive: true,
    focusable: false,
}
```

This is what makes Vis compiler-like rather than just another linter.

The parser and adapter stages are separate on purpose:

- the parser understands syntax
- the adapter understands framework conventions and maps them into semantics

## Starting Point

The first version should stay extremely small and focused.

Initial repository structure:

```text
vis/
├── vis-cli
├── vis-parser
├── vis-ir
├── vis-rules
├── vis-analyzer
└── vis-diagnostics
```

## First Milestone

The first real milestone is:

```sh
vis check example.tsx
```

successfully producing:

```text
error[a11y::clickable_div]
 --> src/App.tsx:14:3

Interactive elements must be keyboard accessible.
```

Once this works:

- the architecture is validated
- the pipeline exists
- rules can scale
- framework support can expand

That is the foundation of the entire project.

## Delivery Approach

This should start at the smallest achievable point and build progressively from there.

The repo should optimize for:

- a very small first vertical slice
- explicit architecture boundaries
- high-confidence rules only
- progressive expansion after the first end-to-end diagnostic works

The direction is intentionally flexible and can pivot as the implementation teaches us what the right product shape should be.
