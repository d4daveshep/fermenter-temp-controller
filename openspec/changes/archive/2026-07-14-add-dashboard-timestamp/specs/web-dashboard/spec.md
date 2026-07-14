## MODIFIED Requirements

### Requirement: Render a current-state dashboard page

The system SHALL serve an HTML dashboard page over HTTP showing the current
brew identifier and the most recently observed reading (average, min, max,
ambient, target, and action), or an explicit no-data indication when no
reading has been observed yet. The page SHALL also display a timestamp
reflecting the server's local wall-clock time at render time, formatted
`DD-MMM-YYYY HH:MM:SS` (e.g. `14-Jul-2026 14:30:45`), so an operator can
distinguish a live server from a stale or frozen view. The timestamp SHALL
render even when no reading has been observed yet.

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
displayed time advances on each poll without a full page reload.

#### Scenario: Fragment reflects the latest state on each poll

- **WHEN** a browser requests the status fragment after a new reading has
  been observed since the previous request
- **THEN** the fragment reflects the newly observed reading

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
