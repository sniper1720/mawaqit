/// Time adjustment for all prayer times.
/// The value is specified in *minutes* and
/// can be either positive or negative.
#[derive(PartialEq, Debug, Copy, Clone, Default)]
pub struct TimeAdjustment {
    pub fajr: i64,
    pub sunrise: i64,
    pub dhuhr: i64,
    pub asr: i64,
    pub maghrib: i64,
    pub isha: i64,
}

impl TimeAdjustment {
    /// Create a [`TimeAdjustment`] with explicit minute values for each prayer.
    #[must_use]
    pub fn new(fajr: i64, sunrise: i64, dhuhr: i64, asr: i64, maghrib: i64, isha: i64) -> Self {
        TimeAdjustment {
            fajr,
            sunrise,
            dhuhr,
            asr,
            maghrib,
            isha,
        }
    }
}

/// Builder for [`TimeAdjustment`].
pub struct Adjustment {
    fajr: i64,
    sunrise: i64,
    dhuhr: i64,
    asr: i64,
    maghrib: i64,
    isha: i64,
}

impl Default for Adjustment {
    fn default() -> Self {
        Self::new()
    }
}

impl Adjustment {
    /// Create an [`Adjustment`] builder with all offsets initialized to zero.
    #[must_use]
    pub fn new() -> Self {
        Adjustment {
            fajr: 0,
            sunrise: 0,
            dhuhr: 0,
            asr: 0,
            maghrib: 0,
            isha: 0,
        }
    }

    /// Set the Fajr adjustment in minutes.
    pub fn fajr(&mut self, fajr: i64) -> &mut Self {
        self.fajr = fajr;
        self
    }

    /// Set the Sunrise adjustment in minutes.
    pub fn sunrise(&mut self, sunrise: i64) -> &mut Self {
        self.sunrise = sunrise;
        self
    }

    /// Set the Dhuhr adjustment in minutes.
    pub fn dhuhr(&mut self, dhuhr: i64) -> &mut Self {
        self.dhuhr = dhuhr;
        self
    }

    /// Set the Asr adjustment in minutes.
    pub fn asr(&mut self, asr: i64) -> &mut Self {
        self.asr = asr;
        self
    }

    /// Set the Maghrib adjustment in minutes.
    pub fn maghrib(&mut self, maghrib: i64) -> &mut Self {
        self.maghrib = maghrib;
        self
    }

    /// Set the Isha adjustment in minutes.
    pub fn isha(&mut self, isha: i64) -> &mut Self {
        self.isha = isha;
        self
    }

    /// Finalise the builder and return the [`TimeAdjustment`].
    #[must_use]
    pub fn done(&self) -> TimeAdjustment {
        TimeAdjustment {
            fajr: self.fajr,
            sunrise: self.sunrise,
            dhuhr: self.dhuhr,
            asr: self.asr,
            maghrib: self.maghrib,
            isha: self.isha,
        }
    }
}
