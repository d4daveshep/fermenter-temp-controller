# web-dashboard (MODIFIED by change show-git-tag-on-dashboard)

## Purpose

The web-facing dashboard that lets an operator view the current brew state
(latest reading and target) over HTTP, with a live-polled fragment so the
displayed state stays current without a full page reload. **Additionally
displays the application version as a subtitle on all pages.**

## Requirements

### Requirement: Render a current-state dashboard page

The system SHALL serve an HTML dashboard page over HTTP showing the current
brew identifier and the most recently observed reading (average, min, max,
ambient, target, action, and reason code), or an explicit no-data indication
when no reading has been observed yet. The reason code SHALL be displayed
immediately after the action, so the operator can see the controller's
decision rationale. The system SHALL additionally display a human-readable
description of the reason code, derived from a static code-to-text mapping,
in the same cell as the raw code, formatted `<code> — <description>`. The
description SHALL be omitted (leaving only the raw code) when the code is
unrecognized, empty, or an error sentinel such as `RC_ERR`, so that the
operator is never shown fabricated text for a code the mapping does not
cover. The page SHALL also display a timestamp
reflecting the server's local wall-clock time at render time, formatted
`DD-MMM-YYYY HH:MM:SS` (e.g. `14-Jul-2026 14:30:45`), so an operator can
distinguish a live server from a stale or frozen view. The timestamp SHALL
render even when no reading has been observed yet. **The page SHALL also
display the application version as a subtitle under the main "Fermenter"
heading on every page.**

#### Scenario: Dashboard shows the latest reading

- **WHEN** a browser requests the dashboard page and a reading has been
  observed
- **THEN** the response is an HTML page displaying the current brew
  identifier and the latest reading's average, min, max, ambient, target,
  action, and reason code values

#### Scenario: Dashboard shows the reason code description alongside the code

- **WHEN** a browser requests the dashboard page and the latest reading's
  reason code has a known entry in the static code-to-text mapping
- **THEN** the response is an HTML page whose `Reason` cell renders the raw
  code followed by an em-dash and the human-readable description, in the
  form `<code> — <description>`

#### Scenario: Dashboard shows the raw code alone when no description is available

- **WHEN** a browser requests the dashboard page and the latest reading's
  reason code is unrecognized, empty, or an error sentinel such as `RC_ERR`
- **THEN** the response is an HTML page whose `Reason` cell renders only the
  raw code with no trailing em-dash or description, without error

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

#### Scenario: Dashboard displays the application version subtitle

- **WHEN** a browser requests the dashboard page (or any page using the base
  template)
- **THEN** the response is an HTML page that includes the application version
  (from the `version-display` capability) rendered as a subtitle beneath the
  main `Fermenter` heading, styled with the `.version` CSS class

### Requirement: Serve a live-polled current-state fragment

The system SHALL serve an HTML fragment, at a distinct URL from the full
dashboard page, containing only the current-state markup, so a browser can
periodically re-fetch it and refresh the displayed state without a full page
reload. The fragment SHALL include a timestamp reflecting the server's local
wall-clock time at render time, formatted `DD-MMM-YYYY HH:MM:SS`, so the
displayed time advances on each poll without a full page reload. When a
reading has been observed, the fragment SHALL include the reason code
immediately after the action value, together with its human-readable
description derived from the same static code-to-text mapping used by the
dashboard page, formatted `<code> — <description>`. The description SHALL
be omitted (leaving only the raw code) when the code is unrecognized, empty,
or an error sentinel such as `RC_ERR`. The dashboard page SHALL poll this
fragment every 10 seconds.

#### Scenario: Fragment reflects the latest state on each poll

- **WHEN** a browser requests the status fragment after a new reading has
  been observed since the previous request
- **THEN** the fragment reflects the newly observed reading, including its
  reason code and, when the code has a known mapping entry, the
  human-readable description rendered in the same form as the dashboard page

#### Scenario: Fragment shows the raw code alone when no description is available

- **WHEN** a browser requests the status fragment and the latest reading's
  reason code is unrecognized, empty, or an error sentinel such as `RC_ERR`
- **THEN** the fragment renders only the raw code with no trailing em-dash
  or description, without error

#### Scenario: Fragment content matches the dashboard's embedded status block

- **WHEN** the status fragment is requested directly and the dashboard page
  is requested
- **THEN** the current-state content rendered in both is equivalent for the
  same underlying state, including the reason code and any description,
  excluding the server-local-time field whose value is by design render-time
  and therefore differs between the two responses

#### Scenario: Dashboard page polls the fragment every 10 seconds

- **WHEN** the dashboard page is loaded in a browser
- **THEN** the page is configured to automatically re-request the status
  fragment every 10 seconds, without requiring a manual refresh

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

### Requirement: Display a selectable temperature-history chart

The dashboard SHALL display a server-rendered temperature-history chart for
the active brew. The chart SHALL show the average fermenter, ambient, and
target temperature series, label its X axis `Time` and its Y axis
`Temperature`, and scale the Y axis to the range of all values displayed for
the selected window. Each series SHALL use a distinct color. The chart SHALL
include a legend that identifies each series by its name and matching color.
The chart SHALL render major X- and Y-axis gridlines and position samples on
the X axis relative to the full selected window. The legend SHALL render below
the X axis, outside the plotting area and aligned to the right. Time-axis ticks
SHALL use calendar-aligned, round intervals appropriate to the selected window.
Axis tick labels SHALL be rendered in sans-serif at 14px. Each plotted
temperature series SHALL be drawn with a 2px stroke width. Legend swatch
lines SHALL match the plotted series stroke width of 2px.

#### Scenario: Chart displays all three temperature series

- **WHEN** the active brew has temperature history in the selected window
- **THEN** the dashboard displays a server-rendered chart with labeled
  average fermenter, ambient, and target series

#### Scenario: Chart labels axes and identifies series colors

- **WHEN** the active brew has temperature history in the selected window
- **THEN** the chart labels its axes `Time` and `Temperature`, renders each
  temperature series in a distinct color, and includes a legend that maps
  each color to its series name

#### Scenario: Chart provides a conventional plotting grid

- **WHEN** the active brew has temperature history in the selected window
- **THEN** the chart renders labeled major ticks and gridlines on both axes,
  and positions each sample by its timestamp within the full selected window

#### Scenario: Chart legend and time ticks stay readable

- **WHEN** the active brew has temperature history in the selected window
- **THEN** the legend renders outside the plotting area below and right of the
  X axis, and time-axis ticks use round calendar intervals rather than offsets
  from the moment the chart was requested

#### Scenario: Chart Y axis follows displayed data

- **WHEN** the selected window contains temperature history
- **THEN** the chart's Y axis covers the range of the average fermenter,
  ambient, and target values displayed in that chart

#### Scenario: Constant temperatures render a valid chart

- **WHEN** every temperature value in the selected window is the same
- **THEN** the dashboard renders a chart with a non-zero Y-axis span

#### Scenario: Chart reports no history for an empty window

- **WHEN** the active brew has no complete temperature history in the
  selected window
- **THEN** the chart area displays an explicit no-history indication without
  error

#### Scenario: Axis tick labels render at 14px

- **WHEN** the active brew has temperature history in the selected window
- **THEN** the chart's X- and Y-axis tick-value labels are rendered in a
  sans-serif font at 14px

#### Scenario: Series and legend lines render at 2px stroke width

- **WHEN** the active brew has temperature history in the selected window
- **THEN** each plotted temperature-series line is drawn with a 2px stroke
  width, and every legend swatch line matches that 2px stroke width

### Requirement: Select and refresh fixed chart windows

The dashboard SHALL offer exactly these selectable chart windows: last 5
minutes, 15 minutes, 1 hour, 3 hours, 6 hours, 12 hours, 24 hours, 3 days, 7
days, and 14 days. The dashboard SHALL request a server-rendered chart for the
selected window when the page loads, when the selection changes, and every 10
seconds without a full page reload. Chart requests SHALL be read-only.

#### Scenario: Selecting a window refreshes the chart

- **WHEN** an operator selects one of the supported chart windows
- **THEN** the dashboard requests and displays the chart for that window
  without a full page reload

#### Scenario: Chart polling preserves the selected window

- **WHEN** the dashboard periodically refreshes the chart after an operator
  has selected a supported window
- **THEN** the request and replacement chart use that selected window

#### Scenario: Dashboard requests the default chart window

- **WHEN** an operator loads the dashboard without choosing a chart window
- **THEN** the dashboard requests the last-15-minutes chart

#### Scenario: Invalid chart window does not create an unbounded query

- **WHEN** a chart request contains an unsupported or missing window value
- **THEN** the server uses the last-15-minutes window

#### Scenario: Chart endpoint cannot mutate controller state

- **WHEN** the web server's chart route receives a non-GET request
- **THEN** the request does not change target temperature, brew identifier, or
  any other runtime setting

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
of a new target temperature. On successful submission, the system SHALL
redirect the browser back to the dashboard page.

#### Scenario: Form displays the current target temperature

- **WHEN** an operator requests the target-setpoint form
- **THEN** the response is an HTML form pre-filled with the current target
  temperature

#### Scenario: Valid submission updates the target and redirects to dashboard

- **WHEN** an operator submits a valid new target temperature through the
  form
- **THEN** the system applies the new target temperature and redirects the
  browser to the dashboard page

#### Scenario: Invalid submission redisplays the form with an error

- **WHEN** an operator submits an invalid target temperature (non-numeric or
  outside the allowed range) through the form
- **THEN** the response redisplays the form with an error message and the
  current target temperature is left unchanged

### Requirement: Serve a brew-identifier form

The system SHALL serve an HTML form, at a distinct URL from the dashboard
page, that displays the current brew identifier and accepts a submission of
a new brew identifier. On successful submission, the system SHALL redirect
the browser back to the dashboard page.

#### Scenario: Form displays the current brew identifier

- **WHEN** an operator requests the brew-identifier form
- **THEN** the response is an HTML form pre-filled with the current brew
  identifier

#### Scenario: Valid submission updates the brew identifier and redirects to dashboard

- **WHEN** an operator submits a valid new brew identifier through the form
- **THEN** the system applies the new brew identifier and redirects the
  browser to the dashboard page

#### Scenario: Invalid submission redisplays the form with an error

- **WHEN** an operator submits an invalid brew identifier (empty or
  whitespace-only) through the form
- **THEN** the response redisplays the form with an error message and the
  current brew identifier is left unchanged