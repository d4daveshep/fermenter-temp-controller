## Context

The dashboard currently uses bare `<a href>` links for navigation to the two mutating forms (`/target` and `/brew`). Both forms, after a successful POST, re-render the same page with a confirmation message and a "Back to dashboard" link at the bottom. Invalid submissions also re-render the same page, with an error message — this behavior is correct and must be preserved.

The template engine is MiniJinja (server-side), and the pages extend `base.html`. There is no custom client-side JavaScript; HTMX is available for future use but is not needed for this change.

## Goals / Non-Goals

**Goals:**
- Replace dashboard navigation links with `<button>` elements inside `<form method="get">` wrappers for pure-HTML navigation
- On successful `POST /target`, respond with an HTTP 303 redirect to `/` instead of re-rendering the form
- On successful `POST /brew`, respond with an HTTP 303 redirect to `/` instead of re-rendering the form
- Remove the "Back to dashboard" link from both form templates
- Remove the confirmation message display logic from both form templates (it becomes dead code)

**Non-Goals:**
- No styling changes beyond what is needed for basic button appearance
- No HTMX-powered inline form submission (forms remain full-page POST)
- No change to error handling flow (invalid input still re-renders the form with an error)
- No change to header/navigation or base template layout

## Decisions

### 1. Use `<form method="get">` wrappers for button navigation

Each dashboard button is wrapped in a `<form method="get" action="<url>">` element. A `<button type="submit">` inside a GET form submits the form as a GET request to the action URL, which navigates the browser without any JavaScript. This is pure HTML and works in any browser, including those with JS disabled or strict content-security policies.

### 2. Use HTTP 303 (See Other) for redirects after successful POST

A 303 redirect tells the browser to `GET` the dashboard after a successful POST. This follows the Post/Redirect/Get (PRG) pattern and avoids the browser re-submitting the form on refresh. Axum provides `axum::response::Redirect` for this.

**Alternative considered:** 302 Found. Rejected because 303 explicitly requires `GET` on the redirect target, which is more semantically correct after a state-changing POST.

### 3. Render forms without confirmation state

Since successful submissions now redirect, the `confirmed` field in `TargetFormContext` and `BrewFormContext` become dead code. The templates lose their `{% if confirmed %}` blocks. The `confirmed` field and `TargetFormContext.confirmed`/`BrewFormContext.confirmed` are removed entirely to avoid dead-code confusion.

**Alternative considered:** Keep the confirmed field for potential HTMX-based inline submission later. Rejected — YAGNI, and the field can be re-added if/when inline submission is implemented.

### 4. Handler return type changes

`set_target` and `set_brew` handlers change their return type from `Result<Html<String>>` to `Result<Response>` (using `axum::response::Response`) since they need to return either a redirect or an HTML page depending on validation outcome. On error, they still render and return the form HTML. On success, they return `Redirect::to("/")`.

## Risks / Trade-offs

- **[Risk] Loss of inline confirmation feedback** → Mitigation: After redirect, the dashboard reloads with the updated target/brew_id value displayed, which serves as implicit confirmation. Operators can see the change immediately.
- **[Risk] Snapshot test breakage** → Mitigation: Update snapshot tests with `INSTA_UPDATE=always cargo test` after implementing template and handler changes.
