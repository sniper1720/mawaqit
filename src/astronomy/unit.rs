use std::f64::consts::PI;
use std::ops::{Add, Div, Mul, Sub};

use crate::astronomy::ops;
use crate::models::rounding::Rounding;
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike};

/// Normalize a value to a given scale.
pub trait Normalize {
    #[must_use]
    fn normalized_to_scale(&self, modulus: f64) -> f64;
}

impl Normalize for f64 {
    fn normalized_to_scale(&self, modulus: f64) -> f64 {
        self - (modulus * (self / modulus).floor())
    }
}

/// Convenience methods for the DateTime type.
pub trait Stride {
    /// Returns the date/time for tomorrow.
    #[must_use]
    fn tomorrow(&self) -> Self;
    /// Returns the date/time for yesterday.
    #[must_use]
    fn yesterday(&self) -> Self;
    /// Returns the Julian day number.
    #[must_use]
    fn julian_day(&self) -> f64;
    /// Adjust time by the given number of minutes (positive or negative).
    #[must_use]
    fn adjust_time(&self, minutes: i64) -> Self;
    /// Round to the nearest minute using the given rounding rule.
    #[must_use]
    fn rounded_minute(&self, rounding: Rounding) -> Self;
}

impl<Tz: TimeZone> Stride for DateTime<Tz> {
    fn tomorrow(&self) -> Self {
        offset_date(self, 1)
    }

    fn yesterday(&self) -> Self {
        offset_date(self, -1)
    }

    fn julian_day(&self) -> f64 {
        ops::julian_day(self.year(), self.month() as i32, self.day() as i32, 0.0)
    }

    fn rounded_minute(&self, rounding: Rounding) -> Self {
        let adjusted = self.clone();
        let seconds = adjusted.second();

        match rounding {
            Rounding::Nearest => {
                let rounded = ((seconds as f64) / 60.0).round() as i64;
                let adjusted_seconds = seconds as i64;

                if rounded == 1 {
                    adjusted + Duration::seconds(60 - adjusted_seconds)
                } else {
                    adjusted + Duration::seconds(-adjusted_seconds)
                }
            }
            Rounding::Up => {
                let adjusted_seconds = seconds as i64;

                adjusted + Duration::seconds(60 - adjusted_seconds)
            }
            Rounding::None => adjusted,
        }
    }

    fn adjust_time(&self, minutes: i64) -> Self {
        let some_date = self.clone();
        some_date
            .checked_add_signed(Duration::seconds(minutes * 60))
            .expect("time adjustment overflowed")
    }
}

fn offset_date<Tz: TimeZone>(date: &DateTime<Tz>, days: i64) -> DateTime<Tz> {
    let ordinal = date.ordinal() as i64 + days;

    match date.with_ordinal(ordinal as u32) {
        Some(dt) => dt,
        None => {
            if days > 0 {
                date.with_year(date.year() + 1)
                    .expect("year + 1 is always valid")
                    .with_ordinal(1)
                    .expect("ordinal 1 exists in every year")
            } else {
                date.with_year(date.year() - 1)
                    .expect("year - 1 is always valid")
                    .with_month(12)
                    .expect("December exists in every year")
                    .with_day(31)
                    .expect("December 31 exists in every year")
            }
        }
    }
}

/// A value in degrees, with arithmetic operations and conversion helpers.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Angle {
    pub degrees: f64,
}

impl Angle {
    /// Create an [`Angle`] from a value in degrees.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Angle { degrees: value }
    }

    /// Create an [`Angle`] from a value in radians.
    #[must_use]
    pub fn from_radians(value: f64) -> Self {
        Angle {
            degrees: (value * 180.0) / PI,
        }
    }

    /// Return the angle converted from degrees to radians.
    #[must_use]
    pub fn radians(&self) -> f64 {
        (self.degrees * PI) / 180.0
    }

    /// Normalize the angle to the range `[0, 360)` degrees.
    #[must_use]
    pub fn unwound(&self) -> Angle {
        Angle {
            degrees: self.degrees.normalized_to_scale(360.0),
        }
    }

    /// Normalize the angle to the range `[-180, 180]` degrees.
    #[must_use]
    pub fn quadrant_shifted(&self) -> Angle {
        if self.degrees >= -180.0 && self.degrees <= 180.0 {
            *self
        } else {
            let value = self.degrees - (360.0 * (self.degrees / 360.0).round());
            Angle { degrees: value }
        }
    }
}

impl Add for Angle {
    type Output = Angle;

    fn add(self, rhs: Angle) -> Angle {
        Angle {
            degrees: self.degrees + rhs.degrees,
        }
    }
}

impl Sub for Angle {
    type Output = Angle;

    fn sub(self, rhs: Angle) -> Angle {
        Angle {
            degrees: self.degrees - rhs.degrees,
        }
    }
}

impl Mul for Angle {
    type Output = Angle;

    fn mul(self, rhs: Angle) -> Angle {
        Angle {
            degrees: self.degrees * rhs.degrees,
        }
    }
}

impl Div for Angle {
    type Output = Angle;

    fn div(self, rhs: Angle) -> Angle {
        if rhs.degrees == 0.0 {
            panic!("Cannot divide by zero.");
        }

        Angle {
            degrees: self.degrees / rhs.degrees,
        }
    }
}

/// The latitude and longitude associated with a location.
/// Both latitude and longitude values are specified in degrees.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinates {
    /// Create [`Coordinates`] from latitude and longitude in degrees.
    #[must_use]
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Coordinates {
            latitude,
            longitude,
        }
    }
}

impl Coordinates {
    /// Return the latitude as an `Angle`.
    #[must_use]
    pub fn latitude_angle(&self) -> Angle {
        Angle::new(self.latitude)
    }

    /// Return the longitude as an `Angle`.
    #[must_use]
    pub fn longitude_angle(&self) -> Angle {
        Angle::new(self.longitude)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::f64::consts::PI;

    #[test]
    fn angle_conversion_from_radians() {
        assert!((Angle::from_radians(PI).degrees - 180.0).abs() < f64::EPSILON);
        assert!((Angle::from_radians(PI / 2.0).degrees - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn angle_conversion_degrees_to_radians() {
        assert!((Angle::new(180.0).radians() - PI).abs() < f64::EPSILON);
        assert!((Angle::new(90.0).radians() - (PI / 2.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn normalize_value() {
        for (input, scale, expected) in [
            (2.0_f64, -5.0, -3.0),
            (-4.0_f64, -5.0, -4.0),
            (-6.0_f64, -5.0, -1.0),
            (-1.0_f64, 24.0, 23.0),
            (1.0_f64, 24.0, 1.0),
            (49.0_f64, 24.0, 1.0),
            (361.0_f64, 360.0, 1.0),
            (360.0_f64, 360.0, 0.0),
            (259.0_f64, 360.0, 259.0),
            (2592.0_f64, 360.0, 72.0),
        ] {
            let normalized = input.normalized_to_scale(scale);
            assert!(
                (normalized - expected).abs() < f64::EPSILON,
                "got {normalized}, expected {expected}"
            );
        }
    }

    #[test]
    fn angle_unwound() {
        assert!((Angle::new(-45.0).unwound().degrees - 315.0).abs() < f64::EPSILON);
        assert!((Angle::new(361.0).unwound().degrees - 1.0).abs() < f64::EPSILON);
        assert!((Angle::new(360.0).unwound().degrees - 0.0).abs() < f64::EPSILON);
        assert!((Angle::new(259.0).unwound().degrees - 259.0).abs() < f64::EPSILON);
        assert!((Angle::new(2592.0).unwound().degrees - 72.0).abs() < f64::EPSILON);
    }

    #[test]
    fn closest_angle() {
        assert!((Angle::new(360.0).quadrant_shifted().degrees - 0.0).abs() < f64::EPSILON);
        assert!((Angle::new(361.0).quadrant_shifted().degrees - 1.0).abs() < f64::EPSILON);
        assert!((Angle::new(1.0).quadrant_shifted().degrees - 1.0).abs() < f64::EPSILON);
        assert!((Angle::new(-1.0).quadrant_shifted().degrees - (-1.0)).abs() < f64::EPSILON);
        assert!((Angle::new(-181.0).quadrant_shifted().degrees - 179.0).abs() < f64::EPSILON);
        assert!((Angle::new(180.0).quadrant_shifted().degrees - 180.0).abs() < f64::EPSILON);
        assert!((Angle::new(359.0).quadrant_shifted().degrees - (-1.0)).abs() < f64::EPSILON);
        assert!((Angle::new(-359.0).quadrant_shifted().degrees - 1.0).abs() < f64::EPSILON);
        assert!((Angle::new(1261.0).quadrant_shifted().degrees - (-179.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn adding_angles() {
        let angle_a = Angle::new(45.0);
        let angle_b = Angle::new(45.0);

        assert!(((angle_a + angle_b).degrees - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn calculate_rounding_nearest() {
        let time_1 = Utc
            .with_ymd_and_hms(2015, 7, 13, 4, 37, 30)
            .single()
            .expect("Invalid date and time.");

        assert_eq!(
            time_1.rounded_minute(Rounding::Nearest),
            Utc.with_ymd_and_hms(2015, 7, 13, 4, 38, 00)
                .single()
                .unwrap()
        );
    }

    #[test]
    fn calculate_rounding_up() {
        let time_1 = Utc
            .with_ymd_and_hms(2015, 7, 13, 5, 59, 20)
            .single()
            .expect("Invalid date and time.");

        assert_eq!(
            time_1.rounded_minute(Rounding::Up),
            Utc.with_ymd_and_hms(2015, 7, 13, 6, 0, 0).single().unwrap()
        );
    }

    #[test]
    fn calculate_rounding_none() {
        let time_1 = Utc
            .with_ymd_and_hms(2015, 7, 13, 5, 59, 20)
            .single()
            .expect("Invalid date and time.");

        assert_eq!(
            time_1.rounded_minute(Rounding::None),
            Utc.with_ymd_and_hms(2015, 7, 13, 5, 59, 20)
                .single()
                .unwrap()
        );
    }
}
