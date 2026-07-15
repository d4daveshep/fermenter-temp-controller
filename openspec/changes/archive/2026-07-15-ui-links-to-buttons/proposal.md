## Why

The dashboard uses plain text links (`<a>` tags) for navigation to the target-temperature and brew-identifier forms. Buttons provide a clearer call-to-action and are more natural for navigation on touch/mobile devices. Additionally, after successfully updating the target temperature or brew identifier, the user is left on a confirmation page with a redundant "Back to dashboard" link — the system should redirect them back to the dashboard automatically, removing unnecessary navigation steps.

## What Changes

- Replace the "Set target temperature" and "Change brew identifier" links on the dashboard with styled `<button>` elements that navigate to the same routes.
- On successful form submission (`POST /target`, `POST /brew`), redirect the browser back to the dashboard (`/`) instead of re-rendering the form page with a confirmation message and a "Back to dashboard" link.
- Remove the "Back to dashboard" `<a>` link from both form pages.
- Retain the error-redisplay behavior: invalid submissions still re-render the form page with an error message (no redirect).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `web-dashboard`: The target-setpoint form and brew-identifier form responses change from re-rendering with a confirmation to redirecting on success. The form templates lose their confirmation message display and their "Back to dashboard" links.

## Impact

- Templates: `dashboard.html`, `target_form.html`, `brew_form.html`
- Handlers: `set_target` and `set_brew` in `fermenter/src/web/handlers.rs` — return type changes to produce an HTTP redirect on success
- Snapshot tests: HTML snapshots for the form pages will need updating
