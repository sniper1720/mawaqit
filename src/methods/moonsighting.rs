use chrono::{DateTime, Duration, Utc};

use crate::astronomy::ops;
use crate::astronomy::unit::Stride;
use crate::models::rounding::Rounding;
use crate::models::shafaq::Shafaq;

/// Twilight adjustment based on observational data for use
/// in the Moonsighting Committee calculation method.
pub fn season_adjusted_morning_twilight(
    latitude: f64,
    day: u32,
    year: u32,
    sunrise: DateTime<Utc>,
) -> DateTime<Utc> {
    let dyy = days_since_solstice(day, year, latitude) as f64;
    let adjustment =
        twilight_adjustments(AdjustmentDaytime::Morning, latitude, dyy, Shafaq::General);

    let rounded_adjustment = (adjustment * -60.0).round() as i64;
    sunrise
        .checked_add_signed(Duration::seconds(rounded_adjustment))
        .expect("morning twilight adjustment overflowed")
}

/// Twilight adjustment based on observational data for use
/// in the Moonsighting Committee calculation method.
pub fn season_adjusted_evening_twilight(
    latitude: f64,
    day: u32,
    year: u32,
    sunset: DateTime<Utc>,
    shafaq: Shafaq,
) -> DateTime<Utc> {
    let dyy = days_since_solstice(day, year, latitude) as f64;
    let adjustment = twilight_adjustments(AdjustmentDaytime::Evening, latitude, dyy, shafaq);

    let rounded_adjustment = (adjustment * 60.0).round() as i64;
    let adjusted_date = sunset
        .checked_add_signed(Duration::seconds(rounded_adjustment))
        .expect("evening twilight adjustment overflowed");

    adjusted_date.rounded_minute(Rounding::Nearest)
}

/// Solstice calculation to determine a date's seasonal progression.
///
/// Used in the Moonsighting Committee calculation method.
pub fn days_since_solstice(day_of_year: u32, year: u32, latitude: f64) -> u32 {
    let days_in_year = if ops::is_leap_year(year) { 366 } else { 365 };

    if latitude >= 0.0 {
        let northern_offset = 10;
        let lapsed_days = day_of_year + northern_offset;

        if lapsed_days >= days_in_year {
            lapsed_days - days_in_year
        } else {
            lapsed_days
        }
    } else {
        let southern_offset = if ops::is_leap_year(year) { 173 } else { 172 };
        (day_of_year - southern_offset) + days_in_year
    }
}

fn twilight_adjustments(
    daytime: AdjustmentDaytime,
    latitude: f64,
    dyy: f64,
    shafaq: Shafaq,
) -> f64 {
    let adjustment_values = twilight_adjustment_values(daytime, latitude, shafaq);

    if (0.00..=90.0).contains(&dyy) {
        adjustment_values.a + (adjustment_values.b - adjustment_values.a) / 91.0 * dyy
    } else if (91.0..=136.0).contains(&dyy) {
        adjustment_values.b + (adjustment_values.c - adjustment_values.b) / 46.0 * (dyy - 91.0)
    } else if (137.0..=182.0).contains(&dyy) {
        adjustment_values.c + (adjustment_values.d - adjustment_values.c) / 46.0 * (dyy - 137.0)
    } else if (183.0..=228.0).contains(&dyy) {
        adjustment_values.d + (adjustment_values.c - adjustment_values.d) / 46.0 * (dyy - 183.0)
    } else if (229.0..=274.0).contains(&dyy) {
        adjustment_values.c + (adjustment_values.b - adjustment_values.c) / 46.0 * (dyy - 229.0)
    } else {
        adjustment_values.b + (adjustment_values.a - adjustment_values.b) / 91.0 * (dyy - 275.0)
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum AdjustmentDaytime {
    Morning,
    Evening,
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct TwilightAdjustmentValues {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
}

fn twilight_adjustment_values(
    daytime: AdjustmentDaytime,
    latitude: f64,
    shafaq: Shafaq,
) -> TwilightAdjustmentValues {
    if daytime == AdjustmentDaytime::Morning {
        TwilightAdjustmentValues {
            a: 75.0 + ((28.65 / 55.0) * latitude.abs()),
            b: 75.0 + ((19.44 / 55.0) * latitude.abs()),
            c: 75.0 + ((32.74 / 55.0) * latitude.abs()),
            d: 75.0 + ((48.10 / 55.0) * latitude.abs()),
        }
    } else {
        match shafaq {
            Shafaq::General => TwilightAdjustmentValues {
                a: 75.0 + ((25.60 / 55.0) * latitude.abs()),
                b: 75.0 + ((2.050 / 55.0) * latitude.abs()),
                c: 75.0 - ((9.210 / 55.0) * latitude.abs()),
                d: 75.0 + ((6.140 / 55.0) * latitude.abs()),
            },
            Shafaq::Ahmer => TwilightAdjustmentValues {
                a: 62.0 + ((17.40 / 55.0) * latitude.abs()),
                b: 62.0 - ((7.160 / 55.0) * latitude.abs()),
                c: 62.0 + ((5.120 / 55.0) * latitude.abs()),
                d: 62.0 + ((19.44 / 55.0) * latitude.abs()),
            },
            Shafaq::Abyad => TwilightAdjustmentValues {
                a: 75.0 + ((25.60 / 55.0) * latitude.abs()),
                b: 75.0 + ((7.160 / 55.0) * latitude.abs()),
                c: 75.0 + ((36.84 / 55.0) * latitude.abs()),
                d: 75.0 + ((81.84 / 55.0) * latitude.abs()),
            },
        }
    }
}
