use super::adjustments::TimeAdjustment;
use super::high_altitude_rule::HighLatitudeRule;
use super::madhab::Madhab;
use super::method::Method;
use super::polar::PolarEstimation;
use super::prayer::Prayer;
use super::rounding::Rounding;
use super::shafaq::Shafaq;

/// Night-portion fractions applied by high-latitude estimation rules,
/// `fajr` and `isha` as fractions of the night.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct NightPortions {
    pub fajr: f64,
    pub isha: f64,
}

/// Settings that determine prayer time calculation:
/// angles, method, madhab, high-latitude rule, rounding, polar estimation, and adjustments.
///
/// Use [`Configuration`] to build a `Parameters` value ergonomically.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Parameters {
    pub method: Method,
    pub fajr_angle: f64,
    pub maghrib_angle: f64,
    pub isha_angle: f64,
    pub isha_interval: i32,
    pub madhab: Madhab,
    /// Estimation rule for Fajr/Isha when the sun does not reach the
    /// method's angle. Ignored by [`Method::MoonsightingCommittee`],
    /// which carries its own high-latitude rules.
    pub high_latitude_rule: HighLatitudeRule,
    /// Estimation strategy for latitudes where the sun may not rise or
    /// set. `None` uses the true latitude and surfaces missing events as
    /// errors from [`crate::schedule::PrayerTimes::try_new`]. Ignored by
    /// [`Method::MoonsightingCommittee`], which resolves high-latitude
    /// days through its own Zone C rules.
    pub polar_estimation: Option<PolarEstimation>,
    pub adjustments: TimeAdjustment,
    pub method_adjustments: TimeAdjustment,
    pub rounding: Rounding,
    pub shafaq: Shafaq,
}

impl Parameters {
    /// Create [`Parameters`] from Fajr and Isha angles, with defaults
    /// for all other fields. Prefer [`Configuration`] for building.
    #[must_use]
    pub fn new(fajr_angle: f64, isha_angle: f64) -> Parameters {
        Parameters {
            fajr_angle,
            maghrib_angle: 0.0,
            isha_angle,
            method: Method::Other,
            isha_interval: 0,
            madhab: Madhab::Shafi,
            high_latitude_rule: HighLatitudeRule::MiddleOfTheNight,
            polar_estimation: None,
            adjustments: TimeAdjustment::default(),
            method_adjustments: TimeAdjustment::default(),
            rounding: Rounding::Nearest,
            shafaq: Shafaq::General,
        }
    }

    /// Return the night-portion fractions ([`NightPortions`]) determined by
    /// the active [`HighLatitudeRule`].
    ///
    /// - `MiddleOfTheNight` → `{ fajr: 0.5, isha: 0.5 }`
    /// - `SeventhOfTheNight` → `{ fajr: 1/7, isha: 1/7 }`
    /// - `TwilightAngle` → `{ fajr: fajr_angle/60, isha: isha_angle/60 }`
    /// - `Recommended` → resolves to MiddleOfTheNight or LRE in [`crate::schedule::PrayerTimes::try_new`]
    /// - `LocalRelativeEstimation` → resolves internally via [`crate::schedule::PrayerTimes::try_new`]
    pub(crate) fn night_portions(&self) -> NightPortions {
        match self.high_latitude_rule {
            HighLatitudeRule::MiddleOfTheNight => NightPortions {
                fajr: 1.0 / 2.0,
                isha: 1.0 / 2.0,
            },
            HighLatitudeRule::SeventhOfTheNight => NightPortions {
                fajr: 1.0 / 7.0,
                isha: 1.0 / 7.0,
            },
            HighLatitudeRule::TwilightAngle => NightPortions {
                fajr: self.fajr_angle / 60.0,
                isha: self.isha_angle / 60.0,
            },
            HighLatitudeRule::LocalRelativeEstimation => {
                unreachable!("LRE resolves in try_new() before night_portions() is called")
            }
            HighLatitudeRule::Recommended => {
                unreachable!("Recommended resolves in try_new() before night_portions() is called")
            }
        }
    }

    /// Return the combined (user + method) time adjustment in minutes
    /// for the given [`Prayer`].
    pub fn time_adjustments(&self, prayer: Prayer) -> i64 {
        match prayer {
            Prayer::Fajr => self.adjustments.fajr + self.method_adjustments.fajr,
            Prayer::Sunrise => self.adjustments.sunrise + self.method_adjustments.sunrise,
            Prayer::Dhuhr => self.adjustments.dhuhr + self.method_adjustments.dhuhr,
            Prayer::Asr => self.adjustments.asr + self.method_adjustments.asr,
            Prayer::Maghrib => self.adjustments.maghrib + self.method_adjustments.maghrib,
            Prayer::Isha => self.adjustments.isha + self.method_adjustments.isha,
            _ => 0,
        }
    }
}

/// Builder for [`Parameters`].
pub struct Configuration {
    method: Method,
    fajr_angle: f64,
    maghrib_angle: f64,
    isha_angle: f64,
    isha_interval: i32,
    madhab: Madhab,
    high_latitude_rule: HighLatitudeRule,
    polar_estimation: Option<PolarEstimation>,
    adjustments: TimeAdjustment,
    method_adjustments: TimeAdjustment,
    rounding: Rounding,
    shafaq: Shafaq,
}

impl Configuration {
    /// Create a [`Configuration`] builder with initial Fajr and Isha angles.
    #[must_use]
    pub fn new(fajr_angle: f64, isha_angle: f64) -> Configuration {
        Configuration {
            fajr_angle,
            maghrib_angle: 0.0,
            isha_angle,
            method: Method::Other,
            isha_interval: 0,
            madhab: Madhab::Shafi,
            high_latitude_rule: HighLatitudeRule::MiddleOfTheNight,
            polar_estimation: None,
            adjustments: TimeAdjustment::default(),
            method_adjustments: TimeAdjustment::default(),
            rounding: Rounding::Nearest,
            shafaq: Shafaq::General,
        }
    }

    /// Convenience method: build [`Parameters`] from a [`Method`] and
    /// [`Madhab`] in one step, bypassing the builder chain.
    #[must_use]
    pub fn with(method: Method, madhab: Madhab) -> Parameters {
        let mut params = method.parameters();
        params.madhab = madhab;
        params.shafaq = match madhab {
            Madhab::Shafi => Shafaq::Ahmer,
            Madhab::Hanafi => Shafaq::Abyad,
        };

        params
    }

    /// Set the calculation authority.
    pub fn method(&mut self, method: Method) -> &mut Configuration {
        self.method = method;
        self
    }

    /// Override the method's built-in time adjustments.
    pub fn method_adjustments(&mut self, method_adjustments: TimeAdjustment) -> &mut Configuration {
        self.method_adjustments = method_adjustments;
        self
    }

    /// Choose the rule for approximating Fajr and Isha at high latitudes.
    pub fn high_latitude_rule(
        &mut self,
        high_latitude_rule: HighLatitudeRule,
    ) -> &mut Configuration {
        self.high_latitude_rule = high_latitude_rule;
        self
    }

    /// Set the madhab for Asr shadow-length calculation (Shafi or Hanafi).
    pub fn madhab(&mut self, madhab: Madhab) -> &mut Configuration {
        self.madhab = madhab;
        self
    }

    /// Set the strategy for handling polar day/night latitudes.
    ///
    /// Leave unset (the default) to use the true latitude; `try_new()`
    /// then returns an informative error when sunrise or sunset (or the
    /// Asr shadow angle) does not occur on the requested date.
    pub fn polar_estimation(&mut self, method: PolarEstimation) -> &mut Configuration {
        self.polar_estimation = Some(method);
        self
    }

    /// Use a fixed interval (in minutes) from Maghrib for Isha instead of
    /// a twilight angle. Sets `isha_angle` to 0.
    /// Used by Umm al-Qura (90 min) and Qatar (90 min).
    pub fn isha_interval(&mut self, isha_interval: i32) -> &mut Configuration {
        self.isha_angle = 0.0;
        self.isha_interval = isha_interval;
        self
    }

    /// Set a custom twilight angle for Maghrib (default: 0° — geometric
    /// sunset). Used by the Tehran method (4.5°).
    pub fn maghrib_angle(&mut self, angle: f64) -> &mut Configuration {
        self.maghrib_angle = angle;
        self
    }

    /// Set the rounding rule for all computed prayer times.
    pub fn rounding(&mut self, value: Rounding) -> &mut Configuration {
        self.rounding = value;
        self
    }

    /// Finalise the builder and return the [`Parameters`].
    #[must_use]
    pub fn done(&self) -> Parameters {
        Parameters {
            fajr_angle: self.fajr_angle,
            maghrib_angle: self.maghrib_angle,
            isha_angle: self.isha_angle,
            method: self.method,
            isha_interval: self.isha_interval,
            madhab: self.madhab,
            high_latitude_rule: self.high_latitude_rule,
            polar_estimation: self.polar_estimation,
            adjustments: self.adjustments,
            method_adjustments: self.method_adjustments,
            rounding: self.rounding,
            shafaq: self.shafaq,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_parameters_with_fajr_and_isha_angles() {
        let params = Parameters::new(18.0, 18.0);

        assert_eq!(params.fajr_angle, 18.0);
        assert_eq!(params.isha_angle, 18.0);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn calculated_night_portions_middle_of_the_night() {
        let params = Parameters::new(18.0, 18.0);

        assert_eq!(params.night_portions().fajr, 1.0 / 2.0);
        assert_eq!(params.night_portions().isha, 1.0 / 2.0);
    }

    #[test]
    fn calculated_night_portions_seventh_of_the_night() {
        let params = Configuration::new(18.0, 18.0)
            .high_latitude_rule(HighLatitudeRule::SeventhOfTheNight)
            .done();

        assert_eq!(params.night_portions().fajr, 1.0 / 7.0);
        assert_eq!(params.night_portions().isha, 1.0 / 7.0);
    }

    #[test]
    fn calculated_night_portions_twilight_angle() {
        let params = Configuration::new(10.0, 15.0)
            .high_latitude_rule(HighLatitudeRule::TwilightAngle)
            .done();

        assert_eq!(params.night_portions().fajr, 10.0 / 60.0);
        assert_eq!(params.night_portions().isha, 15.0 / 60.0);
    }

    #[test]
    fn calculated_night_portions_local_relative_estimation() {
        let params = Configuration::new(18.0, 17.0)
            .high_latitude_rule(HighLatitudeRule::LocalRelativeEstimation)
            .done();

        // LRE resolves internally — night_portions() is not called for it.
        // This test verifies the variant can be set without error.
        assert_eq!(
            params.high_latitude_rule,
            HighLatitudeRule::LocalRelativeEstimation
        );
    }

    #[test]
    fn parameters_using_method_and_madhab() {
        let params = Configuration::with(Method::NorthAmerica, Madhab::Hanafi);

        assert_eq!(params.method, Method::NorthAmerica);
        assert_eq!(params.fajr_angle, 15.0);
        assert_eq!(params.isha_angle, 15.0);
        assert_eq!(params.isha_interval, 0);
        assert_eq!(params.madhab, Madhab::Hanafi);
    }
}
