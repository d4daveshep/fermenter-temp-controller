## Context

The target-temperature form (`target_form.html`) and brew-identifier form (`brew_form.html`) each present a single-field form with an "Update" submit button. There is currently no way to navigate back to the dashboard without submitting (or using the browser back button). The dashboard itself uses `<form method="get">` with buttons to navigate to these forms — the natural counterpart is a cancel button that returns to `/`.

## Goals / Non-Goals

**Goals:**
- Add a Cancel button to the target-temperature form that navigates to `/` without a form submission.
- Add a Cancel button to the brew-identifier form that navigates to `/` without a form submission.
- Cancel button is visually distinct from the submit button (secondary action).

**Non-Goals:**
- No new routes, handlers, or backend logic.
- No JavaScript or HTMX — the cancel action is a plain form GET navigation.
- No form-reset behavior (the operator is leaving the form, not clearing it).

## Decisions

1. **Cancel as `<form method="get" action="/"><button type="submit">Cancel</button></form>`, not `<a>` link or inline button**
   - A `reset` button clears the form fields but stays on the page — it doesn't navigate back.
   - HTMX could provide a `hx-get="/"` with `hx-target` but that replaces content in-place; a full navigation to `/` is simpler and more correct for a "leave this page" action.
   - A separate `<form>` wrapping the Cancel `<button>` matches the existing dashboard navigation pattern (`dashboard.html` uses `<form method="get" action="/target"><button type="submit">Set target temperature</button></form>`), keeping the codebase consistent.
   - A plain `<a>` link doesn't render as a button without additional CSS, and the user specifically requested buttons.
2. **Cancel button rendered as a separate form after the Update form** (secondary below primary)
   - Nested forms are invalid HTML, so Cancel must be its own `<form>` outside the POST form. It renders as a block element below Update rather than inline next to it.
   - Consistent with the dashboard's approach of separate `<form>` blocks for each navigation action.

3. **No backend changes**
   - The cancel action is entirely client-side navigation. No handler, route, or state changes needed.

## Risks / Trade-offs

- **Lost unsaved input**: If the operator has typed a value and clicks Cancel, the input is discarded. This is expected behavior for a cancel action — users understand this convention. No mitigation needed.

- **Existing snapshot tests need updating**: The test `form_templates_do_not_contain_back_link_or_confirmed_block` currently asserts that the forms contain neither "Back to dashboard" text nor a `class="confirmed"` block. The cancel button adds `<form method="get" action="/"><button type="submit">Cancel</button></form>`, so this test must be updated to assert the cancel button's presence with the correct `/` target.

- **Existing snapshot snapshots**: `target_form_with_current_target_snapshot`, `target_form_with_error_snapshot`, `brew_form_with_current_brew_snapshot`, and `brew_form_with_error_snapshot` will all need their snapshots updated via `INSTA_UPDATE=always`.

## Migration Plan

1. Edit `target_form.html` — add Cancel button (`<form method="get" action="/"><button type="submit">Cancel</button></form>`) after the Update form.
2. Edit `brew_form.html` — add Cancel button (same markup) after the Update form.
3. Update tests:
   - Modify `form_templates_do_not_contain_back_link_or_confirmed_block` to assert cancel button presence and correct `/` target instead of asserting absence of back link.
   - Update insta snapshots.
4. Run `cargo test` to confirm everything passes.
5. Visual check by running the app locally and verifying both forms.
