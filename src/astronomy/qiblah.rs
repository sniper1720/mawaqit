use std::fmt;

use crate::astronomy::unit::{Angle, Coordinates};

/// Direction to the Kaaba from a given location, in degrees from true north.
#[derive(Debug)]
pub struct Qiblah(f64);

impl Qiblah {
    /// Calculate the Qibla direction for the given coordinates.
    ///
    /// Equation from "Spherical Trigonometry For the use
    /// of colleges and schools" page 50.
    #[must_use]
    pub fn new(location_coordinates: Coordinates) -> Self {
        let makkah_coordinates = Coordinates::new(21.4225241, 39.8261818);
        let term1 = (makkah_coordinates.longitude_angle().radians()
            - location_coordinates.longitude_angle().radians())
        .sin();
        let term2 = makkah_coordinates.latitude_angle().radians().tan()
            * location_coordinates.latitude_angle().radians().cos();
        let term3 = (makkah_coordinates.longitude_angle().radians()
            - location_coordinates.longitude_angle().radians())
        .cos()
            * location_coordinates.latitude_angle().radians().sin();
        let term4 = term1.atan2(term2 - term3);

        Qiblah(Angle::from_radians(term4).unwound().degrees)
    }

    /// Return the Qibla direction in degrees clockwise from true north.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for Qiblah {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn qiblah_direction_from_nyc_in_north_america() {
        let nyc = Coordinates::new(40.7128, -74.0059);
        let qiblah = Qiblah::new(nyc);

        assert!((qiblah.value() - 58.4817635).abs() < 1e-7);
    }

    #[test]
    fn qiblah_direction_from_sf_in_north_america() {
        let sf = Coordinates::new(37.7749, -122.4194);
        let qiblah = Qiblah::new(sf);

        assert!((qiblah.value() - 18.843822245692426).abs() < 1e-7);
    }

    #[test]
    fn qiblah_direction_from_dc_in_north_america() {
        let dc = Coordinates::new(38.9072, -77.0369);
        let qiblah = Qiblah::new(dc);

        assert!((qiblah.value() - 56.56046821463599).abs() < 1e-7);
    }

    #[test]
    fn qiblah_direction_from_anchorage_in_north_america() {
        let anchorage = Coordinates::new(61.2181, -149.9003);
        let qiblah = Qiblah::new(anchorage);

        assert!((qiblah.value() - 350.8830761159853).abs() < 1e-7);
    }

    #[test]
    fn qiblah_direction_from_sydney_australia() {
        let sydney = Coordinates::new(-33.8688, 151.2093);
        let qiblah = Qiblah::new(sydney);

        assert!((qiblah.value() - 277.4996044487399).abs() < 1e-7);
    }

    #[test]
    fn qiblah_direction_from_auckland_new_zealand() {
        let auckland = Coordinates::new(-36.8485, 174.7633);
        let qiblah = Qiblah::new(auckland);

        assert!((qiblah.value() - 261.19732640365845).abs() < 1e-7);
    }

    #[test]
    fn qiblah_direction_from_london_united_kingdom() {
        let london = Coordinates::new(51.5074, -0.1278);
        let qiblah = Qiblah::new(london);

        assert!((qiblah.value() - 118.9872189).abs() < 1e-7);
    }

    #[test]
    fn qiblah_direction_from_paris_france() {
        let paris = Coordinates::new(48.8566, 2.3522);
        let qiblah = Qiblah::new(paris);

        assert!((qiblah.value() - 119.16313542183347).abs() < 1e-7);
    }

    #[test]
    fn qiblah_direction_from_oslo_norway() {
        let oslo = Coordinates::new(59.9139, 10.7522);
        let qiblah = Qiblah::new(oslo);

        assert!((qiblah.value() - 139.02785605537514).abs() < 1e-7);
    }

    #[test]
    fn qiblah_direction_from_islamabad_pakistan() {
        let islamabad = Coordinates::new(33.7294, 73.0931);
        let qiblah = Qiblah::new(islamabad);

        assert!((qiblah.value() - 255.8816156785436).abs() < 1e-7);
    }

    #[test]
    fn qiblah_direction_from_tokyo_japan() {
        let tokyo = Coordinates::new(35.6895, 139.6917);
        let qiblah = Qiblah::new(tokyo);

        assert!((qiblah.value() - 293.02072441441163).abs() < 1e-7);
    }

    #[test]
    fn qiblah_direction_from_jakarta_indonesia() {
        let jakarta = Coordinates::new(-6.18233995, 106.84287154);
        let qiblah = Qiblah::new(jakarta);

        assert!((qiblah.value() - 295.1442983825265).abs() < 1e-7);
    }

    #[test]
    fn qiblah_direction_display() {
        let nyc = Coordinates::new(40.7128, -74.0059);
        let qiblah = Qiblah::new(nyc);
        let actual_value = qiblah.to_string();

        assert!(actual_value.contains("58.4817635"));
    }
}
