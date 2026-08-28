use chrono::{DateTime, Duration, Utc};

use crate::astronomy::solar::SolarTime;
use crate::astronomy::unit::Coordinates;
use crate::models::madhab::Madhab;

/// Strategy for computing prayer times when the sun never rises or sets
/// (polar day/night above ~66.6° N/S).
///
/// Only latitude is substituted — original longitude is always kept.
/// This means two polar cities at the same latitude but different
/// longitudes get different estimation times.
///
/// Absence of a strategy is represented by `Parameters::polar_estimation`
/// being `None`: the true latitude is used, and
/// [`crate::schedule::PrayerTimes::try_new`] returns an error when
/// sunrise or sunset (or the Asr shadow angle) does not occur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PolarEstimation {
    /// Walk toward the equator along the same longitude until a latitude
    /// with normal sunrise and sunset is found.
    NearestLatitude,
    /// Fixed 45°N/S, original longitude kept.
    Reference45,
}

impl PolarEstimation {
    /// Resolve the reference latitude for solar calculations.
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

        // True when the original latitude is not on a reappearance/disappearance
        // boundary — today supports the full schedule and the adjacent days
        // still have valid sunrise/sunset. Mirrors MWL 2009's concept of an
        // *undisturbed* day (see compute_isha_night_ratio).
        let undisturbed = three_day_window_is_normal(date, &coordinates, shadow);

        match self {
            Self::NearestLatitude => {
                if undisturbed {
                    coordinates.latitude
                } else {
                    nearest_working_latitude(date, &coordinates, shadow)
                }
            }
            Self::Reference45 => {
                if undisturbed {
                    coordinates.latitude
                } else {
                    45.0 * coordinates.latitude.signum()
                }
            }
        }
    }
}

/// True when these coordinates support a complete schedule on `date`:
/// sunrise and sunset occur through the engine's refraction horizon,
/// and the Asr shadow angle is reachable.
#[must_use]
fn latitude_supports_schedule(date: DateTime<Utc>, coordinates: &Coordinates, shadow: f64) -> bool {
    SolarTime::new(date, *coordinates).is_ok_and(|st| st.time_for_shadow(shadow).is_some())
}

/// True when both adjacent days have valid sunrise and sunset.
#[must_use]
fn adjacent_days_have_rise_and_set(date: DateTime<Utc>, coordinates: &Coordinates) -> bool {
    SolarTime::new(date - Duration::days(1), *coordinates).is_ok()
        && SolarTime::new(date + Duration::days(1), *coordinates).is_ok()
}

/// True when `date` supports a complete schedule at these coordinates —
/// sunrise/sunset exist and the Asr shadow angle is reachable — and both
/// adjacent days have valid sunrise/sunset. That is exactly the window a
/// three-day [`crate::schedule::SolarTimeSet`] needs: only today feeds
/// Asr, while the neighbors feed the night-span fields.
#[must_use]
pub(crate) fn three_day_window_is_normal(
    date: DateTime<Utc>,
    coordinates: &Coordinates,
    shadow: f64,
) -> bool {
    latitude_supports_schedule(date, coordinates, shadow)
        && adjacent_days_have_rise_and_set(date, coordinates)
}

/// Binary search along the same longitude toward the equator for a
/// latitude where [`SolarTime::new()`] succeeds and the Asr shadow
/// angle is reachable ([`SolarTime::time_for_shadow`] returns a time).
/// First tries to find a latitude where yesterday and tomorrow also
/// work.  If the boundary is too tight (declination shifts ±0.02°/day),
/// falls back to today-only.
pub(crate) fn nearest_working_latitude(
    date: DateTime<Utc>,
    coordinates: &Coordinates,
    shadow: f64,
) -> f64 {
    let latitude_sign = coordinates.latitude.signum();
    let probe = |magnitude: f64| Coordinates::new(magnitude * latitude_sign, coordinates.longitude);

    for require_adjacent in [true, false] {
        let mut lower_bound = 0.0_f64;
        let mut upper_bound = coordinates.latitude.abs();

        for _ in 0..24 {
            let middle = (lower_bound + upper_bound) / 2.0;

            let ok = latitude_supports_schedule(date, &probe(middle), shadow)
                && (!require_adjacent || adjacent_days_have_rise_and_set(date, &probe(middle)));

            if ok {
                lower_bound = middle;
            } else {
                upper_bound = middle;
            }
        }

        // Only return a result if we actually moved off zero (found at
        // least one valid latitude); otherwise retry without requiring
        // undisturbed adjacent days.
        if lower_bound > 0.0 {
            return lower_bound * latitude_sign;
        }
    }

    0.0
}
