use chrono::{DateTime, Duration, Utc};

use crate::astronomy::solar::SolarTime;
use crate::astronomy::unit::Coordinates;
use crate::models::madhab::Madhab;

/// Strategy for computing prayer times when the sun never rises or sets
/// (polar day/night above ~66.5° N/S).
///
/// Only latitude is substituted — original longitude is always kept.
/// This means two polar cities at the same latitude but different
/// longitudes get different fallback times.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum PolarFallback {
    /// Scan south along the same longitude until a latitude with normal
    /// sunrise and sunset is found.
    NearestLatitude,
    /// Fixed 45°N/S, original longitude kept.
    Reference45,
    /// No fallback.  `PrayerTimes::try_new()` returns `Err` at polar.
    #[default]
    None,
}

impl PolarFallback {
    /// Return the recommended [`PolarFallback`] for the given coordinates.
    ///
    /// - `|lat| > 66.5°` → [`NearestLatitude`]
    /// - `|lat| ≤ 66.5°` → [`None`]
    #[must_use]
    pub fn recommended(coordinates: Coordinates) -> Self {
        if coordinates.latitude.abs() > 66.5 {
            Self::NearestLatitude
        } else {
            Self::None
        }
    }

    /// Resolve the effective latitude for solar calculations.
    ///
    /// If [`SolarTime::new()`] succeeds at the original coordinates,
    /// returns the original latitude unchanged (wrapped in `Some`).
    /// Otherwise applies the fallback strategy.
    ///
    /// Returns `None` only for [`PolarFallback::None`] when the original
    /// latitude has no sunrise/sunset (polar day/night).
    #[must_use]
    pub fn resolve_latitude(
        self,
        date: DateTime<Utc>,
        coordinates: Coordinates,
        madhab: Madhab,
    ) -> Option<f64> {
        let shadow = madhab.shadow() as f64;
        let asr_reachable = |st: SolarTime| st.time_for_shadow(shadow).is_some();

        match self {
            Self::NearestLatitude => {
                // Try original latitude first — must have normal sunrise/sunset
                // and Asr above the geometric horizon.
                if SolarTime::new(date, coordinates).is_ok_and(asr_reachable) {
                    Some(coordinates.latitude)
                } else {
                    Some(nearest_working_latitude(date, &coordinates, shadow))
                }
            }
            Self::Reference45 => {
                if SolarTime::new(date, coordinates).is_ok_and(asr_reachable) {
                    Some(coordinates.latitude)
                } else {
                    Some(45.0 * coordinates.latitude.signum())
                }
            }
            Self::None => {
                if SolarTime::new(date, coordinates).is_ok_and(asr_reachable) {
                    Some(coordinates.latitude)
                } else {
                    None
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
fn nearest_working_latitude(date: DateTime<Utc>, coords: &Coordinates, shadow: f64) -> f64 {
    fn check(
        date: DateTime<Utc>,
        coords: &Coordinates,
        shadow: f64,
        require_adjacent: bool,
    ) -> Option<f64> {
        let sign = coords.latitude.signum();
        let mut lo = 0.0_f64;
        let mut hi = coords.latitude.abs();
        let yesterday = date - Duration::days(1);
        let tomorrow = date + Duration::days(1);

        for _ in 0..24 {
            let mid = (lo + hi) / 2.0;
            let test = Coordinates::new(mid * sign, coords.longitude);

            let today_ok =
                SolarTime::new(date, test).is_ok_and(|st| st.time_for_shadow(shadow).is_some());

            let ok = if require_adjacent {
                today_ok
                    && SolarTime::new(yesterday, test).is_ok()
                    && SolarTime::new(tomorrow, test).is_ok()
            } else {
                today_ok
            };

            if ok {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        // Only return a result if we actually moved off zero (found
        // at least one valid latitude).
        if lo > 0.0 { Some(lo * sign) } else { None }
    }

    check(date, coords, shadow, true)
        .or_else(|| check(date, coords, shadow, false))
        .unwrap_or(0.0)
}
