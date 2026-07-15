# web-dashboard

## Purpose

The web-facing dashboard that lets an operator view the current brew state
(latest reading and target) over HTTP, with a live-polled fragment so the
displayed state stays current without a full page reload.

## Requirements

### Requirement: Render a current-state dashboard page

The system SHALL serve an HTML dashboard page over HTTP showing the current
brew identifier and the most recently observed reading (average, min, max,
ambient, target, action, and reason code), or an explicit no-data indication
when no reading has been observed yet. The reason code SHALL be displayed
immediately after the action, so the operator can see the controller's
decision rationale. The page SHALL also display a timestamp
reflecting the server's local wall-clock time at render time, formatted
`DD-MMM-YYYY HH:MM:SS` (e.g. `14-Jul-2026 14:30:45`), so an operator can
distinguish a live server from a stale or frozen view. The timestamp SHALL
render even when no reading has been observed yet.

#### Scenario: Dashboard shows the latest reading

- **WHEN** a browser requests the dashboard page and a reading has been
  observed
- **THEN** the response is an HTML page displaying the current brew
  identifier and the latest reading's average, min, max, ambient, target,
  action, and reason code values

#### Scenario: Dashboard shows a no-data state before any reading arrives

- **WHEN** a browser requests the dashboard page and no reading has been
  observed yet
- **THEN** the response is an HTML page indicating no reading is available
  yet, without error

#### Scenario: Dashboard displays the server's local time

- **WHEN** a browser requests the dashboard page
- **THEN** the response is an HTML page that includes a timestamp reflecting
  the server's local wall-clock time at render time, formatted
  `DD-MMM-YYYY HH:MM:SS`

#### Scenario: Server time is shown before any reading arrives

- **WHEN** a browser requests the dashboard page and no reading has been
  observed yet
- **THEN** the response still includes the server-local-time timestamp, so
  the operator can confirm the server is alive even with no reading data

### Requirement: Serve a live-polled current-state fragment

The system SHALL serve an HTML fragment, at a distinct URL from the full
dashboard page, containing only the current-state markup, so a browser can
periodically re-fetch it and refresh the displayed state without a full page
reload. The fragment SHALL include a timestamp reflecting the server's local
wall-clock time at render time, formatted `DD-MMM-YYYY HH:MM:SS`, so the
displayed time advances on each poll without a full page reload. When a
reading has been observed, the fragment SHALL include the reason code
immediately after the action value.

#### Scenario: Fragment reflects the latest state on each poll

- **WHEN** a browser requests the status fragment after a new reading has
  been observed since the previous request
- **THEN** the fragment reflects the newly observed reading, including its
  reason code

#### Scenario: Fragment content matches the dashboard's embedded status block

- **WHEN** the status fragment is requested directly and the dashboard page
  is requested
- **THEN** the current-state content rendered in both is equivalent for the
  same underlying state, excluding the server-local-time field whose value is
  by design render-time and therefore differs between the two responses

#### Scenario: Dashboard page polls the fragment automatically

- **WHEN** the dashboard page is loaded in a browser
- **THEN** the page is configured to automatically re-request the status
  fragment on an interval, without requiring a manual refresh

#### Scenario: Fragment includes a server-time timestamp that refreshes on each poll

- **WHEN** a browser requests the status fragment
- **THEN** the fragment includes a timestamp reflecting the server's local
  wall-clock time at render time, formatted `DD-MMM-YYYY HH:MM:SS`

- **WHEN** the browser re-requests the status fragment on a subsequent poll
  at least one second later
- **THEN** the timestamp in the new response is greater than or equal to the
  timestamp in the previous response, reflecting that it is recomputed on
  each render

#### Scenario: Fragment shows server time even with no reading

- **WHEN** a browser requests the status fragment and no reading has been
  observed yet
- **THEN** the fragment still includes the server-local-time timestamp

### Requirement: Serve static assets

The system SHALL serve static assets (such as stylesheets) referenced by the
dashboard page over HTTP.

#### Scenario: Stylesheet is retrievable

- **WHEN** a browser requests a static asset referenced by the dashboard page
- **THEN** the system returns the asset's contents

### Requirement: Dashboard rendering is read-only in this capability

The system SHALL NOT expose any HTTP endpoint under the dashboard page or the
live-polled status fragment that mutates controller state (target
temperature, brew identifier, or any other runtime setting); those two
routes only display current state. Mutating routes (such as the
target-setpoint form) are a distinct part of this capability and are exempt
from this requirement.

#### Scenario: No mutating routes are exposed by the dashboard or status fragment

- **WHEN** the web server's routes are enumerated
- **THEN** neither the dashboard page route nor the status fragment route
  accepts a request that changes target temperature, brew identifier, or any
  other runtime setting

### Requirement: Serve a target-setpoint form

The system SHALL serve an HTML form, at a distinct URL from the dashboard
page, that displays the current target temperature and accepts a submission
of a new target temperature.

#### Scenario: Form displays the current target temperature

- **WHEN** an operator requests the target-setpoint form
- **THEN** the response is an HTML form pre-filled with the current target
  temperature

#### Scenario: Valid submission updates the target and confirms

- **WHEN** an operator submits a valid new target temperature through the
  form
- **THEN** the system applies the new target temperature and the response
  confirms the update

#### Scenario: Invalid submission redisplays the form with an error

- **WHEN** an operator submits an invalid target temperature (non-numeric or
  outside the allowed range) through the form
- **THEN** the response redisplays the form with an error message and the
  current target temperature is left unchanged

### Requirement: Serve a brew-identifier form

The system SHALL serve an HTML form, at a distinct URL from the dashboard
page, that displays the current brew identifier and accepts a submission of
a new brew identifier.

#### Scenario: Form displays the current brew identifier

- **WHEN** an operator requests the brew-identifier form
- **THEN** the response is an HTML form pre-filled with the current brew
  identifier

#### Scenario: Valid submission updates the brew identifier and confirms

- **WHEN** an operator submits a valid new brew identifier through the form
- **THEN** the system applies the new brew identifier and the response
  confirms the update

#### Scenario: Invalid submission redisplays the form with an error

- **WHEN** an operator submits an invalid brew identifier (empty or
  whitespace-only) through the form
- **THEN** the response redisplays the form with an error message and the
  current brew identifier is left unchanged
</content>
