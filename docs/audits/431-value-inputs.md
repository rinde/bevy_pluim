# Audit: value inputs (#431)

Slice of the tech-debt epic #426: `slider.rs`, `numeric_stepper.rs`, `text_field.rs`. No code changed.

| Finding | Issue |
| --- | --- |
| `Slider` allows min > max, NaN, non-positive step, value out of range | #470 |
| `Value` / `UIntValue` allow min > max, step 0, value out of range | #471 |
| `TextField` min width above max width; `Val` accepts Auto and percentages | #472 |
| `ValueChange` fires on spawn and app-set values; silent fallbacks in `NumericStepper::on_insert` | #473 |
| Reflect gaps (`ValueRef`, `SliderRefs`, `UIntValue`), duplicated 54 px constant | #474 |
| Missing unit tests for `Value` stepping and `Slider::key_step` | #475 |
| Untested `TextField` promises, Escape does not stop propagation while Enter does | #476 |
| Public fields mutated in place are silently ignored | #477 |

Dismissed: `NumericStepper` `label_min_width` is read at insert like every field and is documented; only the duplicated constant and an untested margin hand-back remain (#474, #476).
Not filed: duplicate children on re-inserting a `NumericStepper` is unverified and an existing test suggests it is handled.
