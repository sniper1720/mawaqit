use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};

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
    /// This is recommended for locations above 48° latitude to prevent prayer
    /// times that would be difficult to perform.
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
    /// standard angles (18° Fajr / 17° Isha).  The `f64` is the
    /// precomputed average Isha proportion of the night
    /// (`isha_length / night_length`), obtained by scanning a full
    /// year across days where the sign is stably present.
    LocalRelativeEstimation(f64),
}

impl HighLatitudeRule {
    /// Return the recommended [`HighLatitudeRule`] for the given coordinates
    /// and calculation parameters.
    ///
    /// - Latitudes with **\|latitude\| ≤ 48.6°** → [`MiddleOfTheNight`](HighLatitudeRule::MiddleOfTheNight)
    /// - Latitudes with **\|latitude\| > 48.6°** → [`LocalRelativeEstimation`](HighLatitudeRule::LocalRelativeEstimation)
    ///   (MWL 2009, the authoritative Zone 2 rule for 18°/17° angles).
    ///
    /// The boundary ±48.6° is specified by the Muslim World League's 2009
    /// document as the lower limit of Zone 2.  Within it the standard
    /// angles are always reachable year-round, so the angle-based
    /// calculation always succeeds without any fallback.  Beyond ±48.6°
    /// the more precise
    /// [`LocalRelativeEstimation`](HighLatitudeRule::LocalRelativeEstimation)
    /// is applied instead.  The percentage is computed on the fly by
    /// scanning a full year (see [`mwl_2009`](HighLatitudeRule::mwl_2009)).
    #[must_use]
    pub fn recommended(coordinates: Coordinates, params: &Parameters) -> Self {
        if coordinates.latitude.abs() > 48.6 {
            Self::mwl_2009(coordinates, params)
        } else {
            Self::MiddleOfTheNight
        }
    }

    /// Compute and return a [`LocalRelativeEstimation`](HighLatitudeRule::LocalRelativeEstimation)
    /// rule for the given location and parameters.
    ///
    /// Scans a full calendar year of real days, computes the average Isha
    /// proportion of the night, and stores it in the variant.
    #[must_use]
    pub fn mwl_2009(coordinates: Coordinates, params: &Parameters) -> Self {
        Self::LocalRelativeEstimation(Self::compute_pct(coordinates, params))
    }

    /// Scan a full calendar year and return the average Isha proportion
    /// of the night (`ratio = isha_length / night_length`).
    ///
    /// Per the MWL 2009 Arabic spec, only days where the sign is present
    /// AND not disturbed (day-to-day jump ≤ 10 min) are included in the
    /// average — days of disappearance or disturbance are excluded.
    fn compute_pct(coordinates: Coordinates, params: &Parameters) -> f64 {
        let year = Utc::now().year();
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        let jan1 = NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date");

        let mut total = 0.0;
        let mut count = 0usize;
        let mut prev_isha: Option<DateTime<Utc>> = None;
        let mut prev_was_reachable = false;

        for day_offset in 0..days_in_year {
            let date = jan1 + Duration::days(day_offset);
            let tomorrow = date + Duration::days(1);

            let Some(solar_today) = SolarTime::try_new(
                date.and_hms_opt(0, 0, 0).expect("valid time").and_utc(),
                coordinates,
            ) else {
                prev_isha = None;
                prev_was_reachable = false;
                continue;
            };
            let Some(solar_tomorrow) = SolarTime::try_new(
                tomorrow.and_hms_opt(0, 0, 0).expect("valid time").and_utc(),
                coordinates,
            ) else {
                prev_isha = None;
                prev_was_reachable = false;
                continue;
            };

            let isha_angle = Angle::new(-params.isha_angle);

            let isha_time = solar_today.time_for_solar_angle(isha_angle, true);

            let include = if let Some(current) = isha_time {
                if let Some(prev) = prev_isha {
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
                } else {
                    // First reachable day of the year — no baseline to judge
                    true
                }
            } else {
                false
            };

            if include {
                let night = solar_tomorrow
                    .sunrise
                    .signed_duration_since(solar_today.sunset);
                let night_secs = night.num_seconds() as f64;
                if night_secs > 0.0 {
                    let isha_len = isha_time
                        .expect("just checked Some")
                        .signed_duration_since(solar_today.sunset);
                    let ratio = isha_len.num_seconds() as f64 / night_secs;
                    total += ratio;
                    count += 1;
                }
            }

            prev_isha = isha_time;
            prev_was_reachable = isha_time.is_some();
        }

        if count == 0 {
            0.5
        } else {
            total / count as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommended_rule_mwl_2009_above_48_degrees() {
        let location = Coordinates::new(48.983226, -3.216649);
        let params = Parameters::new(18.0, 17.0);

        assert!(matches!(
            HighLatitudeRule::recommended(location, &params),
            HighLatitudeRule::LocalRelativeEstimation(_)
        ));
    }

    #[test]
    fn recommended_rule_middle_of_night_below_48_degrees() {
        let location = Coordinates::new(45.983226, -3.216649);
        let params = Parameters::new(18.0, 17.0);

        assert_eq!(
            HighLatitudeRule::recommended(location, &params),
            HighLatitudeRule::MiddleOfTheNight
        );
    }

    #[test]
    fn compute_pct_brussels_is_reasonable() {
        let location = Coordinates::new(50.85, 4.35);
        let params = Parameters::new(18.0, 17.0);
        let rule = HighLatitudeRule::mwl_2009(location, &params);

        if let HighLatitudeRule::LocalRelativeEstimation(pct) = rule {
            assert!(
                pct > 0.1 && pct < 0.9,
                "Brussels pct should be between 0.1 and 0.9, got {pct}"
            );
        } else {
            panic!("mwl_2009 did not return LocalRelativeEstimation");
        }
    }

    #[test]
    fn compute_pct_oslo_is_reasonable() {
        let location = Coordinates::new(59.9094, 10.7349);
        let params = Parameters::new(18.0, 17.0);
        let rule = HighLatitudeRule::mwl_2009(location, &params);

        if let HighLatitudeRule::LocalRelativeEstimation(pct) = rule {
            assert!(
                pct > 0.1 && pct < 0.9,
                "Oslo pct should be between 0.1 and 0.9, got {pct}"
            );
        } else {
            panic!("mwl_2009 did not return LocalRelativeEstimation");
        }
    }

    #[test]
    fn mwl_2009_equator_fallback() {
        let location = Coordinates::new(0.0, 0.0);
        let params = Parameters::new(18.0, 17.0);
        let rule = HighLatitudeRule::mwl_2009(location, &params);

        // At the equator night and day are ~12 h year round.
        // 17° Isha is ~(17/60) ≈ 0.28 of the night after sunset.
        // Pct should be reasonable (0 < pct < 1).
        if let HighLatitudeRule::LocalRelativeEstimation(pct) = rule {
            assert!(
                pct > 0.0 && pct < 1.0,
                "Equator pct must be between 0 and 1, got {pct}"
            );
            // At the equator with 17° angle, expect roughly 0.25–0.35
            assert!(pct < 0.5, "Equator pct should be < 0.5, got {pct}");
        } else {
            panic!("mwl_2009 did not return LocalRelativeEstimation");
        }
    }
}
