/// Setting for the Asr prayer time.
/// For Hanafi madhab, the Asr is a bit later
/// than that of the Shafi madhab.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Madhab {
    Shafi = 1,
    Hanafi = 2,
}

impl Madhab {
    #[must_use]
    pub const fn shadow(&self) -> i32 {
        *self as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shafi_shadow() {
        let shafi = Madhab::Shafi;

        assert_eq!(shafi.shadow(), 1);
    }

    #[test]
    fn hanafi_shadow() {
        let hanafi = Madhab::Hanafi;

        assert_eq!(hanafi.shadow(), 2);
    }
}
