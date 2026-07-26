//! Ports `arduino/TempController/TemperatureReadings.h`/`.cpp`.

/// Tracks an exponential moving average (EMA) of temperature readings over a
/// configurable window `n`, along with the min/max readings seen and a count
/// of updates.
///
/// `new average = (old average * (n-1) + new value) / n`
pub struct TemperatureReadings {
    window: u32,
    average: f64,
    latest: f64,
    count: u64,
    minimum: f64,
    maximum: f64,
}

impl TemperatureReadings {
    pub fn new(window: u32) -> Self {
        Self {
            window,
            average: 0.0,
            latest: 0.0,
            count: 0,
            minimum: 1000.0,
            maximum: -1000.0,
        }
    }

    pub fn clear(&mut self) {
        let window = self.window;
        *self = Self::new(window);
    }

    pub fn window(&self) -> u32 {
        self.window
    }

    pub fn set_window(&mut self, window: u32) {
        self.window = window;
    }

    pub fn average(&self) -> f64 {
        self.average
    }

    pub fn latest(&self) -> f64 {
        self.latest
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn minimum(&self) -> f64 {
        self.minimum
    }

    pub fn maximum(&self) -> f64 {
        self.maximum
    }

    /// Seeds the average with `initial_average`, but only before the first
    /// reading has been recorded — a no-op after that (matches
    /// `setInitialAverageTemperature`'s silent-ignore behaviour).
    pub fn set_initial_average(&mut self, initial_average: f64) {
        if self.count < 1 {
            self.average = initial_average;
        }
    }

    pub fn update(&mut self, new_reading: f64) {
        self.latest = new_reading;
        let n = f64::from(self.window);
        self.average = (self.average * (n - 1.0) + new_reading) / n;
        self.count += 1;
        if new_reading < self.minimum {
            self.minimum = new_reading;
        }
        if new_reading > self.maximum {
            self.maximum = new_reading;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fixed sequence from Test_TemperatureReadings.cpp's `EMA`/
    // `ExponentialMovingAverageOfTemperatureReadings` tests: the first 10
    // values Arduino's `randomSeed(1); randomSeed(0);` sequence produced,
    // recorded verbatim in that file's comments so the exact PRNG doesn't
    // need porting. True average 18.717, expected EMA 12.14163.
    const KNOWN_SEQUENCE: [f64; 10] = [
        22.49, 18.58, 15.72, 19.78, 18.09, 15.65, 17.42, 22.03, 22.29, 15.12,
    ];

    #[test]
    fn seed_sets_initial_average() {
        let mut readings = TemperatureReadings::new(10);
        assert_eq!(readings.average(), 0.0);

        readings.set_initial_average(12.34);
        assert_eq!(readings.average(), 12.34);
    }

    #[test]
    fn seed_ignored_after_first_reading() {
        let mut readings = TemperatureReadings::new(10);
        readings.set_initial_average(12.34);
        readings.update(13.46);

        readings.set_initial_average(23.45);

        assert_eq!(readings.average(), (12.34 * 9.0 + 13.46) / 10.0);
    }

    #[test]
    fn ema_matches_expected_value_over_known_sequence() {
        let mut readings = TemperatureReadings::new(10);
        for value in KNOWN_SEQUENCE {
            readings.update(value);
        }
        assert_eq!(readings.count(), 10);
        assert!((readings.average() - 12.14163).abs() < 1e-5);
    }

    #[test]
    fn min_and_max_tracked_correctly() {
        let mut readings = TemperatureReadings::new(10);
        assert_eq!(readings.minimum(), 1000.0);
        assert_eq!(readings.maximum(), -1000.0);

        for value in KNOWN_SEQUENCE {
            readings.update(value);
        }

        assert_eq!(readings.minimum(), 15.12);
        assert_eq!(readings.maximum(), 22.49);
    }

    #[test]
    fn count_increments_with_each_update() {
        let mut readings = TemperatureReadings::new(10);
        for (i, value) in KNOWN_SEQUENCE.iter().enumerate() {
            readings.update(*value);
            assert_eq!(readings.count(), i as u64 + 1);
        }
    }
}
