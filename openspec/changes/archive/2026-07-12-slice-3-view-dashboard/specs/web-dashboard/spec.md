## ADDED Requirements

### Requirement: Render a current-state dashboard page

The system SHALL serve an HTML dashboard page over HTTP showing the current
brew identifier and the most recently observed reading (average, min, max,
ambient, target, and action), or an explicit no-data indication when no
reading has been observed yet.

#### Scenario: Dashboard shows the latest reading

- **WHEN** a browser requests the dashboard page and a reading has been
  observed
- **THEN** the response is an HTML page displaying the current brew
  identifier and the latest reading's average, min, max, ambient, target, and
  action values

#### Scenario: Dashboard shows a no-data state before any reading arrives

- **WHEN** a browser requests the dashboard page and no reading has been
  observed yet
- **THEN** the response is an HTML page indicating no reading is available
  yet, without error

### Requirement: Serve a live-polled current-state fragment

The system SHALL serve an HTML fragment, at a distinct URL from the full
dashboard page, containing only the current-state markup, so a browser can
periodically re-fetch it and refresh the displayed state without a full page
reload.

#### Scenario: Fragment reflects the latest state on each poll

- **WHEN** a browser requests the status fragment after a new reading has
  been observed since the previous request
- **THEN** the fragment reflects the newly observed reading

#### Scenario: Fragment content matches the dashboard's embedded status block

- **WHEN** the status fragment is requested directly and the dashboard page
  is requested
- **THEN** the current-state content rendered in both is equivalent for the
  same underlying state

#### Scenario: Dashboard page polls the fragment automatically

- **WHEN** the dashboard page is loaded in a browser
- **THEN** the page is configured to automatically re-request the status
  fragment on an interval, without requiring a manual refresh

### Requirement: Serve static assets

The system SHALL serve static assets (such as stylesheets) referenced by the
dashboard page over HTTP.

#### Scenario: Stylesheet is retrievable

- **WHEN** a browser requests a static asset referenced by the dashboard page
- **THEN** the system returns the asset's contents

### Requirement: Dashboard rendering is read-only in this capability

The system SHALL NOT expose any HTTP endpoint under this capability that
mutates controller state (target temperature, brew identifier, or any other
runtime setting); the dashboard and its fragment only display current state.

#### Scenario: No mutating routes are exposed

- **WHEN** the web server's routes are enumerated
- **THEN** none of the dashboard-related routes accept a request that changes
  target temperature, brew identifier, or any other runtime setting
