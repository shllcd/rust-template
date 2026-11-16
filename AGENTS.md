# AGENTS.md

## Build

- Before handover: `cargo xtask ci` (fmt, clippy, check-deps, test).
- Rust 1.100+, edition 2024, resolver 3.
- Dependency versions live only in `[workspace.dependencies]`.

## Comments

Write no comments. Doc comments (`///`, `//!`) count as comments.
Rename, extract or tighten a signature instead.

Exceptions: a fact the code cannot express (`// upstream returns 200 on auth failure`)
and license headers of ported code. No dead code, commented-out code or `TODO`.

## Structure

Code that changes together lives together; everything depends on what changes less.

- Crates live in `crates/` and are stability tiers: a crate depends only on more stable ones.
  Create a crate when code needs it, never ahead of time, and add it to `TIERS` in
  `xtask/src/main.rs`. `cargo xtask check-deps` enforces the order.
- Modules are domains, not kinds of code: no `utils`, `helpers`, `common`, `types`.
- One source of truth for every value, list and rule.
- No mutable globals. Read env and files once in `main`, parse them into types, pass them down.
- Make invalid states unrepresentable: newtypes and enums at the boundary.
- Keep logic pure and I/O at the edges: functions take data and return data or decisions.
- Concrete types first. A trait, generic, flag or parameter needs two uses that exist today.
- Public APIs return the smallest shape that answers the caller's question. Storage and
  transport details (hashes, internal ids, cursors) stay inside.
- A special case (an `if` for one value, a swallowed error, a retry, a sleep) is a requirement
  the design missed. Find the general rule that removes it.

## Idioms

- Errors: one `thiserror` enum per domain, propagated with `?`.
- Cleanup on every exit path: `std::mem::DropGuard`, dismissed on success.
- Borrow in parameters (`&str`, `&[T]`, `&Path`); own in return values and fields.
- Clone to share ownership on purpose, never to quiet the borrow checker.
- Suppress a lint only with `#[expect(lint, reason = "...")]`, never `#[allow]`.
- Units in names (`timeout_ms`), predicates for booleans (`is_pending`).

## Tests

Verify your work with scratch tests, then delete them. Commit tests only when the user asks;
committed tests go through public interfaces and can actually fail.

## Commits

- One logical change per commit: imperative subject, optional scope (`router: ...`), no body.
- Always commit `Cargo.lock`.
- Worktree changes you didn't make are the user's: never edit, revert, reformat or absorb
  them. If they overlap your task, stop and ask.
