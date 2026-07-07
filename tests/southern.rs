use chrono::NaiveDate;
use mawaqit::prelude::*;

/// Buenos Aires (-34.6°N) on January (summer in southern hemisphere).
/// Verify prayer ordering is correct — Fajr < Sunrise < Dhuhr < Asr < Maghrib < Isha.
#[test]
fn buenos_aires_summer() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid date");
    let coords = Coordinates::new(-34.6, -58.4);
    let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);

    let times = PrayerTimes::try_new(date, coords, params).unwrap();

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Buenos Aires: Fajr before Sunrise"
    );
    assert!(
        times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
        "Buenos Aires: Sunrise before Dhuhr"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Buenos Aires: Dhuhr before Asr"
    );
    assert!(
        times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
        "Buenos Aires: Asr before Maghrib"
    );
    assert!(
        times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
        "Buenos Aires: Maghrib before Isha"
    );
}

/// Sydney (-33.9°N) on June (winter in southern hemisphere).
/// Verify prayer ordering is correct.
#[test]
fn sydney_winter() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(-33.9, 151.2);
    let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);

    let times = PrayerTimes::try_new(date, coords, params).unwrap();

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Sydney: Fajr before Sunrise"
    );
    assert!(
        times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
        "Sydney: Sunrise before Dhuhr"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Sydney: Dhuhr before Asr"
    );
    assert!(
        times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
        "Sydney: Asr before Maghrib"
    );
    assert!(
        times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
        "Sydney: Maghrib before Isha"
    );
}

/// Punta Arenas (-53.2°N) on summer solstice (December).
/// High southern latitude — equivalent to Zone 2 in the north.
/// This tests that the LRE-aware calculation works correctly for the
/// southern hemisphere with abs(latitude) handled properly.
#[test]
fn punta_arenas_summer_solstice() {
    let date = NaiveDate::from_ymd_opt(2026, 12, 21).expect("valid date");
    let coords = Coordinates::new(-53.2, -70.9);
    let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);

    let times = PrayerTimes::try_new(date, coords, params).unwrap();

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Punta Arenas: Fajr before Sunrise"
    );
    assert!(
        times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
        "Punta Arenas: Sunrise before Dhuhr"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Punta Arenas: Dhuhr before Asr"
    );
    assert!(
        times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
        "Punta Arenas: Asr before Maghrib"
    );
    assert!(
        times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
        "Punta Arenas: Maghrib before Isha"
    );
}

/// Punta Arenas (-53.2°N) on winter solstice (June).
/// Tests that the high-latitude rule (Zone 2, |lat| > 48.6) works
/// correctly in the southern hemisphere.
#[test]
fn punta_arenas_winter_solstice() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(-53.2, -70.9);
    let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);

    let times = PrayerTimes::try_new(date, coords, params).unwrap();

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Punta Arenas winter: Fajr before Sunrise"
    );
    assert!(
        times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
        "Punta Arenas winter: Sunrise before Dhuhr"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Punta Arenas winter: Dhuhr before Asr"
    );
    assert!(
        times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
        "Punta Arenas winter: Asr before Maghrib"
    );
    assert!(
        times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
        "Punta Arenas winter: Maghrib before Isha"
    );
}

/// Equator (0°, 0°) with standard angles.
/// Day and night are ~12h year-round, so prayer times should be stable.
#[test]
fn equator_equinox() {
    let date = NaiveDate::from_ymd_opt(2026, 3, 20).expect("valid date");
    let coords = Coordinates::new(0.0, 0.0);
    let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);

    let times = PrayerTimes::try_new(date, coords, params).unwrap();

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Equator: Fajr before Sunrise"
    );
    assert!(
        times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
        "Equator: Sunrise before Dhuhr"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Equator: Dhuhr before Asr"
    );
    assert!(
        times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
        "Equator: Asr before Maghrib"
    );
    assert!(
        times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
        "Equator: Maghrib before Isha"
    );
}

/// Verify that Sydney times are internally consistent on an equinox date.
#[test]
fn sydney_equinox() {
    let date = NaiveDate::from_ymd_opt(2026, 9, 23).expect("valid date");
    let coords = Coordinates::new(-33.9, 151.2);
    let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);

    let times = PrayerTimes::try_new(date, coords, params).unwrap();

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Sydney equinox: Fajr before Sunrise"
    );
    assert!(
        times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
        "Sydney equinox: Sunrise before Dhuhr"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Sydney equinox: Dhuhr before Asr"
    );
    assert!(
        times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
        "Sydney equinox: Asr before Maghrib"
    );
    assert!(
        times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
        "Sydney equinox: Maghrib before Isha"
    );
}
