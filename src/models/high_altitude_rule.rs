use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::astronomy::solar::SolarTime;
use crate::astronomy::unit::{Angle, Coordinates};
use crate::models::parameters::Parameters;

/// Rule for approximating Fajr and Isha at high latitudes
#[derive(PartialEq, Debug, Copy, Clone)]
#[non_exhaustive]
pub enum HighLatitudeRule {
    /// Fajr won't be earlier than the midpoint of the night and Isha
    /// won't be later than the midpoint of the night. This is the default
    /// value to prevent Fajr and Isha crossing boundaries.
    MiddleOfTheNight,

    /// Fajr will never be earlier than the beginning of the last seventh of
    /// the night and Isha will never be later than the end of the first seventh of the night.
    ///
    /// Useful at high latitudes where the standard angles cannot be
    /// reached; the MWL's Zone-2 recommendation since 2009 is
    /// [`LocalRelativeEstimation`](HighLatitudeRule::LocalRelativeEstimation).
    SeventhOfTheNight,

    /// The fajr/isha angle α determines a fraction t = α ÷ 60 of the night.
    /// Isha begins after the first t part; Fajr before the last t part.
    /// Example: 15° → t = 0.25 → Isha after the first quarter of the night.
    ///
    /// This can be used to prevent difficult fajr and isha times at certain locations.
    TwilightAngle,

    /// MWL 2009 Local Relative Estimation.
    ///
    /// Designed for the Muslim World League's 2009 high-latitude
    /// methodology (between 48.6° and 66.6° latitude) using their
    /// standard angles (18° Fajr / 17° Isha).  The percentage is
    /// resolved automatically inside [`crate::schedule::PrayerTimes::try_new`] by
    /// scanning a full year at the reference latitude.
    LocalRelativeEstimation,

    /// Deferred variant resolved inside [`crate::schedule::PrayerTimes::try_new`].
    ///
    /// Evaluated via [`recommended()`](HighLatitudeRule::recommended) against
    /// the reference latitude (original or estimation-resolved).
    Recommended,
}

impl HighLatitudeRule {
    /// Return the recommended [`HighLatitudeRule`] for the given coordinates.
    ///
    /// Based on MWL 2009 latitude zones:
    /// - |latitude| ≤ 48.6° → [`MiddleOfTheNight`](HighLatitudeRule::MiddleOfTheNight) (Zone 1 — no effect, angles always reachable)
    /// - 48.6° < |latitude| ≤ 66.6° → [`LocalRelativeEstimation`](HighLatitudeRule::LocalRelativeEstimation) (Zone 2)
    /// - |latitude| > 66.6° → [`MiddleOfTheNight`](HighLatitudeRule::MiddleOfTheNight) (Zone 3 — LRE not designed for polar)
    #[must_use]
    pub fn recommended(coordinates: Coordinates) -> Self {
        let absolute_latitude = coordinates.latitude.abs();
        if absolute_latitude > 66.6 {
            Self::MiddleOfTheNight
        } else if absolute_latitude > 48.6 {
            Self::LocalRelativeEstimation
        } else {
            Self::MiddleOfTheNight
        }
    }

    /// Scan the given calendar year and return the average Isha proportion
    /// of the night (`ratio = isha_length / night_length`).
    ///
    /// Per the MWL 2009 Arabic spec, only days where the sign is present
    /// AND not disturbed (day-to-day jump ≤ 10 min) are included in the
    /// average — days of disappearance or disturbance are excluded.
    pub fn compute_isha_night_ratio(
        coordinates: Coordinates,
        params: &Parameters,
        year: i32,
    ) -> f64 {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        let jan1 = NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date");

        let mut ratio_sum = 0.0;
        let mut days_included = 0usize;
        let mut prev_isha: Option<DateTime<Utc>> = None;
        let mut prev_was_reachable = false;

        for day_offset in 0..days_in_year {
            let date = jan1 + Duration::days(day_offset);
            let tomorrow = date + Duration::days(1);

            let Ok(solar_today) = SolarTime::new(
                date.and_hms_opt(0, 0, 0).expect("valid time").and_utc(),
                coordinates,
            ) else {
                prev_isha = None;
                prev_was_reachable = false;
                continue;
            };
            let Ok(solar_tomorrow) = SolarTime::new(
                tomorrow.and_hms_opt(0, 0, 0).expect("valid time").and_utc(),
                coordinates,
            ) else {
                prev_isha = None;
                prev_was_reachable = false;
                continue;
            };

            let isha_angle = Angle::new(-params.isha_angle);

            let isha_time = solar_today.time_for_solar_angle(isha_angle, true);

            let include = if let Some(current) = isha_time
                && let Some(prev) = prev_isha
            {
                if prev_was_reachable {
                    // Both days reachable — exclude if jump > 10 min
                    let prev_today = date.and_time(prev.time()).and_utc();
                    let raw_diff = (current - prev_today).num_seconds() as f64 / 60.0;
                    let diff = if raw_diff.abs() > 720.0 {
                        let wrapped = if raw_diff > 0.0 {
                            raw_diff - 1440.0
                        } else {
                            raw_diff + 1440.0
                        };
                        wrapped.abs()
                    } else {
                        raw_diff.abs()
                    };
                    diff <= 10.0
                } else {
                    // Previous day was unreachable → reappearance day → exclude
                    false
                }
            } else if isha_time.is_some() {
                // First reachable day of the year — no baseline to judge
                true
            } else {
                false
            };

            if include {
                let night = solar_tomorrow
                    .sunrise
                    .expect("compute_isha_night_ratio only runs where sunrise exists")
                    .signed_duration_since(
                        solar_today
                            .sunset
                            .expect("compute_isha_night_ratio only runs where sunset exists"),
                    );
                let night_secs = night.num_seconds() as f64;
                if night_secs > 0.0 {
                    let isha_len = isha_time
                        .expect("include is only true when isha_time is Some")
                        .signed_duration_since(
                            solar_today
                                .sunset
                                .expect("compute_isha_night_ratio only runs where sunset exists"),
                        );
                    let ratio = isha_len.num_seconds() as f64 / night_secs;
                    ratio_sum += ratio;
                    days_included += 1;
                }
            }

            prev_isha = isha_time;
            prev_was_reachable = isha_time.is_some();
        }

        if days_included == 0 {
            0.5
        } else {
            ratio_sum / days_included as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommended_rule_lre_above_48_degrees() {
        let location = Coordinates::new(48.983226, -3.216649);

        assert_eq!(
            HighLatitudeRule::recommended(location),
            HighLatitudeRule::LocalRelativeEstimation
        );
    }

    #[test]
    fn recommended_rule_middle_of_night_below_48_degrees() {
        let location = Coordinates::new(45.983226, -3.216649);

        assert_eq!(
            HighLatitudeRule::recommended(location),
            HighLatitudeRule::MiddleOfTheNight
        );
    }

    #[test]
    fn recommended_rule_middle_of_night_above_66_degrees() {
        let location = Coordinates::new(70.0, 20.0);

        assert_eq!(
            HighLatitudeRule::recommended(location),
            HighLatitudeRule::MiddleOfTheNight
        );
    }

    #[test]
    fn compute_isha_night_ratio_brussels_is_reasonable() {
        let location = Coordinates::new(50.85, 4.35);
        let params = Parameters::new(18.0, 17.0);
        let ratio = HighLatitudeRule::compute_isha_night_ratio(location, &params, 2026);

        assert!(
            ratio > 0.1 && ratio < 0.9,
            "Brussels ratio should be between 0.1 and 0.9, got {ratio}"
        );
    }

    #[test]
    fn compute_isha_night_ratio_oslo_is_reasonable() {
        let location = Coordinates::new(59.9094, 10.7349);
        let params = Parameters::new(18.0, 17.0);
        let ratio = HighLatitudeRule::compute_isha_night_ratio(location, &params, 2026);

        assert!(
            ratio > 0.1 && ratio < 0.9,
            "Oslo ratio should be between 0.1 and 0.9, got {ratio}"
        );
    }

    #[test]
    fn compute_isha_night_ratio_equator_is_reasonable() {
        let location = Coordinates::new(0.0, 0.0);
        let params = Parameters::new(18.0, 17.0);
        let ratio = HighLatitudeRule::compute_isha_night_ratio(location, &params, 2026);

        // At the equator night and day are ~12 h year round.
        // 17° Isha is ~(17/60) ≈ 0.28 of the night after sunset.
        // Ratio should be reasonable (0 < ratio < 1).
        assert!(
            ratio > 0.0 && ratio < 1.0,
            "Equator ratio must be between 0 and 1, got {ratio}"
        );
        // At the equator with 17° angle, expect roughly 0.25–0.35
        assert!(ratio < 0.5, "Equator ratio should be < 0.5, got {ratio}");
    }
}
