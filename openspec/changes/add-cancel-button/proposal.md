## Why

The target-temperature and brew-identifier forms have no way to exit without submitting a change. An operator who navigates to either form must either submit (possibly with an unintended value) or use the browser's back button. A cancel button on each form gives a clear, one-click path back to the dashboard.

## What Changes

- Add a **Cancel** button (styled as a secondary action) to the target-temperature form (`target_form.html`) that navigates to `/` without submitting the form.
- Add a **Cancel** button (styled as a secondary action) to the brew-identifier form (`brew_form.html`) that navigates to `/` without submitting the form.
- No new routes, no new handlers, no changes to validation or persistence logic.

## Capabilities

### New Capabilities

*(none)*

### Modified Capabilities

*(none — purely a UI affordance; no spec-level requirement changes)*

## Impact

- **Templates**: `target_form.html` and `brew_form.html` — each gets a Cancel button with a link to `/`.
- **Handlers**: Unchanged.
- **Routes**: Unchanged.
- **Tests**: Snapshot tests for both form templates will need updating (insta accepts). The existing test `form_templates_do_not_contain_back_link_or_confirmed_block` will need to be relaxed or removed since the cancel button replaces the previous assertion.
