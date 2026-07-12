## ADDED Requirements

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
