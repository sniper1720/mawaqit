use chrono::{DateTime, Duration, Utc};

use crate::astronomy::ops;
use crate::astronomy::unit::Stride;
use crate::models::rounding::Rounding;
use crate::models::shafaq::Shafaq;

/// Fajr from the seasonal morning-twilight (dawn) function for the
/// Moonsighting Committee method, applied to the given sunrise.
pub fn season_adjusted_morning_twilight(
    latitude: f64,
    day: u32,
    year: u32,
    sunrise: DateTime<Utc>,
) -> DateTime<Utc> {
    let solstice_days = days_since_solstice(day, year, latitude) as f64;
    let adjustment = twilight_adjustments(
        AdjustmentDaytime::Morning,
        latitude,
        solstice_days,
        Shafaq::General,
    );

    let rounded_adjustment = (adjustment * -60.0).round() as i64;
    sunrise
        .checked_add_signed(Duration::seconds(rounded_adjustment))
        .expect("morning twilight adjustment overflowed")
}

/// Isha from the seasonal evening-twilight (shafaq) function for the
/// Moonsighting Committee method, applied to the given sunset.
pub fn season_adjusted_evening_twilight(
    latitude: f64,
    day: u32,
    year: u32,
    sunset: DateTime<Utc>,
    shafaq: Shafaq,
) -> DateTime<Utc> {
    let solstice_days = days_since_solstice(day, year, latitude) as f64;
    let adjustment =
        twilight_adjustments(AdjustmentDaytime::Evening, latitude, solstice_days, shafaq);

    let rounded_adjustment = (adjustment * 60.0).round() as i64;
    let adjusted_date = sunset
        .checked_add_signed(Duration::seconds(rounded_adjustment))
        .expect("evening twilight adjustment overflowed");

    adjusted_date.rounded_minute(Rounding::Nearest)
}

/// Solstice calculation to determine a date's seasonal progression.
///
/// Used in the Moonsighting Committee calculation method. Counts days
/// forward from the solstice that anchors the seasonal function, matching
/// the method's DYY definition: the December solstice for the northern
/// hemisphere, the June solstice for the southern hemisphere (DYY = 0 at
/// the solstice).
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
        if day_of_year >= southern_offset {
            day_of_year - southern_offset
        } else {
            day_of_year + days_in_year - southern_offset
        }
    }
}

fn twilight_adjustments(
    daytime: AdjustmentDaytime,
    latitude: f64,
    days_since_solstice: f64,
    shafaq: Shafaq,
) -> f64 {
    let adjustment_values = twilight_adjustment_values(daytime, latitude, shafaq);

    if (0.00..=90.0).contains(&days_since_solstice) {
        adjustment_values.december_solstice
            + (adjustment_values.equinox - adjustment_values.december_solstice) / 91.0
                * days_since_solstice
    } else if (91.0..=136.0).contains(&days_since_solstice) {
        adjustment_values.equinox
            + (adjustment_values.cross_quarter - adjustment_values.equinox) / 46.0
                * (days_since_solstice - 91.0)
    } else if (137.0..=182.0).contains(&days_since_solstice) {
        adjustment_values.cross_quarter
            + (adjustment_values.june_solstice - adjustment_values.cross_quarter) / 46.0
                * (days_since_solstice - 137.0)
    } else if (183.0..=228.0).contains(&days_since_solstice) {
        adjustment_values.june_solstice
            + (adjustment_values.cross_quarter - adjustment_values.june_solstice) / 46.0
                * (days_since_solstice - 183.0)
    } else if (229.0..=274.0).contains(&days_since_solstice) {
        adjustment_values.cross_quarter
            + (adjustment_values.equinox - adjustment_values.cross_quarter) / 46.0
                * (days_since_solstice - 229.0)
    } else {
        adjustment_values.equinox
            + (adjustment_values.december_solstice - adjustment_values.equinox) / 91.0
                * (days_since_solstice - 275.0)
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum AdjustmentDaytime {
    Morning,
    Evening,
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct TwilightAdjustmentValues {
    december_solstice: f64,
    equinox: f64,
    cross_quarter: f64,
    june_solstice: f64,
}

fn twilight_adjustment_values(
    daytime: AdjustmentDaytime,
    latitude: f64,
    shafaq: Shafaq,
) -> TwilightAdjustmentValues {
    if daytime == AdjustmentDaytime::Morning {
        TwilightAdjustmentValues {
            december_solstice: 75.0 + ((28.65 / 55.0) * latitude.abs()),
            equinox: 75.0 + ((19.44 / 55.0) * latitude.abs()),
            cross_quarter: 75.0 + ((32.74 / 55.0) * latitude.abs()),
            june_solstice: 75.0 + ((48.10 / 55.0) * latitude.abs()),
        }
    } else {
        match shafaq {
            Shafaq::General => TwilightAdjustmentValues {
                december_solstice: 75.0 + ((25.60 / 55.0) * latitude.abs()),
                equinox: 75.0 + ((2.050 / 55.0) * latitude.abs()),
                cross_quarter: 75.0 - ((9.210 / 55.0) * latitude.abs()),
                june_solstice: 75.0 + ((6.140 / 55.0) * latitude.abs()),
            },
            Shafaq::Ahmer => TwilightAdjustmentValues {
                december_solstice: 62.0 + ((17.40 / 55.0) * latitude.abs()),
                equinox: 62.0 - ((7.160 / 55.0) * latitude.abs()),
                cross_quarter: 62.0 + ((5.120 / 55.0) * latitude.abs()),
                june_solstice: 62.0 + ((19.44 / 55.0) * latitude.abs()),
            },
            Shafaq::Abyad => TwilightAdjustmentValues {
                december_solstice: 75.0 + ((25.60 / 55.0) * latitude.abs()),
                equinox: 75.0 + ((7.160 / 55.0) * latitude.abs()),
                cross_quarter: 75.0 + ((36.84 / 55.0) * latitude.abs()),
                june_solstice: 75.0 + ((81.84 / 55.0) * latitude.abs()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::days_since_solstice;

    #[test]
    fn days_since_solstice_matches_the_method_definition() {
        // Northern hemisphere: DYY starts at the December solstice
        // (DYY = 0 for December 21, counting forward).
        assert_eq!(days_since_solstice(355, 2026, 60.0), 0); // Dec 21
        assert_eq!(days_since_solstice(1, 2026, 60.0), 11); // Jan 1
        assert_eq!(days_since_solstice(172, 2026, 60.0), 182); // Jun 21
        assert_eq!(days_since_solstice(365, 2026, 60.0), 10); // Dec 31 (wraps the year boundary)

        // Southern hemisphere: DYY starts at the June solstice
        // (DYY = 0 for June 21, counting forward).
        assert_eq!(days_since_solstice(172, 2026, -60.0), 0); // Jun 21
        assert_eq!(days_since_solstice(355, 2026, -60.0), 183); // Dec 21
        assert_eq!(days_since_solstice(1, 2026, -60.0), 194); // Jan 1, wraps before the offset
        assert_eq!(days_since_solstice(171, 2026, -60.0), 364); // Jun 20, last day of the cycle

        // Leap years shift the southern offset by one day.
        assert_eq!(days_since_solstice(173, 2024, -60.0), 0); // Jun 21
        assert_eq!(days_since_solstice(172, 2024, -60.0), 365); // Jun 20
    }
}
