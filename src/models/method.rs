use super::adjustments::Adjustment;
use super::parameters::{Configuration, Parameters};
use super::rounding::Rounding;

/// Provides preset configuration for a few authorities
/// for calculating prayer times.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Method {
    /// Muslim World League. Standard Fajr time with an angle of 18°.
    /// Earlier Isha time with an angle of 17°.
    MuslimWorldLeague,

    /// Egyptian General Authority of Survey. Early Fajr time using an angle of 19.5°
    /// and a slightly earlier Isha time using an angle of 17.5°.
    Egyptian,

    /// University of Islamic Sciences, Karachi. A generally applicable method that
    /// uses standard Fajr and Isha angles of 18°.
    Karachi,

    /// Umm al-Qura University, Makkah. Uses a fixed interval of 90 minutes
    /// from Maghrib to calculate Isha, and a slightly earlier Fajr time with
    /// an angle of 18.5°. Note: you should add a +30 minute custom adjustment
    /// for Isha during Ramadan.
    UmmAlQura,

    /// Used in the UAE. Slightly earlier Fajr time and slightly later Isha
    /// time with angles of 18.2° for Fajr and Isha in addition to 3 minute
    /// offsets for sunrise, Dhuhr, Asr, and Maghrib.
    Dubai,

    /// Method developed by Khalid Shaukat, founder of Moonsighting Committee Worldwide.
    /// Uses standard 18° angles for Fajr and Isha in addition to seasonal adjustment values.
    /// This method automatically applies the 1/7 approximation rule for locations above 55°
    /// latitude. Recommended for North America and the UK.
    MoonsightingCommittee,

    /// Also known as the ISNA method. Can be used for North America,
    /// but the MoonsightingCommittee method is preferable. Gives later Fajr times and early
    /// Isha times with angles of 15°.
    NorthAmerica,

    /// Standard Fajr time with an angle of 18°. Slightly earlier Isha time with an angle of 17.5°.
    Kuwait,

    /// Same Isha interval as `ummAlQura` but with the standard Fajr time using an angle of 18°.
    Qatar,

    /// Used in Singapore, Malaysia, and Indonesia. Early Fajr time with an angle of 20°
    /// and standard Isha time with an angle of 18°.
    Singapore,

    /// Institute of Geophysics, University of Tehran. Early Isha time with an angle of 14°.
    /// Slightly later Fajr time with an angle of 17.7°. Calculates Maghrib based on the sun
    /// reaching an angle of 4.5° below the horizon.
    Tehran,

    /// An approximation of the Diyanet method used in Turkey.
    /// This approximation is less accurate outside the region of Turkey.
    Turkey,

    /// Union des Organisations Islamiques de France (UOIF / ECFR).
    /// Standard Fajr time with an angle of 12° and Isha time with an angle of 12°.
    France,

    /// Algerian Ministry of Religious Affairs.
    /// Standard Fajr time with an angle of 18°, Isha time with an angle of 17°,
    /// and a +3 minute adjustment for Maghrib.
    Algeria,

    /// Defaults to angles of 0°, should generally be used for making a custom method
    /// and setting your own values.
    Other,
}

impl Method {
    /// Build the default [`Parameters`] for this calculation method.
    #[must_use]
    pub fn parameters(&self) -> Parameters {
        match self {
            Self::MuslimWorldLeague => Configuration::new(18.0, 17.0)
                .method(*self)
                .method_adjustments(Adjustment::new().dhuhr(1).done())
                .done(),

            Self::Egyptian => Configuration::new(19.5, 17.5)
                .method(*self)
                .method_adjustments(Adjustment::new().dhuhr(1).done())
                .done(),

            Self::Karachi => Configuration::new(18.0, 18.0)
                .method(*self)
                .method_adjustments(Adjustment::new().dhuhr(1).done())
                .done(),

            Self::UmmAlQura => Configuration::new(18.5, 0.0)
                .method(*self)
                .isha_interval(90)
                .done(),
            Self::Dubai => Configuration::new(18.2, 18.2)
                .method(*self)
                .method_adjustments(
                    Adjustment::new()
                        .sunrise(-3)
                        .dhuhr(3)
                        .asr(3)
                        .maghrib(3)
                        .done(),
                )
                .done(),

            Self::MoonsightingCommittee => Configuration::new(18.0, 18.0)
                .method(*self)
                .method_adjustments(Adjustment::new().dhuhr(5).maghrib(3).done())
                .done(),

            Self::NorthAmerica => Configuration::new(15.0, 15.0)
                .method(*self)
                .method_adjustments(Adjustment::new().dhuhr(1).done())
                .done(),

            Self::Kuwait => Configuration::new(18.0, 17.5).method(*self).done(),

            Self::Qatar => Configuration::new(18.0, 0.0)
                .method(*self)
                .isha_interval(90)
                .done(),

            Self::Singapore => Configuration::new(20.0, 18.0)
                .method(*self)
                .method_adjustments(Adjustment::new().dhuhr(1).done())
                .rounding(Rounding::Up)
                .done(),

            Self::Tehran => Configuration::new(17.7, 14.0)
                .method(*self)
                .maghrib_angle(4.5)
                .done(),

            Self::Turkey => Configuration::new(18.0, 17.0)
                .method(*self)
                .method_adjustments(
                    Adjustment::new()
                        .sunrise(-7)
                        .dhuhr(5)
                        .asr(4)
                        .maghrib(7)
                        .done(),
                )
                .done(),

            Self::France => Configuration::new(12.0, 12.0).method(*self).done(),

            Self::Algeria => Configuration::new(18.0, 17.0)
                .method(*self)
                .method_adjustments(Adjustment::new().maghrib(3).done())
                .done(),

            Self::Other => Configuration::new(0.0, 0.0).method(*self).done(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters_for_muslim_world_league() {
        let method = Method::MuslimWorldLeague;
        let params = method.parameters();

        assert_eq!(params.method, Method::MuslimWorldLeague);
        assert_eq!(params.fajr_angle, 18.0);
        assert_eq!(params.isha_angle, 17.0);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn parameters_for_egyptian() {
        let method = Method::Egyptian;
        let params = method.parameters();

        assert_eq!(params.method, Method::Egyptian);
        assert_eq!(params.fajr_angle, 19.5);
        assert_eq!(params.isha_angle, 17.5);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn parameters_for_karachi() {
        let method = Method::Karachi;
        let params = method.parameters();

        assert_eq!(params.method, Method::Karachi);
        assert_eq!(params.fajr_angle, 18.0);
        assert_eq!(params.isha_angle, 18.0);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn parameters_for_umm_al_qura() {
        let method = Method::UmmAlQura;
        let params = method.parameters();

        assert_eq!(params.method, Method::UmmAlQura);
        assert_eq!(params.fajr_angle, 18.5);
        assert_eq!(params.isha_angle, 0.0);
        assert_eq!(params.isha_interval, 90);
    }

    #[test]
    fn parameters_for_dubai() {
        let method = Method::Dubai;
        let params = method.parameters();

        assert_eq!(params.method, Method::Dubai);
        assert_eq!(params.fajr_angle, 18.2, "Parameters: {:?}", params);
        assert_eq!(params.isha_angle, 18.2);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn parameters_for_moonsighting_committee() {
        let method = Method::MoonsightingCommittee;
        let params = method.parameters();

        assert_eq!(params.method, Method::MoonsightingCommittee);
        assert_eq!(params.fajr_angle, 18.0);
        assert_eq!(params.isha_angle, 18.0);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn parameters_for_north_america() {
        let method = Method::NorthAmerica;
        let params = method.parameters();

        assert_eq!(params.method, Method::NorthAmerica);
        assert_eq!(params.fajr_angle, 15.0);
        assert_eq!(params.isha_angle, 15.0);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn parameters_for_kuwait() {
        let method = Method::Kuwait;
        let params = method.parameters();

        assert_eq!(params.method, Method::Kuwait);
        assert_eq!(params.fajr_angle, 18.0);
        assert_eq!(params.isha_angle, 17.5);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn parameters_for_qatar() {
        let method = Method::Qatar;
        let params = method.parameters();

        assert_eq!(params.method, Method::Qatar);
        assert_eq!(params.fajr_angle, 18.0);
        assert_eq!(params.isha_angle, 0.0);
        assert_eq!(params.isha_interval, 90);
    }

    #[test]
    fn parameters_for_singapore() {
        let method = Method::Singapore;
        let params = method.parameters();

        assert_eq!(params.method, Method::Singapore);
        assert_eq!(params.fajr_angle, 20.0);
        assert_eq!(params.isha_angle, 18.0);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn parameters_for_other() {
        let method = Method::Other;
        let params = method.parameters();

        assert_eq!(params.method, Method::Other);
        assert_eq!(params.fajr_angle, 0.0);
        assert_eq!(params.isha_angle, 0.0);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn parameters_for_tehran() {
        let method = Method::Tehran;
        let params = method.parameters();

        assert_eq!(params.method, Method::Tehran);
        assert_eq!(params.fajr_angle, 17.7);
        assert_eq!(params.isha_angle, 14.0);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn parameters_for_turkey() {
        let method = Method::Turkey;
        let params = method.parameters();

        assert_eq!(params.method, Method::Turkey);
        assert_eq!(params.fajr_angle, 18.0);
        assert_eq!(params.isha_angle, 17.0);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn parameters_for_france() {
        let method = Method::France;
        let params = method.parameters();

        assert_eq!(params.method, Method::France);
        assert_eq!(params.fajr_angle, 12.0);
        assert_eq!(params.isha_angle, 12.0);
        assert_eq!(params.isha_interval, 0);
    }

    #[test]
    fn parameters_for_algeria() {
        let method = Method::Algeria;
        let params = method.parameters();

        assert_eq!(params.method, Method::Algeria);
        assert_eq!(params.fajr_angle, 18.0);
        assert_eq!(params.isha_angle, 17.0);
        assert_eq!(params.isha_interval, 0);
        assert_eq!(params.method_adjustments.maghrib, 3);
    }
}
