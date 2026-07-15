## Context

The `reason_code` field is fully plumbed through the system: Arduino serial JSON → serde deserialization in `model::Reading` → in-memory `AppState` → Redis persistence. The `status.html` MiniJinja template renders a `<dl class="reading">` with `Action`, `Target`, `Average`, `Min`, `Max`, and `Ambient` rows but omits `reason_code`. The field is already available as `{{ reading.reason_code }}` in the template context — no backend changes needed.

## Goals / Non-Goals

**Goals:**
- Display the controller's reason code on the live-polled dashboard status fragment, adjacent to the action name.

**Non-Goals:**
- Human-readable descriptions or lookups for reason codes.
- Adding reason code to the time-series data (it's a string, not a numeric metric).
- Controller logic or serial contract changes.

## Decisions

1. **Add row after Action, before Target** — placing `Reason` immediately after `Action` keeps the cause-mechanism pair adjacent, making it intuitive to read: the action is *what* the controller is doing, the reason is *why*.

2. **Template-only change** — `reading.reason_code` is already populated and present in the render context for any observed reading. No controller, handler, or model changes required.

## Risks / Trade-offs

- **Reason codes are opaque strings** (e.g. `"RC3.1"`) without further documentation. Operators unfamiliar with the firmware may not understand them. → This is acceptable for the initial change; a follow-up could add tooltip descriptions or a lookup table.
