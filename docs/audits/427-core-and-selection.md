# Audit: core and selection (#427)

Slice of the tech-debt epic #426: `lib.rs`, `select.rs`, `stack.rs`, `breadcrumbs.rs`, `tailwind.rs`. No code changed.

| Finding | Issue |
| --- | --- |
| `process_event_buffer` drains entries it cannot repair | #443 |
| `Selection::Ref` to a missing target, itself or a chain fails silently, untested | #444 |
| Invalid states representable (`One(PLACEHOLDER)`, empty `OneOrMore`, `StackItem` without creator) | #445 |
| Missing `select.rs` tests, `clear` doc mismatch | #446 |
| `COLORS` count, uniqueness and order untested | #447 |
| Breadcrumbs index shift, `Stack` observer never removed, `Reflect` gaps | #448 |
| Public fields on selection and stack types | #449 |

Dismissed: `UiRoot` claims are covered in `tests/headless.rs`; `MouseEvent` is crate-private and covered through widget scenarios.
