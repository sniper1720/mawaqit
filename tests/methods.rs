use chrono::NaiveDate;
use mawaqit::prelude::*;

/// Paris (48.85°N) on summer solstice with Method::France (12°/12°).
/// 12° angles are shallow enough to work year-round at this latitude.
#[test]
fn france_integration() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(48.85, 2.35);
    let params = Configuration::with(Method::France, Madhab::Shafi);

    let times = PrayerTimes::try_new(date, coords, params).unwrap();

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Paris France: Fajr before Sunrise"
    );
    assert!(
        times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
        "Paris France: Sunrise before Dhuhr"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Paris France: Dhuhr before Asr"
    );
    assert!(
        times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
        "Paris France: Asr before Maghrib"
    );
    assert!(
        times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
        "Paris France: Maghrib before Isha"
    );
}

/// Algiers (36.75°N) on summer solstice with Method::Algeria (18°/17° +3 min Maghrib).
/// Only difference from France at same location is the +3 min Maghrib adjustment.
#[test]
fn algeria_integration() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(36.75, 3.04);

    let params_algeria = Configuration::with(Method::Algeria, Madhab::Shafi);
    let params_france = Configuration::with(Method::France, Madhab::Shafi);

    let algeria_times = PrayerTimes::try_new(date, coords, params_algeria).unwrap();
    let france_times = PrayerTimes::try_new(date, coords, params_france).unwrap();

    // Verify the +3 min Maghrib adjustment
    let maghrib_diff =
        (algeria_times.time(Prayer::Maghrib) - france_times.time(Prayer::Maghrib)).num_minutes();
    assert_eq!(
        maghrib_diff, 3,
        "Algeria Maghrib should be exactly 3 min later than France at same location"
    );

    // Ordering
    assert!(
        algeria_times.time(Prayer::Fajr) < algeria_times.time(Prayer::Sunrise),
        "Algiers: Fajr before Sunrise"
    );
    assert!(
        algeria_times.time(Prayer::Sunrise) < algeria_times.time(Prayer::Dhuhr),
        "Algiers: Sunrise before Dhuhr"
    );
    assert!(
        algeria_times.time(Prayer::Dhuhr) < algeria_times.time(Prayer::Asr),
        "Algiers: Dhuhr before Asr"
    );
    assert!(
        algeria_times.time(Prayer::Asr) < algeria_times.time(Prayer::Maghrib),
        "Algiers: Asr before Maghrib"
    );
    assert!(
        algeria_times.time(Prayer::Maghrib) < algeria_times.time(Prayer::Isha),
        "Algiers: Maghrib before Isha"
    );
}
