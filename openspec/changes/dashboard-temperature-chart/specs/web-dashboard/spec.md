## ADDED Requirements

### Requirement: Display a selectable temperature-history chart
The dashboard SHALL display a server-rendered temperature-history chart for
the active brew. The chart SHALL show the average fermenter, ambient, and
target temperature series, label its X axis `Time` and its Y axis
`Temperature`, and scale the Y axis to the range of all values displayed for
the selected window. Each series SHALL use a distinct color. The chart SHALL
include a legend that identifies each series by its name and matching color.

#### Scenario: Chart displays all three temperature series
- **WHEN** the active brew has temperature history in the selected window
- **THEN** the dashboard displays a server-rendered chart with labeled
  average fermenter, ambient, and target series

#### Scenario: Chart labels axes and identifies series colors
- **WHEN** the active brew has temperature history in the selected window
- **THEN** the chart labels its axes `Time` and `Temperature`, renders each
  temperature series in a distinct color, and includes a legend that maps
  each color to its series name

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

### Requirement: Select and refresh fixed chart windows
The dashboard SHALL offer exactly these selectable chart windows: last 15
minutes, 1 hour, 3 hours, 6 hours, 12 hours, 24 hours, 3 days, 7 days, and 14
days. The dashboard SHALL request a server-rendered chart for the selected
window when the page loads, when the selection changes, and at a periodic
interval without a full page reload. Chart requests SHALL be read-only.

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
