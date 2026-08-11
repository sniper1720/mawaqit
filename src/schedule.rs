//! # Prayer Schedule
//!
//! This module provides the main objects that are used for calculating
//! the prayer times.

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};

use crate::astronomy::solar::SolarTime;
use crate::astronomy::unit::{Angle, Coordinates, Stride};
use crate::methods::moonsighting;
use crate::models::high_altitude_rule::HighLatitudeRule;
use crate::models::method::Method;
use crate::models::parameters::Parameters;

use crate::models::prayer::Prayer;
use crate::models::rounding::Rounding;

/// Which prayer a Local Relative Estimation target represents.
///
/// Mirrors MWL 2009's Fajr/Isha pair; kept private to the schedule
/// because LRE is an internal concern.
#[derive(Clone, Copy)]
enum PrayerSide {
    Fajr,
    Isha,
}

impl PrayerSide {
    /// True when the target crosses the solar angle after transit (evening),
    /// as opposed to before transit (dawn).
    fn is_after_transit(self) -> bool {
        matches!(self, PrayerSide::Isha)
    }
}

/// Times of all prayers for a given date, location, and configuration.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct PrayerTimes {
    fajr: DateTime<Utc>,
    sunrise: DateTime<Utc>,
    dhuhr: DateTime<Utc>,
    asr: DateTime<Utc>,
    maghrib: DateTime<Utc>,
    isha: DateTime<Utc>,
    isha_yesterday: DateTime<Utc>,
    middle_of_the_night: DateTime<Utc>,
    qiyam: DateTime<Utc>,
    fajr_tomorrow: DateTime<Utc>,
    coordinates: Coordinates,
    reference_latitude: f64,
    date: DateTime<Utc>,
    parameters: Parameters,
}

impl PrayerTimes {
    /// Try to compute all prayer times. With no [`crate::models::polar::PolarEstimation`]
    /// configured, returns an informative error at polar latitudes
    /// where sunrise or sunset (or the Asr shadow angle) does not occur.
    pub fn try_new(
        date: NaiveDate,
        coordinates: Coordinates,
        mut parameters: Parameters,
    ) -> Result<PrayerTimes, &'static str> {
        let prayer_date = date
            .and_hms_opt(0, 0, 0)
            .ok_or("Invalid date provided")?
            .and_utc();
        let tomorrow = prayer_date.tomorrow();
        let yesterday = prayer_date - Duration::days(1);

        // Resolve the latitude used for solar calculations. A missing
        // fallback strategy uses the true latitude — polar day/night and
        // the unreachable-Asr window then surface as errors below.
        let resolved_latitude = match parameters.polar_estimation {
            Some(fallback) => {
                fallback.resolve_latitude(prayer_date, coordinates, parameters.madhab)
            }
            None => coordinates.latitude,
        };
        let reference_coordinates = Coordinates::new(resolved_latitude, coordinates.longitude);

        // Full SolarTime at reference latitude for all prayers.
        let solar_at_reference = SolarTime::new(prayer_date, reference_coordinates)?;
        let solar_at_reference_tomorrow = SolarTime::new(tomorrow, reference_coordinates)?;
        let solar_at_reference_yesterday = SolarTime::new(yesterday, reference_coordinates)?;

        // Night length from reference latitude
        let night = solar_at_reference_tomorrow
            .sunrise
            .unwrap()
            .signed_duration_since(solar_at_reference.sunset.unwrap());
        let night_yesterday = solar_at_reference
            .sunrise
            .unwrap()
            .signed_duration_since(solar_at_reference_yesterday.sunset.unwrap());

        // Resolve deferred Recommended variant against the working latitude.
        if parameters.high_latitude_rule == HighLatitudeRule::Recommended {
            parameters.high_latitude_rule = HighLatitudeRule::recommended(reference_coordinates);
        }

        // MWL 2009: LRE is defined only for Zone 2 (|latitude| ≤ 66.6°).
        if parameters.high_latitude_rule == HighLatitudeRule::LocalRelativeEstimation
            && reference_coordinates.latitude.abs() > 66.6
        {
            return Err("LocalRelativeEstimation not applicable above 66.6° latitude");
        }

        // LRE has its own MWL 2009 path; others use calculate_fajr/calculate_isha.
        // Compute the ratio on the fly from a year-long scan.
        let lre_ratio =
            if parameters.high_latitude_rule == HighLatitudeRule::LocalRelativeEstimation {
                Some(HighLatitudeRule::compute_isha_night_ratio(
                    reference_coordinates,
                    &parameters,
                    prayer_date.year(),
                ))
            } else {
                None
            };

        let final_fajr = if let Some(isha_night_ratio) = lre_ratio {
            PrayerTimes::compute_lre(
                isha_night_ratio,
                &parameters,
                prayer_date,
                reference_coordinates,
                PrayerSide::Fajr,
            )
            .adjust_time(parameters.time_adjustments(Prayer::Fajr))
            .rounded_minute(parameters.rounding)
        } else {
            PrayerTimes::calculate_fajr(
                parameters,
                solar_at_reference,
                night_yesterday,
                reference_coordinates,
                prayer_date,
            )
            .rounded_minute(parameters.rounding)
        };

        let final_sunrise = solar_at_reference
            .sunrise
            .unwrap()
            .adjust_time(parameters.time_adjustments(Prayer::Sunrise))
            .rounded_minute(parameters.rounding);

        // ── Dhuhr (NearestLatitude shifts transit to the resolved latitude too) ──
        let final_dhuhr = solar_at_reference
            .transit
            .adjust_time(parameters.time_adjustments(Prayer::Dhuhr))
            .rounded_minute(parameters.rounding);

        let final_asr = solar_at_reference
            .time_for_shadow(parameters.madhab.shadow().into())
            .ok_or("Asr shadow angle not reachable at this latitude on this date")?
            .adjust_time(parameters.time_adjustments(Prayer::Asr))
            .rounded_minute(parameters.rounding);

        let final_maghrib = PrayerTimes::calculate_maghrib(parameters, solar_at_reference)
            .rounded_minute(parameters.rounding);

        let final_isha = if let Some(isha_night_ratio) = lre_ratio {
            PrayerTimes::compute_lre(
                isha_night_ratio,
                &parameters,
                prayer_date,
                reference_coordinates,
                PrayerSide::Isha,
            )
            .adjust_time(parameters.time_adjustments(Prayer::Isha))
            .rounded_minute(parameters.rounding)
        } else {
            PrayerTimes::calculate_isha(
                parameters,
                solar_at_reference,
                night,
                reference_coordinates,
                prayer_date,
            )
            .rounded_minute(parameters.rounding)
        };

        // ── Yesterday's Isha (covers midnight-to-Fajr gap) ──
        let isha_yesterday = if let Some(isha_night_ratio) = lre_ratio {
            PrayerTimes::compute_lre(
                isha_night_ratio,
                &parameters,
                yesterday,
                reference_coordinates,
                PrayerSide::Isha,
            )
            .adjust_time(parameters.time_adjustments(Prayer::Isha))
            .rounded_minute(parameters.rounding)
        } else {
            PrayerTimes::calculate_isha(
                parameters,
                solar_at_reference_yesterday,
                night_yesterday,
                reference_coordinates,
                yesterday,
            )
            .rounded_minute(parameters.rounding)
        };

        // ── Tomorrow's Fajr (for qiyam) ──
        let tomorrow_fajr = if let Some(isha_night_ratio) = lre_ratio {
            PrayerTimes::compute_lre(
                isha_night_ratio,
                &parameters,
                tomorrow,
                reference_coordinates,
                PrayerSide::Fajr,
            )
            .adjust_time(parameters.time_adjustments(Prayer::Fajr))
            .rounded_minute(parameters.rounding)
        } else {
            PrayerTimes::calculate_fajr(
                parameters,
                solar_at_reference_tomorrow,
                night,
                reference_coordinates,
                tomorrow,
            )
            .rounded_minute(parameters.rounding)
        };

        let (final_middle_of_night, final_qiyam, final_fajr_tomorrow) =
            PrayerTimes::calculate_qiyam(final_maghrib, tomorrow_fajr);

        Ok(PrayerTimes {
            fajr: final_fajr,
            sunrise: final_sunrise,
            dhuhr: final_dhuhr,
            asr: final_asr,
            maghrib: final_maghrib,
            isha: final_isha,
            isha_yesterday,
            middle_of_the_night: final_middle_of_night,
            qiyam: final_qiyam,
            fajr_tomorrow: final_fajr_tomorrow,
            coordinates,
            reference_latitude: resolved_latitude,
            date: prayer_date,
            parameters,
        })
    }

    /// Return the latitude actually used for solar calculations after any
    /// [`crate::models::polar::PolarEstimation`] resolution. Equals `coordinates.latitude` when
    /// no fallback strategy is configured.
    #[must_use]
    pub fn resolved_latitude(&self) -> f64 {
        self.reference_latitude
    }

    /// Return the UTC [`DateTime`] at which the given [`Prayer`] occurs.
    #[must_use]
    pub fn time(&self, prayer: Prayer) -> DateTime<Utc> {
        match prayer {
            Prayer::Fajr => self.fajr,
            Prayer::Sunrise => self.sunrise,
            Prayer::Dhuhr => self.dhuhr,
            Prayer::Asr => self.asr,
            Prayer::Maghrib => self.maghrib,
            Prayer::Isha => self.isha,
            Prayer::Qiyam => self.qiyam,
            Prayer::FajrTomorrow => self.fajr_tomorrow,
        }
    }

    /// Return the [`Prayer`] in effect at the given instant.
    #[must_use]
    pub fn current_at(&self, now: DateTime<Utc>) -> Prayer {
        self.current_time(now)
    }

    /// Return the [`Prayer`] currently in effect.
    #[must_use]
    pub fn current(&self) -> Prayer {
        self.current_at(Utc::now())
    }

    /// Return the UTC [`DateTime`] when the current prayer started.
    ///
    /// During the midnight-to-Fajr gap (which falls in the previous
    /// Islamic day's Isha window), returns yesterday's Isha time so the
    /// value is always in the past.
    #[must_use]
    pub fn current_prayer_time(&self) -> DateTime<Utc> {
        self.current_prayer_time_at(Utc::now())
    }

    fn current_prayer_time_at(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        let prayer = self.current_time(now);
        match prayer {
            Prayer::Isha if self.isha > now => self.isha_yesterday,
            _ => self.time(prayer),
        }
    }

    /// Return the [`Prayer`] that begins next after the given instant.
    #[must_use]
    pub fn next_at(&self, now: DateTime<Utc>) -> Prayer {
        match self.current_time(now) {
            Prayer::Fajr => Prayer::Sunrise,
            Prayer::Sunrise => Prayer::Dhuhr,
            Prayer::Dhuhr => Prayer::Asr,
            Prayer::Asr => Prayer::Maghrib,
            Prayer::Maghrib => Prayer::Isha,
            Prayer::Isha if self.isha <= now => Prayer::Qiyam,
            Prayer::Qiyam => Prayer::FajrTomorrow,
            Prayer::FajrTomorrow => Prayer::FajrTomorrow,
            _ => Prayer::Fajr,
        }
    }

    /// Return the [`Prayer`] that begins next.
    #[must_use]
    pub fn next(&self) -> Prayer {
        self.next_at(Utc::now())
    }

    /// Return the exact time until the prayer after the given instant starts.
    ///
    /// The value is negative once that prayer has already started — most
    /// notably when this schedule has gone stale past tomorrow's Fajr.
    #[must_use]
    pub fn time_remaining_at(&self, now: DateTime<Utc>) -> Duration {
        self.time(self.next_at(now)).signed_duration_since(now)
    }

    fn current_time(&self, now: DateTime<Utc>) -> Prayer {
        if self.fajr_tomorrow.signed_duration_since(now).num_seconds() <= 0 {
            Prayer::FajrTomorrow
        } else if self.qiyam.signed_duration_since(now).num_seconds() <= 0 {
            Prayer::Qiyam
        } else if self.isha.signed_duration_since(now).num_seconds() <= 0 {
            Prayer::Isha
        } else if self.maghrib.signed_duration_since(now).num_seconds() <= 0 {
            Prayer::Maghrib
        } else if self.asr.signed_duration_since(now).num_seconds() <= 0 {
            Prayer::Asr
        } else if self.dhuhr.signed_duration_since(now).num_seconds() <= 0 {
            Prayer::Dhuhr
        } else if self.sunrise.signed_duration_since(now).num_seconds() <= 0 {
            Prayer::Sunrise
        } else if self.fajr.signed_duration_since(now).num_seconds() <= 0 {
            Prayer::Fajr
        } else {
            Prayer::Isha
        }
    }

    fn calculate_fajr(
        parameters: Parameters,
        solar_time: SolarTime,
        night: Duration,
        coordinates: Coordinates,
        prayer_date: DateTime<Utc>,
    ) -> DateTime<Utc> {
        let sunrise = solar_time
            .sunrise
            .expect("calculate_fajr requires valid sunrise");
        let safe_fajr = if parameters.method == Method::MoonsightingCommittee {
            let day_of_year = prayer_date.ordinal();
            moonsighting::season_adjusted_morning_twilight(
                coordinates.latitude,
                day_of_year,
                prayer_date.year() as u32,
                sunrise,
            )
        } else {
            let portion = parameters.night_portions().fajr;
            let night_fraction = portion * (night.num_seconds() as f64);

            sunrise
                .checked_add_signed(Duration::seconds(-night_fraction as i64))
                .expect("fajr safe-time computation overflowed")
        };

        let mut fajr = solar_time
            .time_for_solar_angle(Angle::new(-parameters.fajr_angle), false)
            .unwrap_or(safe_fajr);

        // special case for moonsighting committee above latitude 55
        if parameters.method == Method::MoonsightingCommittee && coordinates.latitude >= 55.0 {
            let night_fraction = night.num_seconds() / 7;
            fajr = sunrise
                .checked_add_signed(Duration::seconds(-night_fraction))
                .expect("fajr MoonsightingCommittee night-fraction overflowed");
        }

        if fajr < safe_fajr {
            fajr = safe_fajr;
        }

        fajr.adjust_time(parameters.time_adjustments(Prayer::Fajr))
    }

    fn calculate_isha(
        parameters: Parameters,
        solar_time: SolarTime,
        night: Duration,
        coordinates: Coordinates,
        prayer_date: DateTime<Utc>,
    ) -> DateTime<Utc> {
        let sunset = solar_time
            .sunset
            .expect("calculate_isha requires valid sunset");
        let mut isha: DateTime<Utc>;

        if parameters.isha_interval > 0 {
            isha = sunset
                .checked_add_signed(Duration::seconds((parameters.isha_interval * 60) as i64))
                .expect("isha interval-based computation overflowed");
        } else {
            let safe_isha = if parameters.method == Method::MoonsightingCommittee {
                let day_of_year = prayer_date.ordinal();

                moonsighting::season_adjusted_evening_twilight(
                    coordinates.latitude,
                    day_of_year,
                    prayer_date.year() as u32,
                    sunset,
                    parameters.shafaq,
                )
            } else {
                let portion = parameters.night_portions().isha;
                let night_fraction = portion * (night.num_seconds() as f64);

                sunset
                    .checked_add_signed(Duration::seconds(night_fraction as i64))
                    .expect("isha safe-time computation overflowed")
            };

            isha = solar_time
                .time_for_solar_angle(Angle::new(-parameters.isha_angle), true)
                .unwrap_or(safe_isha);

            // special case for moonsighting committee above latitude 55
            if parameters.method == Method::MoonsightingCommittee && coordinates.latitude >= 55.0 {
                let night_fraction = night.num_seconds() / 7;
                isha = sunset
                    .checked_add_signed(Duration::seconds(night_fraction))
                    .expect("isha MoonsightingCommittee night-fraction overflowed");
            }

            if isha > safe_isha {
                isha = safe_isha;
            }
        }

        isha.adjust_time(parameters.time_adjustments(Prayer::Isha))
    }

    fn calculate_maghrib(parameters: Parameters, solar_time: SolarTime) -> DateTime<Utc> {
        let sunset = solar_time
            .sunset
            .expect("calculate_maghrib requires valid sunset");
        let maghrib = if parameters.maghrib_angle > 0.0 {
            solar_time
                .time_for_solar_angle(Angle::new(-parameters.maghrib_angle), true)
                .unwrap_or(sunset)
        } else {
            sunset
        };

        maghrib.adjust_time(parameters.time_adjustments(Prayer::Maghrib))
    }

    fn calculate_qiyam(
        current_maghrib: DateTime<Utc>,
        tomorrow_fajr: DateTime<Utc>,
    ) -> (DateTime<Utc>, DateTime<Utc>, DateTime<Utc>) {
        let night_duration = tomorrow_fajr
            .signed_duration_since(current_maghrib)
            .num_seconds() as f64;
        let middle_night_portion = (night_duration / 2.0) as i64;
        let last_third_portion = (night_duration * (2.0 / 3.0)) as i64;
        let middle_of_night = current_maghrib
            .checked_add_signed(Duration::seconds(middle_night_portion))
            .expect("middle-of-night computation overflowed")
            .rounded_minute(Rounding::Nearest);
        let last_third_of_night = current_maghrib
            .checked_add_signed(Duration::seconds(last_third_portion))
            .expect("last-third-of-night computation overflowed")
            .rounded_minute(Rounding::Nearest);

        (middle_of_night, last_third_of_night, tomorrow_fajr)
    }

    // ── MWL 2009 Local Relative Estimation helpers ──────────────

    /// Compute LRE-smoothed time for a single prayer on a single date.
    ///
    /// Disturbance is detected by a day-to-day jump > 10 minutes
    /// (per the Arabic definition of اضطراب العلامة) or by the
    /// angle being unreachable.  The entry ramp steps ±5 min/day
    /// toward the PCT-based time, and the exit ramp (at reappearance)
    /// steps in the opposite direction toward the real time.
    ///
    /// The sequence is:  Real → Adjusted → PCT → Adjusted → Real.
    fn compute_lre(
        isha_night_ratio: f64,
        params: &Parameters,
        prayer_date: DateTime<Utc>,
        coordinates: Coordinates,
        side: PrayerSide,
    ) -> DateTime<Utc> {
        let tomorrow_date = prayer_date.tomorrow();
        let solar_time = SolarTime::new(prayer_date, coordinates).unwrap();
        let solar_time_tomorrow = SolarTime::new(tomorrow_date, coordinates).unwrap();
        let night_duration = solar_time_tomorrow
            .sunrise
            .unwrap()
            .signed_duration_since(solar_time.sunset.unwrap());

        let solar_altitude = match side {
            PrayerSide::Isha => -params.isha_angle,
            PrayerSide::Fajr => -params.fajr_angle,
        };
        let today_real =
            solar_time.time_for_solar_angle(Angle::new(solar_altitude), side.is_after_transit());

        let night_duration_secs = night_duration.num_seconds() as f64;
        let sunset = solar_time.sunset.unwrap();
        let sunrise = solar_time.sunrise.unwrap();
        let ratio_time = match side {
            PrayerSide::Isha => sunset
                .checked_add_signed(Duration::seconds(
                    (isha_night_ratio * night_duration_secs) as i64,
                ))
                .expect("LRE pct_isha overflow"),
            PrayerSide::Fajr => sunrise
                .checked_add_signed(Duration::seconds(
                    -(isha_night_ratio * night_duration_secs) as i64,
                ))
                .expect("LRE pct_fajr overflow"),
        };

        // Entry/exit step direction
        // Entry: toward PCT (Isha −5, Fajr +5)
        // Exit:  toward real (Isha +5, Fajr −5)
        let entry_step = match side {
            PrayerSide::Isha => Duration::minutes(-5),
            PrayerSide::Fajr => Duration::minutes(5),
        };
        let exit_step = match side {
            PrayerSide::Isha => Duration::minutes(5),
            PrayerSide::Fajr => Duration::minutes(-5),
        };

        // Minimum distance around the 24h clock (handles midnight wrapping)
        let clock_dist = |a: DateTime<Utc>, b: DateTime<Utc>| -> f64 {
            let raw = a.signed_duration_since(b).num_seconds() as f64 / 60.0;
            let wrapped = if raw >= 720.0 {
                raw - 1440.0
            } else if raw <= -720.0 {
                raw + 1440.0
            } else {
                raw
            };
            wrapped.abs()
        };

        // Yesterday's real (for day-to-day disturbance detection)
        let yesterday_date = prayer_date - Duration::days(1);
        let yesterday_solar = SolarTime::new(yesterday_date, coordinates).unwrap();
        let yesterday_real = yesterday_solar
            .time_for_solar_angle(Angle::new(solar_altitude), side.is_after_transit());

        // Disturbance: unreachable OR day-to-day jump > 10 min
        if let (Some(today_time), Some(yesterday_time)) = (today_real, yesterday_real) {
            let yesterday_real_on_today = prayer_date
                .date_naive()
                .and_time(yesterday_time.time())
                .and_utc();
            if clock_dist(today_time, yesterday_real_on_today) <= 10.0 {
                return today_time;
            }
        }

        // LRE mode: compute previous day's smoothed value (may recurse)
        let prev_lre_time =
            Self::compute_lre(isha_night_ratio, params, yesterday_date, coordinates, side);

        // Normalise prev to today's date so only time-of-day is compared
        let prev_lre_on_today = prayer_date
            .date_naive()
            .and_time(prev_lre_time.time())
            .and_utc();

        let was_in_lre = clock_dist(prev_lre_on_today, ratio_time) <= 5.0;

        if let Some(today_time) = today_real {
            // Real exists but disturbed → LRE transition (entry or exit)
            if was_in_lre
                || clock_dist(prev_lre_on_today, ratio_time)
                    < clock_dist(prev_lre_on_today, today_time)
            {
                // Closer to PCT → exit toward real (opposite direction)
                let adjusted = prev_lre_on_today + exit_step;
                if clock_dist(adjusted, today_time) <= 5.0 {
                    today_time
                } else {
                    adjusted
                }
            } else {
                // Closer to real → entry toward PCT
                let adjusted = prev_lre_on_today + entry_step;
                if clock_dist(adjusted, ratio_time) <= 5.0 {
                    ratio_time
                } else {
                    adjusted
                }
            }
        } else {
            // No real → entry toward PCT
            if was_in_lre {
                ratio_time
            } else {
                let adjusted = prev_lre_on_today + entry_step;
                if clock_dist(adjusted, ratio_time) <= 5.0 {
                    ratio_time
                } else {
                    adjusted
                }
            }
        }
    }
}

/// A builder for [`PrayerTimes`].
pub struct PrayerSchedule {
    date: Option<NaiveDate>,
    coordinates: Option<Coordinates>,
    params: Option<Parameters>,
}

impl Default for PrayerSchedule {
    fn default() -> Self {
        Self::new()
    }
}

impl PrayerSchedule {
    /// Create an empty [`PrayerSchedule`] builder.
    #[must_use]
    pub fn new() -> Self {
        PrayerSchedule {
            date: None,
            coordinates: None,
            params: None,
        }
    }

    /// Set the date for calculation.
    pub fn on(&mut self, date: NaiveDate) -> &mut PrayerSchedule {
        self.date = Some(date);
        self
    }

    /// Set the geographic coordinates.
    pub fn for_location(&mut self, location: Coordinates) -> &mut PrayerSchedule {
        self.coordinates = Some(location);
        self
    }

    /// Provide [`Parameters`] (method, madhab, adjustments, etc.).
    pub fn with_configuration(&mut self, params: Parameters) -> &mut PrayerSchedule {
        self.params = Some(params);
        self
    }

    /// Compute [`PrayerTimes`]. Returns an error if any required field
    /// (date, coordinates, parameters) has not been set.
    pub fn calculate(&self) -> Result<PrayerTimes, String> {
        if let (Some(date), Some(coordinates), Some(params)) =
            (self.date, self.coordinates, self.params)
        {
            PrayerTimes::try_new(date, coordinates, params).map_err(|e| e.to_string())
        } else {
            Err(
                "Missing required date, coordinates, and parameters to calculate prayer times"
                    .to_string(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Configuration;
    use crate::models::high_altitude_rule::HighLatitudeRule;
    use crate::models::madhab::Madhab;
    use chrono::{NaiveDate, TimeZone, Utc};

    #[test]
    fn current_prayer_should_be_fajr() {
        // Given the above DateTime, the Fajr prayer is at 2015-07-12T08:42:00Z
        let local_date = NaiveDate::from_ymd_opt(2015, 7, 12).expect("Invalid date provided");
        let params = Configuration::with(Method::NorthAmerica, Madhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::try_new(local_date, coordinates, params).unwrap();
        let current_prayer_time = local_date.and_hms_opt(9, 0, 0).unwrap().and_utc();

        assert_eq!(times.current_time(current_prayer_time), Prayer::Fajr);
    }

    #[test]
    fn current_prayer_should_be_sunrise() {
        // Given the below DateTime, sunrise is at 2015-07-12T10:08:00Z
        let local_date = NaiveDate::from_ymd_opt(2015, 7, 12).expect("Invalid date provided");
        let params = Configuration::with(Method::NorthAmerica, Madhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::try_new(local_date, coordinates, params).unwrap();
        let current_prayer_time = local_date.and_hms_opt(11, 0, 0).unwrap().and_utc();

        assert_eq!(times.current_time(current_prayer_time), Prayer::Sunrise);
    }

    #[test]
    fn current_prayer_should_be_dhuhr() {
        // Given the above DateTime, dhuhr prayer is at 2015-07-12T17:21:00Z
        let local_date = NaiveDate::from_ymd_opt(2015, 7, 12).expect("Invalid date provided");
        let params = Configuration::with(Method::NorthAmerica, Madhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::try_new(local_date, coordinates, params).unwrap();
        let current_prayer_time = local_date.and_hms_opt(19, 0, 0).unwrap().and_utc();

        assert_eq!(times.current_time(current_prayer_time), Prayer::Dhuhr);
    }

    #[test]
    fn current_prayer_should_be_asr() {
        // Given the below DateTime, asr is at 2015-07-12T22:22:00Z
        let local_date = NaiveDate::from_ymd_opt(2015, 7, 12).expect("Invalid date provided");
        let params = Configuration::with(Method::NorthAmerica, Madhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::try_new(local_date, coordinates, params).unwrap();
        let current_prayer_time = local_date.and_hms_opt(22, 26, 0).unwrap().and_utc();

        assert_eq!(times.current_time(current_prayer_time), Prayer::Asr);
    }

    #[test]
    fn current_prayer_should_be_maghrib() {
        // Given the below DateTime, maghrib is at 2015-07-13T00:32:00Z
        let local_date = NaiveDate::from_ymd_opt(2015, 7, 12).expect("Invalid date provided");
        let params = Configuration::with(Method::NorthAmerica, Madhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::try_new(local_date, coordinates, params).unwrap();
        let current_prayer_time = Utc.with_ymd_and_hms(2015, 7, 13, 1, 0, 0).unwrap();

        assert_eq!(times.current_time(current_prayer_time), Prayer::Maghrib);
    }

    #[test]
    fn current_prayer_should_be_isha() {
        // Given the below DateTime, isha is at 2015-07-13T01:57:00Z
        let local_date = NaiveDate::from_ymd_opt(2015, 7, 12).expect("Invalid date provided");
        let params = Configuration::with(Method::NorthAmerica, Madhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::try_new(local_date, coordinates, params).unwrap();
        let current_prayer_time = Utc.with_ymd_and_hms(2015, 7, 13, 2, 0, 0).unwrap();

        assert_eq!(times.current_time(current_prayer_time), Prayer::Isha);
    }

    #[test]
    fn current_prayer_should_be_isha_before_fajr() {
        let local_date = NaiveDate::from_ymd_opt(2015, 7, 12).expect("Invalid date provided");
        let params = Configuration::with(Method::NorthAmerica, Madhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::try_new(local_date, coordinates, params).unwrap();
        let probe = local_date.and_hms_opt(8, 0, 0).unwrap().and_utc();

        assert_eq!(times.current_time(probe), Prayer::Isha);
        // In the gap, current_prayer_time should return isha_yesterday (a time in the past),
        // NOT self.isha (which is in the future during the gap)
        let cpt = times.current_prayer_time_at(probe);
        assert!(
            cpt <= probe,
            "current_prayer_time() during gap should return a time ≤ probe, got {cpt} > {probe}"
        );
        assert_eq!(times.time(Prayer::Isha), times.isha);
        assert!(
            times.isha > probe,
            "self.isha should be in the future during the gap"
        );
    }

    #[test]
    fn next_prayer_progresses_through_the_day() {
        let local_date = NaiveDate::from_ymd_opt(2015, 7, 12).expect("Invalid date provided");
        let params = Configuration::with(Method::NorthAmerica, Madhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::try_new(local_date, coordinates, params).unwrap();

        // Times (UTC): Fajr 08:42, Sunrise 10:08, Dhuhr 17:21, Asr 22:22,
        // Maghrib 00:32 (Jul 13), Isha 01:57, Qiyam 05:59, FajrTomorrow 08:43.
        assert_eq!(
            times.next_at(Utc.with_ymd_and_hms(2015, 7, 12, 9, 0, 0).unwrap()),
            Prayer::Sunrise
        );
        assert_eq!(
            times.next_at(Utc.with_ymd_and_hms(2015, 7, 12, 11, 0, 0).unwrap()),
            Prayer::Dhuhr
        );
        assert_eq!(
            times.next_at(Utc.with_ymd_and_hms(2015, 7, 12, 19, 0, 0).unwrap()),
            Prayer::Asr
        );
        assert_eq!(
            times.next_at(Utc.with_ymd_and_hms(2015, 7, 12, 23, 0, 0).unwrap()),
            Prayer::Maghrib
        );
        assert_eq!(
            times.next_at(Utc.with_ymd_and_hms(2015, 7, 13, 1, 0, 0).unwrap()),
            Prayer::Isha
        );
        assert_eq!(
            times.next_at(Utc.with_ymd_and_hms(2015, 7, 13, 6, 0, 0).unwrap()),
            Prayer::FajrTomorrow
        );
        // Stale schedule (past tomorrow's Fajr): next stays the sentinel.
        assert_eq!(
            times.next_at(Utc.with_ymd_and_hms(2015, 7, 13, 9, 0, 0).unwrap()),
            Prayer::FajrTomorrow
        );
    }

    #[test]
    fn time_remaining_at_is_exact_to_the_second() {
        let local_date = NaiveDate::from_ymd_opt(2015, 7, 12).expect("Invalid date provided");
        let params = Configuration::with(Method::NorthAmerica, Madhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::try_new(local_date, coordinates, params).unwrap();

        // Sunrise at 10:08 UTC, probed 5 minutes before: exactly 5 minutes left.
        let before_sunrise = Utc.with_ymd_and_hms(2015, 7, 12, 10, 3, 0).unwrap();
        assert_eq!(
            times.time_remaining_at(before_sunrise),
            Duration::minutes(5)
        );

        // 1h59m59s before Maghrib (00:32 Jul 13): stays exact, never (1, 60).
        let before_maghrib = Utc.with_ymd_and_hms(2015, 7, 12, 22, 32, 1).unwrap();
        assert_eq!(
            times.time_remaining_at(before_maghrib),
            Duration::seconds(7199)
        );

        // Stale schedule (past tomorrow's Fajr): negative duration reports the drift.
        let stale = Utc.with_ymd_and_hms(2015, 7, 13, 8, 44, 0).unwrap();
        assert_eq!(times.time_remaining_at(stale), Duration::seconds(-60));
    }

    #[test]
    fn calculate_times_for_moonsighting_method() {
        let date = NaiveDate::from_ymd_opt(2016, 1, 31).expect("Invalid date provided");
        let params = Configuration::with(Method::MoonsightingCommittee, Madhab::Shafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let result = PrayerSchedule::new()
            .on(date)
            .for_location(coordinates)
            .with_configuration(params)
            .calculate();

        match result {
            Ok(schedule) => {
                // fajr    = 2016-01-31 10:48:00 UTC
                // sunrise = 2016-01-31 12:16:00 UTC
                // dhuhr   = 2016-01-31 17:33:00 UTC
                // asr     = 2016-01-31 20:20:00 UTC
                // maghrib = 2016-01-31 22:43:00 UTC
                // isha    = 2016-01-31 23:46:00 UTC
                assert_eq!(
                    schedule.time(Prayer::Fajr).format("%-l:%M %p").to_string(),
                    "10:48 AM"
                );
                assert_eq!(
                    schedule
                        .time(Prayer::Sunrise)
                        .format("%-l:%M %p")
                        .to_string(),
                    "12:16 PM"
                );
                assert_eq!(
                    schedule.time(Prayer::Dhuhr).format("%-l:%M %p").to_string(),
                    "5:33 PM"
                );
                assert_eq!(
                    schedule.time(Prayer::Asr).format("%-l:%M %p").to_string(),
                    "8:20 PM"
                );
                assert_eq!(
                    schedule
                        .time(Prayer::Maghrib)
                        .format("%-l:%M %p")
                        .to_string(),
                    "10:43 PM"
                );
                assert_eq!(
                    schedule.time(Prayer::Isha).format("%-l:%M %p").to_string(),
                    "11:46 PM"
                );
            }

            Err(e) => panic!("calculation failed: {e}"),
        }
    }

    #[test]
    fn calculate_times_for_moonsighting_method_with_high_latitude() {
        let date = NaiveDate::from_ymd_opt(2016, 1, 1).expect("Invalid date provided");
        let params = Configuration::with(Method::MoonsightingCommittee, Madhab::Hanafi);
        let coordinates = Coordinates::new(59.9094, 10.7349);
        let result = PrayerSchedule::new()
            .on(date)
            .for_location(coordinates)
            .with_configuration(params)
            .calculate();

        match result {
            Ok(schedule) => {
                // fajr    = 2016-01-01 06:34:00 UTC
                // sunrise = 2016-01-01 08:19:00 UTC
                // dhuhr   = 2016-01-01 11:25:00 UTC
                // asr     = 2016-01-01 12:36:00 UTC
                // maghrib = 2016-01-01 14:25:00 UTC
                // isha    = 2016-01-01 16:02:00 UTC
                assert_eq!(
                    schedule.time(Prayer::Fajr).format("%-l:%M %p").to_string(),
                    "6:34 AM"
                );
                assert_eq!(
                    schedule
                        .time(Prayer::Sunrise)
                        .format("%-l:%M %p")
                        .to_string(),
                    "8:19 AM"
                );
                assert_eq!(
                    schedule.time(Prayer::Dhuhr).format("%-l:%M %p").to_string(),
                    "11:25 AM"
                );
                assert_eq!(
                    schedule.time(Prayer::Asr).format("%-l:%M %p").to_string(),
                    "12:36 PM"
                );
                assert_eq!(
                    schedule
                        .time(Prayer::Maghrib)
                        .format("%-l:%M %p")
                        .to_string(),
                    "2:25 PM"
                );
                assert_eq!(
                    schedule.time(Prayer::Isha).format("%-l:%M %p").to_string(),
                    "4:02 PM"
                );
            }

            Err(e) => panic!("calculation failed: {e}"),
        }
    }

    #[test]
    fn lre_prayer_times_brussels_summer_solstice() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
        let coordinates = Coordinates::new(50.85, 4.35);

        let lre_params = Configuration::new(18.0, 17.0)
            .high_latitude_rule(HighLatitudeRule::LocalRelativeEstimation)
            .done();

        let times = PrayerTimes::try_new(date, coordinates, lre_params).unwrap();

        assert!(
            times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
            "Fajr must be before Sunrise"
        );
        assert!(
            times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
            "Sunrise must be before Dhuhr"
        );
        assert!(
            times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
            "Dhuhr must be before Asr"
        );
        assert!(
            times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
            "Asr must be before Maghrib"
        );
        assert!(
            times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
            "Maghrib must be before Isha"
        );
    }

    #[test]
    fn lre_prayer_times_oslo_winter() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid date");
        let coordinates = Coordinates::new(59.9094, 10.7349);

        let lre_params = Configuration::new(18.0, 17.0)
            .high_latitude_rule(HighLatitudeRule::LocalRelativeEstimation)
            .done();

        let times = PrayerTimes::try_new(date, coordinates, lre_params).unwrap();

        assert!(
            times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
            "Fajr must be before Sunrise"
        );
        assert!(
            times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
            "Sunrise must be before Dhuhr"
        );
        assert!(
            times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
            "Dhuhr must be before Asr"
        );
        assert!(
            times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
            "Asr must be before Maghrib"
        );
        assert!(
            times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
            "Maghrib must be before Isha"
        );
    }

    #[test]
    fn lre_unit_variant_brussels_july_25() {
        let date = NaiveDate::from_ymd_opt(2026, 7, 25).expect("valid date");
        let coordinates = Coordinates::new(50.85, 4.35);

        let params = Configuration::new(18.0, 17.0)
            .high_latitude_rule(HighLatitudeRule::LocalRelativeEstimation)
            .done();

        let times = PrayerTimes::try_new(date, coordinates, params).unwrap();

        assert!(
            times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
            "Fajr must be before Sunrise"
        );
        assert!(
            times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
            "Sunrise must be before Dhuhr"
        );
        assert!(
            times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
            "Dhuhr must be before Asr"
        );
        assert!(
            times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
            "Asr must be before Maghrib"
        );
        assert!(
            times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
            "Maghrib must be before Isha"
        );
    }

    #[test]
    fn maghrib_uses_geometric_sunset_when_angle_is_zero() {
        let prayer_date = NaiveDate::from_ymd_opt(2015, 7, 12)
            .expect("valid date")
            .and_hms_opt(0, 0, 0)
            .expect("valid time")
            .and_utc();
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let solar = SolarTime::new(prayer_date, coordinates).unwrap();
        let params = Parameters::new(18.0, 18.0);

        let maghrib = PrayerTimes::calculate_maghrib(params, solar);

        assert_eq!(maghrib, solar.sunset.unwrap().adjust_time(0));
    }

    #[test]
    fn maghrib_uses_angle_when_set() {
        let prayer_date = NaiveDate::from_ymd_opt(2015, 7, 12)
            .expect("valid date")
            .and_hms_opt(0, 0, 0)
            .expect("valid time")
            .and_utc();
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let solar = SolarTime::new(prayer_date, coordinates).unwrap();
        let params = Configuration::new(18.0, 18.0).maghrib_angle(4.5).done();

        let maghrib = PrayerTimes::calculate_maghrib(params, solar);

        assert_eq!(
            maghrib,
            solar
                .time_for_solar_angle(Angle::new(-4.5), true)
                .unwrap()
                .adjust_time(0)
        );
    }

    #[test]
    fn maghrib_falls_back_to_sunset_when_angle_unreachable() {
        let prayer_date = NaiveDate::from_ymd_opt(2015, 6, 21)
            .expect("valid date")
            .and_hms_opt(0, 0, 0)
            .expect("valid time")
            .and_utc();
        let coordinates = Coordinates::new(65.0, 0.0);
        let solar = SolarTime::new(prayer_date, coordinates).unwrap();
        let params = Configuration::new(18.0, 18.0).maghrib_angle(4.5).done();

        assert!(solar.time_for_solar_angle(Angle::new(-4.5), true).is_none());

        let maghrib = PrayerTimes::calculate_maghrib(params, solar);

        assert_eq!(maghrib, solar.sunset.unwrap().adjust_time(0));
    }

    #[test]
    fn tehran_maghrib_uses_angle_after_sunset() {
        let date = NaiveDate::from_ymd_opt(2015, 7, 12).expect("valid date");
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let tehran_params = Configuration::with(Method::Tehran, Madhab::Shafi);
        let sunset_params = Configuration::new(18.0, 18.0).done();

        let tehran_times = PrayerTimes::try_new(date, coordinates, tehran_params).unwrap();
        let sunset_times = PrayerTimes::try_new(date, coordinates, sunset_params).unwrap();

        let delta_minutes = tehran_times
            .time(Prayer::Maghrib)
            .signed_duration_since(sunset_times.time(Prayer::Maghrib))
            .num_minutes();

        assert!(
            (15..=25).contains(&delta_minutes),
            "Tehran Maghrib should be about 15-25 min after geometric sunset, was {delta_minutes}"
        );
    }
}
