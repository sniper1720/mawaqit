use chrono::{DateTime, Duration, Utc};

use crate::astronomy::solar::SolarTime;
use crate::astronomy::unit::Coordinates;
use crate::models::madhab::Madhab;

/// Strategy for computing prayer times when the sun never rises or sets
/// (polar day/night above ~66.6° N/S).
///
/// Only latitude is substituted — original longitude is always kept.
/// This means two polar cities at the same latitude but different
/// longitudes get different fallback times.
///
/// Absence of a strategy is represented by `Parameters::polar_estimation`
/// being `None`: the true latitude is used, and
/// [`crate::schedule::PrayerTimes::try_new`] returns an error when
/// sunrise or sunset (or the Asr shadow angle) does not occur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PolarEstimation {
    /// Scan south along the same longitude until a latitude with normal
    /// sunrise and sunset is found.
    NearestLatitude,
    /// Fixed 45°N/S, original longitude kept.
    Reference45,
}

impl PolarEstimation {
    /// Resolve the effective latitude for solar calculations.
    ///
    /// When the original coordinates already produce a valid sunrise,
    /// sunset, and Asr shadow on `date` and the adjacent days, the
    /// original latitude is returned unchanged; otherwise the strategy
    /// is applied.
    #[must_use]
    pub fn resolve_latitude(
        self,
        date: DateTime<Utc>,
        coordinates: Coordinates,
        madhab: Madhab,
    ) -> f64 {
        let shadow = madhab.shadow() as f64;
        let asr_reachable = |st: SolarTime| st.time_for_shadow(shadow).is_some();
        let yesterday = date - Duration::days(1);
        let tomorrow = date + Duration::days(1);

        // True when the original latitude is not on a reappearance/disappearance
        // boundary — yesterday and tomorrow also have valid sunrise/sunset.
        // Mirrors MWL 2009's concept of an *undisturbed* day (see compute_isha_night_ratio).
        fn undisturbed(
            date: DateTime<Utc>,
            yesterday: DateTime<Utc>,
            tomorrow: DateTime<Utc>,
            coordinates: Coordinates,
            asr_reachable: impl Fn(SolarTime) -> bool,
        ) -> bool {
            SolarTime::new(date, coordinates).is_ok_and(&asr_reachable)
                && SolarTime::new(yesterday, coordinates).is_ok()
                && SolarTime::new(tomorrow, coordinates).is_ok()
        }

        match self {
            Self::NearestLatitude => {
                if undisturbed(date, yesterday, tomorrow, coordinates, asr_reachable) {
                    coordinates.latitude
                } else {
                    nearest_working_latitude(date, &coordinates, shadow)
                }
            }
            Self::Reference45 => {
                if undisturbed(date, yesterday, tomorrow, coordinates, asr_reachable) {
                    coordinates.latitude
                } else {
                    45.0 * coordinates.latitude.signum()
                }
            }
        }
    }
}

/// Binary search along the same longitude toward the equator for a
/// latitude where [`SolarTime::new()`] succeeds and [`time_for_shadow`]
/// returns a valid Asr (sun above the geometric horizon at Asr time).
/// First tries to find a latitude where yesterday and tomorrow also
/// work.  If the boundary is too tight (declination shifts ±0.02°/day),
/// falls back to today-only.
fn nearest_working_latitude(date: DateTime<Utc>, coordinates: &Coordinates, shadow: f64) -> f64 {
    fn check(
        date: DateTime<Utc>,
        coordinates: &Coordinates,
        shadow: f64,
        require_adjacent: bool,
    ) -> Option<f64> {
        let latitude_sign = coordinates.latitude.signum();
        let mut lower_bound = 0.0_f64;
        let mut upper_bound = coordinates.latitude.abs();
        let yesterday = date - Duration::days(1);
        let tomorrow = date + Duration::days(1);

        for _ in 0..24 {
            let probe_magnitude = (lower_bound + upper_bound) / 2.0;
            let probe_coordinates =
                Coordinates::new(probe_magnitude * latitude_sign, coordinates.longitude);

            let today_ok = SolarTime::new(date, probe_coordinates)
                .is_ok_and(|st| st.time_for_shadow(shadow).is_some());

            let ok = if require_adjacent {
                today_ok
                    && SolarTime::new(yesterday, probe_coordinates).is_ok()
                    && SolarTime::new(tomorrow, probe_coordinates).is_ok()
            } else {
                today_ok
            };

            if ok {
                lower_bound = probe_magnitude;
            } else {
                upper_bound = probe_magnitude;
            }
        }
        // Only return a result if we actually moved off zero (found
        // at least one valid latitude).
        if lower_bound > 0.0 {
            Some(lower_bound * latitude_sign)
        } else {
            None
        }
    }

    check(date, coordinates, shadow, true)
        .or_else(|| check(date, coordinates, shadow, false))
        .unwrap_or(0.0)
}
