use chrono::{Datelike, NaiveDate, Weekday};

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
    /// Return the English name of this prayer, independent of the date.
    ///
    /// Always returns `"Dhuhr"` for Dhuhr; use [`Prayer::name_on`] to get
    /// the `"Jumua"` label for Friday dates.
    /// Both `Fajr` and `FajrTomorrow` return `"Fajr"`.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Fajr | Self::FajrTomorrow => String::from("Fajr"),
            Self::Sunrise => String::from("Sunrise"),
            Self::Dhuhr => String::from("Dhuhr"),
            Self::Asr => String::from("Asr"),
            Self::Maghrib => String::from("Maghrib"),
            Self::Isha => String::from("Isha"),
            Self::Qiyam => String::from("Qiyam"),
        }
    }

    /// Return the English name of this prayer for the given date.
    ///
    /// `Dhuhr` returns `"Jumua"` when `date` is a Friday, and `"Dhuhr"`
    /// otherwise. All other prayers return the same value as [`Self::name`].
    #[must_use]
    pub fn name_on(&self, date: NaiveDate) -> String {
        match self {
            Self::Dhuhr if date.weekday() == Weekday::Fri => String::from("Jumua"),
            _ => self.name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prayer_names_are_date_independent_base_labels() {
        assert_eq!(Prayer::Fajr.name(), "Fajr");
        assert_eq!(Prayer::FajrTomorrow.name(), "Fajr");
        assert_eq!(Prayer::Sunrise.name(), "Sunrise");
        assert_eq!(Prayer::Dhuhr.name(), "Dhuhr");
        assert_eq!(Prayer::Asr.name(), "Asr");
        assert_eq!(Prayer::Maghrib.name(), "Maghrib");
        assert_eq!(Prayer::Isha.name(), "Isha");
        assert_eq!(Prayer::Qiyam.name(), "Qiyam");
    }

    #[test]
    fn dhuhr_is_jumua_on_friday() {
        let friday = NaiveDate::from_ymd_opt(2026, 6, 19).expect("valid date");
        assert_eq!(Prayer::Dhuhr.name_on(friday), "Jumua");
    }

    #[test]
    fn dhuhr_is_plain_on_non_friday() {
        let sunday = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
        assert_eq!(Prayer::Dhuhr.name_on(sunday), "Dhuhr");
        assert_eq!(Prayer::Fajr.name_on(sunday), "Fajr");
    }
}
