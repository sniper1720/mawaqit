use crate::astronomy::unit::Coordinates;

/// Rule for approximating Fajr and Isha at high latitudes
#[derive(PartialEq, Debug, Copy, Clone)]
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
}

impl HighLatitudeRule {
    /// Return the recommended [`HighLatitudeRule`] for the given coordinates.
    /// Locations above 48° latitude use `SeventhOfTheNight`; all others use
    /// `MiddleOfTheNight`.
    #[must_use]
    pub fn recommended(coordinates: Coordinates) -> Self {
        if coordinates.latitude > 48.0 {
            Self::SeventhOfTheNight
        } else {
            Self::MiddleOfTheNight
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommended_rule_seventh_of_night() {
        let location = Coordinates::new(48.983226, -3.216649);

        assert_eq!(
            HighLatitudeRule::recommended(location),
            HighLatitudeRule::SeventhOfTheNight
        );
    }

    #[test]
    fn recommended_rule_middle_of_night() {
        let location = Coordinates::new(45.983226, -3.216649);

        assert_eq!(
            HighLatitudeRule::recommended(location),
            HighLatitudeRule::MiddleOfTheNight
        );
    }
}
