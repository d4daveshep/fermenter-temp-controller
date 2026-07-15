## Context

The target-temperature form (`target_form.html`) and brew-identifier form (`brew_form.html`) each present a single-field form with an "Update" submit button. There is currently no way to navigate back to the dashboard without submitting (or using the browser back button). The dashboard itself uses `<form method="get">` with buttons to navigate to these forms — the natural counterpart is a cancel button that returns to `/`.

## Goals / Non-Goals

**Goals:**
- Add a Cancel button to the target-temperature form that navigates to `/` without a form submission.
- Add a Cancel button to the brew-identifier form that navigates to `/` without a form submission.
- Cancel button is visually distinct from the submit button (secondary action).

**Non-Goals:**
- No new routes, handlers, or backend logic.
- No JavaScript or HTMX — the cancel action is a plain navigation link.
- No form-reset behavior (the operator is leaving the form, not clearing it).

## Decisions

1. **Cancel as `<a>` link styled as button, not `<button type="reset">` or HTMX**
   - A `reset` button clears the form fields but stays on the page — it doesn't navigate back.
   - HTMX could provide a `hx-get="/"` with `hx-target` but that replaces content in-place; a full navigation to `/` is simpler and more correct for a "leave this page" action.
   - A plain `<a href="/">` is the most robust, accessible, and SSR-friendly approach.

2. **Cancel button placed before the submit button** (left-to-right visual order)
   - Standard form convention: secondary action on the left, primary action on the right.
   - Consistent with OS-level dialog patterns (Cancel / OK).

3. **No backend changes**
   - The cancel action is entirely client-side navigation. No handler, route, or state changes needed.

## Risks / Trade-offs

- **Lost unsaved input**: If the operator has typed a value and clicks Cancel, the input is discarded. This is expected behavior for a cancel action — users understand this convention. No mitigation needed.

- **Existing snapshot tests need updating**: The test `form_templates_do_not_contain_back_link_or_confirmed_block` currently asserts that the forms contain neither "Back to dashboard" text nor a `class="confirmed"` block. The cancel link will introduce the text "Cancel" and an `<a>` tag, so this test must be updated (or replaced with a new assertion that the cancel link points to `/`).

- **Existing snapshot snapshots**: `target_form_with_current_target_snapshot`, `target_form_with_error_snapshot`, `brew_form_with_current_brew_snapshot`, and `brew_form_with_error_snapshot` will all need their snapshots updated via `INSTA_UPDATE=always`.

## Migration Plan

1. Edit `target_form.html` — add Cancel link before the submit button.
2. Edit `brew_form.html` — add Cancel link before the submit button.
3. Update tests:
   - Modify `form_templates_do_not_contain_back_link_or_confirmed_block` to check for Cancel link presence and correct `/` target.
   - Update insta snapshots.
4. Run `cargo test` to confirm everything passes.
5. Visual check by running the app locally and verifying both forms.
