## ADDED Requirements

### Requirement: Query bounded temperature history
The system SHALL retrieve timestamped average fermenter, ambient, and target
temperature samples for a specified brew and inclusive time range. The query
SHALL return only samples with all three temperature values and SHALL support
an aggregation interval that bounds the number of returned samples.

#### Scenario: Querying a brew's history returns complete temperature samples
- **WHEN** a caller requests a time range containing persisted average,
  ambient, and target values for a brew
- **THEN** the system returns timestamped samples containing all three values
  within that range

#### Scenario: Aggregated query bounds long-range history
- **WHEN** a caller requests a long time range with an aggregation interval
- **THEN** the system returns aggregated samples at that interval rather than
  every raw reading

#### Scenario: No data in the requested range returns no samples
- **WHEN** a caller requests a time range for a brew with no complete
  temperature samples in that range
- **THEN** the system returns an empty result without error

#### Scenario: History remains isolated by brew identifier
- **WHEN** readings exist for multiple brew identifiers and a caller queries
  the history of one brew
- **THEN** the result contains only samples persisted under that brew
