# firmware-logic

## Purpose

The pure, hardware-free controller logic that runs identically on the embedded
target and under `cargo test` on a developer's machine. Covers temperature
decision rules, exponential moving average tracking, and the action/decision
value types. This capability has no I/O dependencies and is fully unit-testable
without an embedded toolchain or physical hardware.

## Requirements

### Requirement: Action and decision types

The system SHALL define an `Action` enum with variants `Rest`, `Heat`, `Cool`,
and `Error`, and a `Decision` struct that pairs an `Action` with a static
reason code string.

#### Scenario: Decision stores action and reason code

- **WHEN** a `Decision` is constructed with an `Action` and a reason code string
- **THEN** both are retrievable unchanged via accessor methods

#### Scenario: Decision carries a display string for each action

- **WHEN** a `Decision`'s action is inspected
- **THEN** the system can produce a human-readable string (`"Rest"`, `"Heat"`,
  `"Cool"`, `"Error"`, `"No Action"`) matching the strings the host's
  `reason_code.rs` and templates expect

#### Scenario: An unmade decision is distinguishable from a made one

- **WHEN** a `Decision` has not yet had its action set (initial/cleared state)
- **THEN** `is_made()` returns `false`; once set to any action other than the
  sentinel, `is_made()` returns `true`

#### Scenario: A decision can be cleared back to unmade state

- **WHEN** `clear()` is called on a `Decision` that was previously made
- **THEN** `is_made()` returns `false` and the action returns to the sentinel
  value

### Requirement: Natural drift classification

The system SHALL classify the relationship between ambient temperature and
fermenter temperature as `NaturalHeating` (ambient >= fermenter) or
`NaturalCooling` (ambient < fermenter), determining which direction the
fermenter temperature will drift without active intervention.

#### Scenario: Natural heating when ambient equals fermenter

- **WHEN** ambient temperature equals the fermenter temperature
- **THEN** natural drift is classified as `NaturalHeating`

#### Scenario: Natural heating when ambient is above fermenter

- **WHEN** ambient temperature is strictly above the fermenter temperature
- **THEN** natural drift is classified as `NaturalHeating`

#### Scenario: Natural cooling when ambient is below fermenter

- **WHEN** ambient temperature is strictly below the fermenter temperature
- **THEN** natural drift is classified as `NaturalCooling`

### Requirement: Failsafe boundary enforcement

The system SHALL force heating when the fermenter temperature falls below the
failsafe minimum (`target - 2 × range`) and force cooling when it rises above
the failsafe maximum (`target + 2 × range`), regardless of current action or
ambient conditions (RC1/RC6 and RC5/RC10).

#### Scenario: Below failsafe forces heat regardless of current action and ambient

- **WHEN** the fermenter temperature is below `target - 2 × range`
- **THEN** the decision is `Heat` with reason code `"RC1"`, for any combination
  of current action (`Rest`, `Heat`, `Cool`) and ambient temperature

#### Scenario: Above failsafe forces cool regardless of current action and ambient

- **WHEN** the fermenter temperature is above `target + 2 × range`
- **THEN** the decision is `Cool` with reason code `"RC5"`, for any combination
  of current action and ambient temperature

### Requirement: Cooling overrun adjustment

The system SHALL stop active cooling early when the fermenter temperature falls
below `target_range_min + cooling_overrun_adjustment` (default `0.2`°C) while
already cooling, to prevent overshooting the bottom of the target range.

#### Scenario: Cooling overrun stops cooling with natural heating (RC2.2)

- **WHEN** current action is `Cool`, fermenter temperature is below
  `target_range_min + 0.2`, and natural drift is `NaturalHeating`
- **THEN** the decision is `Rest` with reason code `"RC2.2"`

#### Scenario: Cooling overrun switches to heating with natural cooling (RC7.2)

- **WHEN** current action is `Cool`, fermenter temperature is below
  `target_range_min + 0.2`, and natural drift is `NaturalCooling`
- **THEN** the decision is `Heat` with reason code `"RC7.2"`

### Requirement: Temperature-below-range decisions (RC2 and RC7 groups)

The system SHALL apply the correct action when the fermenter temperature is
below the target range (`target - range`) based on the current action and
natural drift direction, per the following rules (cooling-overrun cases handled
separately above).

#### Scenario: REST when below range with natural heating and current REST (RC2.1)

- **WHEN** fermenter is below target range, drift is `NaturalHeating`, current
  action is `Rest`
- **THEN** decision is `Rest` with reason code `"RC2.1"`

#### Scenario: REST when below range with natural heating and current HEAT (RC2.3)

- **WHEN** fermenter is below target range, drift is `NaturalHeating`, current
  action is `Heat`
- **THEN** decision is `Rest` with reason code `"RC2.3"`

#### Scenario: HEAT when below range with natural cooling and current REST (RC7.1)

- **WHEN** fermenter is below target range, drift is `NaturalCooling`, current
  action is `Rest`
- **THEN** decision is `Heat` with reason code `"RC7.1"`

#### Scenario: HEAT when below range with natural cooling and current HEAT (RC7.3)

- **WHEN** fermenter is below target range, drift is `NaturalCooling`, current
  action is `Heat`
- **THEN** decision is `Heat` with reason code `"RC7.3"`

### Requirement: Temperature-in-range decisions (RC3 and RC8 groups)

The system SHALL apply the correct action when the fermenter temperature is
within the target range (`target ± range`) based on the current action and
natural drift direction.

#### Scenario: REST when in range with natural heating and current REST (RC3.1)

- **WHEN** fermenter is in target range, drift is `NaturalHeating`, current
  action is `Rest`
- **THEN** decision is `Rest` with reason code `"RC3.1"`

#### Scenario: COOL when in range with natural heating and current COOL (RC3.2)

- **WHEN** fermenter is in target range, drift is `NaturalHeating`, current
  action is `Cool`
- **THEN** decision is `Cool` with reason code `"RC3.2"`

#### Scenario: REST when in range with natural heating and current HEAT (RC3.3)

- **WHEN** fermenter is in target range, drift is `NaturalHeating`, current
  action is `Heat`
- **THEN** decision is `Rest` with reason code `"RC3.3"`

#### Scenario: REST when in range with natural cooling and current REST (RC8.1)

- **WHEN** fermenter is in target range, drift is `NaturalCooling`, current
  action is `Rest`
- **THEN** decision is `Rest` with reason code `"RC8.1"`

#### Scenario: REST when in range with natural cooling and current COOL (RC8.2)

- **WHEN** fermenter is in target range, drift is `NaturalCooling`, current
  action is `Cool`
- **THEN** decision is `Rest` with reason code `"RC8.2"`

#### Scenario: HEAT when in range with natural cooling and current HEAT (RC8.3)

- **WHEN** fermenter is in target range, drift is `NaturalCooling`, current
  action is `Heat`
- **THEN** decision is `Heat` with reason code `"RC8.3"`

### Requirement: Temperature-above-range decisions (RC4 and RC9 groups)

The system SHALL apply the correct action when the fermenter temperature is
above the target range based on the current action and natural drift direction.

#### Scenario: COOL when above range with natural heating and current REST (RC4.1)

- **WHEN** fermenter is above target range, drift is `NaturalHeating`, current
  action is `Rest`
- **THEN** decision is `Cool` with reason code `"RC4.1"`

#### Scenario: COOL when above range with natural heating and current COOL (RC4.2)

- **WHEN** fermenter is above target range, drift is `NaturalHeating`, current
  action is `Cool`
- **THEN** decision is `Cool` with reason code `"RC4.2"`

#### Scenario: COOL when above range with natural heating and current HEAT (RC4.3)

- **WHEN** fermenter is above target range, drift is `NaturalHeating`, current
  action is `Heat`
- **THEN** decision is `Cool` with reason code `"RC4.3"`

#### Scenario: REST when above range with natural cooling and current REST (RC9.1)

- **WHEN** fermenter is above target range, drift is `NaturalCooling`, current
  action is `Rest`
- **THEN** decision is `Rest` with reason code `"RC9.1"`

#### Scenario: COOL when above range with natural cooling and current COOL (RC9.2)

- **WHEN** fermenter is above target range, drift is `NaturalCooling`, current
  action is `Cool`
- **THEN** decision is `Cool` with reason code `"RC9.2"`

#### Scenario: REST when above range with natural cooling and current HEAT (RC9.3)

- **WHEN** fermenter is above target range, drift is `NaturalCooling`, current
  action is `Heat`
- **THEN** decision is `Rest` with reason code `"RC9.3"`

### Requirement: Target temperature mutability

The system SHALL allow the target temperature to be updated at runtime, with
the change taking effect on the next call to `make_action_decision`.

#### Scenario: Updated target temperature is used on next decision

- **WHEN** `set_target_temp` is called with a new value
- **THEN** subsequent calls to `make_action_decision` use the new target for all
  range and failsafe calculations

### Requirement: Exponential moving average temperature tracking

The system SHALL compute an exponential moving average (EMA) of temperature
readings using a configurable window size `n`, track the minimum and maximum
readings seen, and count total readings received.

The EMA formula is: `new_average = (old_average × (n-1) + new_value) / n`

#### Scenario: EMA initialises correctly from a seed value

- **WHEN** `set_initial_average` is called before any readings have been added
- **THEN** the current average equals the seed value and the reading count is 0

#### Scenario: Seed is ignored after readings have started

- **WHEN** `set_initial_average` is called after one or more readings have
  already been added
- **THEN** the average is unchanged (seeding is only valid before first reading)

#### Scenario: EMA converges correctly over multiple readings

- **WHEN** a sequence of known temperature values is fed into the tracker
- **THEN** the average after all readings matches the expected EMA value
  computed by the formula (regression test mirroring the Arduino AUnit suite)

#### Scenario: Minimum and maximum are tracked correctly

- **WHEN** temperature readings are added
- **THEN** `minimum()` returns the lowest value seen and `maximum()` returns the
  highest value seen across all readings

#### Scenario: Reading count increments with each update

- **WHEN** `n` readings have been added
- **THEN** `count()` returns `n`
