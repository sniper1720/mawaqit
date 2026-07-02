use chrono::{Datelike, Utc, Weekday};

/// Names of all obligatory prayers,
/// sunrise, and Qiyam.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Prayer {
    Fajr,
    Sunrise,
    Dhuhr,
    Asr,
    Maghrib,
    Isha,
    Qiyam,
    FajrTomorrow,
}

impl Prayer {
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Fajr | Self::FajrTomorrow => String::from("Fajr"),
            Self::Sunrise => String::from("Sunrise"),
            Self::Dhuhr => {
                if Utc::now().weekday() == Weekday::Fri {
                    String::from("Jumua")
                } else {
                    String::from("Dhuhr")
                }
            }
            Self::Asr => String::from("Asr"),
            Self::Maghrib => String::from("Maghrib"),
            Self::Isha => String::from("Isha"),
            Self::Qiyam => String::from("Qiyam"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prayer_name_for_fajr_en_transliteration() {
        assert_eq!(Prayer::Fajr.name(), "Fajr");
        assert_eq!(Prayer::Sunrise.name(), "Sunrise");

        if Utc::now().weekday() == Weekday::Fri {
            assert_eq!(Prayer::Dhuhr.name(), "Jumua");
        } else {
            assert_eq!(Prayer::Dhuhr.name(), "Dhuhr");
        }

        assert_eq!(Prayer::Asr.name(), "Asr");
        assert_eq!(Prayer::Maghrib.name(), "Maghrib");
        assert_eq!(Prayer::Isha.name(), "Isha");
        assert_eq!(Prayer::Qiyam.name(), "Qiyam");
    }
}
