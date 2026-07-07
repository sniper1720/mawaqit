use chrono::NaiveDate;
use mawaqit::prelude::*;

/// Scan resolved latitude and prayer times from 65–70° N/S to find
/// the practical floor where Asr→Maghrib gap is meaningful.
#[test]
fn scan_asr_gaps() {
    let dates = [
        ("winter", NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()),
        ("summer", NaiveDate::from_ymd_opt(2026, 6, 21).unwrap()),
    ];
    let params = Configuration::new(18.0, 17.0)
        .method(Method::MuslimWorldLeague)
        .madhab(Madhab::Shafi)
        .polar_fallback(PolarFallback::NearestLatitude)
        .done();

    println!(
        "\n{:>10} {:>7} {:>8} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>8}",
        "Season", "Input°", "Resolved°", "Fajr", "Sun", "Dhuhr", "Asr", "Magh", "Isha", "A→M"
    );
    println!("{}", "-".repeat(72));

    for &(season, date) in &dates {
        for &hemi in &["N", "S"] {
            for raw_lat in (65..=70).rev() {
                let lat = if hemi == "N" {
                    raw_lat as f64
                } else {
                    -(raw_lat as f64)
                };
                let coords = Coordinates::new(lat, 0.0);

                let result = PrayerTimes::try_new(date, coords, params);
                match result {
                    Ok(times) => {
                        let f = times.time(Prayer::Fajr).format("%H:%M");
                        let s = times.time(Prayer::Sunrise).format("%H:%M");
                        let d = times.time(Prayer::Dhuhr).format("%H:%M");
                        let a = times.time(Prayer::Asr).format("%H:%M");
                        let m = times.time(Prayer::Maghrib).format("%H:%M");
                        let i = times.time(Prayer::Isha).format("%H:%M");
                        let gap =
                            (times.time(Prayer::Maghrib) - times.time(Prayer::Asr)).num_minutes();
                        let resolved = PolarFallback::NearestLatitude
                            .resolve_latitude(date.and_hms_opt(0, 0, 0).unwrap().and_utc(), coords)
                            .unwrap_or(f64::NAN);
                        println!(
                            "{:>10} {:>7.0} {:>8.3} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5}min",
                            format!("{hemi} {season}"),
                            lat,
                            resolved,
                            f,
                            s,
                            d,
                            a,
                            m,
                            i,
                            gap
                        );
                    }
                    Err(_) => {
                        let resolved = PolarFallback::NearestLatitude
                            .resolve_latitude(date.and_hms_opt(0, 0, 0).unwrap().and_utc(), coords)
                            .unwrap_or(f64::NAN);
                        println!(
                            "{:>10} {:>7.0} {:>8.3}   FAILED",
                            format!("{hemi} {season}"),
                            lat,
                            resolved
                        );
                    }
                }
            }
        }
    }
    println!();
}

/// Scan resolved latitude stability across latitudes in both hemispheres
#[test]
fn scan_latitude_stability() {
    let dates = [
        ("winter", NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()),
        ("summer", NaiveDate::from_ymd_opt(2026, 6, 21).unwrap()),
    ];
    let lats_north = [69.0, 68.0, 67.0, 66.0, 65.0];
    let lats_south = [-69.0, -68.0, -67.0, -66.0, -65.0];

    for (season, date) in dates {
        for &lat in &lats_north {
            let coords = Coordinates::new(lat, 0.0);
            let prayer_date = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let resolved = PolarFallback::NearestLatitude.resolve_latitude(prayer_date, coords);
            let params = Configuration::new(18.0, 17.0)
                .method(Method::MuslimWorldLeague)
                .madhab(Madhab::Shafi)
                .polar_fallback(PolarFallback::NearestLatitude)
                .done();
            if let Ok(times) = PrayerTimes::try_new(date, coords, params) {
                let f = times.time(Prayer::Fajr).format("%H:%M");
                let s = times.time(Prayer::Sunrise).format("%H:%M");
                let d = times.time(Prayer::Dhuhr).format("%H:%M");
                let a = times.time(Prayer::Asr).format("%H:%M");
                let m = times.time(Prayer::Maghrib).format("%H:%M");
                let i = times.time(Prayer::Isha).format("%H:%M");
                let da = (times.time(Prayer::Asr) - times.time(Prayer::Dhuhr)).num_minutes();
                let am = (times.time(Prayer::Maghrib) - times.time(Prayer::Asr)).num_minutes();
                let ds = (times.time(Prayer::Sunrise) - times.time(Prayer::Dhuhr)).num_minutes();
                println!(
                    "N {season:6} {lat:5.0}° → {resolved:8.3?}  F:{f} S:{s} D:{d} A:{a} M:{m} I:{i}  D→A:{da:2}min A→M:{am:2}min S→D:{ds:2}min"
                );
            } else {
                println!("N {season:6} {lat:5.0}° → FAILED");
            }
        }
        for &lat in &lats_south {
            let coords = Coordinates::new(lat, 0.0);
            let prayer_date = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let resolved = PolarFallback::NearestLatitude.resolve_latitude(prayer_date, coords);
            let params = Configuration::new(18.0, 17.0)
                .method(Method::MuslimWorldLeague)
                .madhab(Madhab::Shafi)
                .polar_fallback(PolarFallback::NearestLatitude)
                .done();
            if let Ok(times) = PrayerTimes::try_new(date, coords, params) {
                let f = times.time(Prayer::Fajr).format("%H:%M");
                let s = times.time(Prayer::Sunrise).format("%H:%M");
                let d = times.time(Prayer::Dhuhr).format("%H:%M");
                let a = times.time(Prayer::Asr).format("%H:%M");
                let m = times.time(Prayer::Maghrib).format("%H:%M");
                let i = times.time(Prayer::Isha).format("%H:%M");
                let da = (times.time(Prayer::Asr) - times.time(Prayer::Dhuhr)).num_minutes();
                let am = (times.time(Prayer::Maghrib) - times.time(Prayer::Asr)).num_minutes();
                let ds = (times.time(Prayer::Sunrise) - times.time(Prayer::Dhuhr)).num_minutes();
                println!(
                    "S {season:6} {lat:5.0}° → {resolved:8.3?}  F:{f} S:{s} D:{d} A:{a} M:{m} I:{i}  D→A:{da:2}min A→M:{am:2}min S→D:{ds:2}min"
                );
            } else {
                println!("S {season:6} {lat:5.0}° → FAILED");
            }
        }
    }
}

/// Tromsø (70°N) on summer solstice: NearestLatitude should use a ref lat
/// ~48.5°N, Dhuhr/Asr from original 70°N, no panic.
#[test]
fn tromso_summer_solstice_nearest_latitude() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(70.0, 20.0);
    let params = Configuration::new(18.0, 17.0)
        .method(Method::MuslimWorldLeague)
        .madhab(Madhab::Shafi)
        .polar_fallback(PolarFallback::NearestLatitude)
        .done();

    // Should succeed — uses NearestLatitude to find a working latitude
    let result = PrayerTimes::try_new(date, coords, params);

    assert!(
        result.is_ok(),
        "NearestLatitude should work at polar: {:?}",
        result.err()
    );
    let times = result.expect("already checked");

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Fajr must be before Sunrise at polar with NearestLatitude"
    );
    assert!(
        times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
        "Sunrise must be before Dhuhr"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Dhuhr must be before Asr"
    );
    assert!(
        times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
        "Asr must be before Maghrib"
    );
    assert!(
        times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
        "Maghrib must be before Isha"
    );
}

/// Tromsø (70°N) on summer solstice with PolarFallback::None:
/// try_new() should return Err gracefully.
#[test]
fn tromso_summer_solstice_no_fallback_returns_error() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(70.0, 20.0);
    let mut params: Parameters = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);

    // Explicitly set None (already default, but to be clear)
    params.polar_fallback = PolarFallback::None;

    let result = PrayerTimes::try_new(date, coords, params);

    assert!(
        result.is_err(),
        "PolarFallback::None at polar should return Err"
    );
}

/// Tromsø (70°N) on spring equinox: try_new() succeeds because sunrise/sunset
/// exist even at this latitude. No fallback needed.
#[test]
fn tromso_equinox_no_fallback_needed() {
    let date = NaiveDate::from_ymd_opt(2026, 3, 20).expect("valid date");
    let coords = Coordinates::new(70.0, 20.0);
    let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);

    let result = PrayerTimes::try_new(date, coords, params);

    assert!(
        result.is_ok(),
        "try_new should succeed at 70°N on equinox: {:?}",
        result.err()
    );
}

/// Longyearbyen (78°N) with Reference45: uses 45°N, same longitude (15°E).
/// Times should be internally consistent.
#[test]
fn longyearbyen_reference_45() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(78.0, 15.0);
    let params = Configuration::new(18.0, 17.0)
        .method(Method::MuslimWorldLeague)
        .madhab(Madhab::Shafi)
        .polar_fallback(PolarFallback::Reference45)
        .done();

    let result = PrayerTimes::try_new(date, coords, params);

    assert!(
        result.is_ok(),
        "Reference45 should work at polar: {:?}",
        result.err()
    );
    let times = result.expect("already checked");

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Fajr before Sunrise"
    );
    assert!(
        times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
        "Sunrise before Dhuhr"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Dhuhr before Asr"
    );
    assert!(
        times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
        "Asr before Maghrib"
    );
    assert!(
        times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
        "Maghrib before Isha"
    );
}

/// Southern polar (70°S, 0°E) on December solstice with Reference45.
/// Should use 45°S, correct hemisphere.
#[test]
fn southern_polar_reference_45() {
    let date = NaiveDate::from_ymd_opt(2026, 12, 21).expect("valid date");
    let coords = Coordinates::new(-70.0, 0.0);
    let params = Configuration::new(18.0, 17.0)
        .method(Method::MuslimWorldLeague)
        .madhab(Madhab::Shafi)
        .polar_fallback(PolarFallback::Reference45)
        .done();

    let result = PrayerTimes::try_new(date, coords, params);

    assert!(
        result.is_ok(),
        "Reference45 should work at southern polar: {:?}",
        result.err()
    );
    let times = result.expect("already checked");

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Fajr before Sunrise at southern polar"
    );
    assert!(
        times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
        "Sunrise before Dhuhr"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Dhuhr before Asr"
    );
    assert!(
        times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
        "Asr before Maghrib"
    );
    assert!(
        times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
        "Maghrib before Isha"
    );
}

/// Non-polar control (Singapore 1.4°N, 103.8°E) with PolarFallback::None:
/// should behave identically to the old code — no polar issues.
#[test]
fn singapore_no_fallback() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(1.4, 103.8);
    let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);

    let result = PrayerTimes::try_new(date, coords, params);

    assert!(
        result.is_ok(),
        "Non-polar should work with PolarFallback::None: {:?}",
        result.err()
    );
    let times = result.expect("already checked");

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Fajr before Sunrise"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Dhuhr before Asr"
    );
}

/// builder-style PolarFallback can be set via Configuration::polar_fallback()
#[test]
fn builder_polar_fallback_nearest_latitude() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(70.0, 20.0);

    let params = Configuration::new(18.0, 17.0)
        .polar_fallback(PolarFallback::NearestLatitude)
        .done();

    let result = PrayerTimes::try_new(date, coords, params);
    assert!(result.is_ok(), "Builder with NearestLatitude should work");
}

/// PolarFallback::recommended returns NearestLatitude for >66.5°
#[test]
fn polar_fallback_recommended_above_66() {
    let coords = Coordinates::new(70.0, 20.0);
    assert_eq!(
        PolarFallback::recommended(coords),
        PolarFallback::NearestLatitude
    );
}

/// PolarFallback::recommended returns None for ≤66.5°
#[test]
fn polar_fallback_recommended_below_66() {
    let coords = Coordinates::new(48.6, 20.0);
    assert_eq!(PolarFallback::recommended(coords), PolarFallback::None);
}

/// PolarFallback::recommended returns None at exactly 66.5° (boundary, not >)
#[test]
fn polar_fallback_recommended_at_boundary() {
    let coords = Coordinates::new(66.5, 0.0);
    assert_eq!(PolarFallback::recommended(coords), PolarFallback::None);
}

/// Tromsø (70°N) in winter (polar night, no sunrise) with NearestLatitude.
#[test]
fn tromso_winter_nearest_latitude() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid date");
    let coords = Coordinates::new(70.0, 20.0);
    let params = Configuration::new(18.0, 17.0)
        .method(Method::MuslimWorldLeague)
        .madhab(Madhab::Shafi)
        .polar_fallback(PolarFallback::NearestLatitude)
        .done();

    let result = PrayerTimes::try_new(date, coords, params);
    assert!(
        result.is_ok(),
        "NearestLatitude should work at polar winter: {:?}",
        result.err()
    );
    let times = result.expect("already checked");

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Polar winter: Fajr before Sunrise"
    );
    assert!(
        times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
        "Polar winter: Sunrise before Dhuhr"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Polar winter: Dhuhr before Asr"
    );
    assert!(
        times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
        "Polar winter: Asr before Maghrib"
    );
    assert!(
        times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
        "Polar winter: Maghrib before Isha"
    );
}

/// Tromsø (70°N) in winter: PolarFallback::None should return Err.
#[test]
fn tromso_winter_no_fallback_returns_error() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid date");
    let coords = Coordinates::new(70.0, 20.0);
    let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);

    let result = PrayerTimes::try_new(date, coords, params);
    assert!(
        result.is_err(),
        "PolarFallback::None at polar winter should return Err"
    );
}

/// Debug: print actual times for southern polar winter
#[test]
fn debug_southern_times() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(-70.0, 0.0);
    let prayer_date = date.and_hms_opt(0, 0, 0).unwrap().and_utc();

    // Resolved latitude
    let resolved = PolarFallback::NearestLatitude.resolve_latitude(prayer_date, coords);
    println!("Original lat: -70.0, Resolved lat: {:?}", resolved);

    // Compute transit at original lat by running try_new and printing individual prayer times
    let params = Configuration::new(18.0, 17.0)
        .method(Method::MuslimWorldLeague)
        .madhab(Madhab::Shafi)
        .polar_fallback(PolarFallback::NearestLatitude)
        .done();

    let times = PrayerTimes::try_new(date, coords, params).expect("should work");
    println!("Fajr:     {}", times.time(Prayer::Fajr).format("%H:%M:%S"));
    println!(
        "Sunrise:  {}",
        times.time(Prayer::Sunrise).format("%H:%M:%S")
    );
    println!("Dhuhr:    {}", times.time(Prayer::Dhuhr).format("%H:%M:%S"));
    println!("Asr:      {}", times.time(Prayer::Asr).format("%H:%M:%S"));
    println!(
        "Maghrib:  {}",
        times.time(Prayer::Maghrib).format("%H:%M:%S")
    );
    println!("Isha:     {}", times.time(Prayer::Isha).format("%H:%M:%S"));
    println!("Date:     {}", times.time(Prayer::Fajr).format("%Y-%m-%d"));
}

/// Southern polar (70°S) in June: polar night, NearestLatitude.
#[test]
fn southern_polar_winter_nearest_latitude() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(-70.0, 0.0);
    let params = Configuration::new(18.0, 17.0)
        .method(Method::MuslimWorldLeague)
        .madhab(Madhab::Shafi)
        .polar_fallback(PolarFallback::NearestLatitude)
        .done();

    let result = PrayerTimes::try_new(date, coords, params);
    assert!(
        result.is_ok(),
        "Southern polar winter with NearestLatitude: {:?}",
        result.err()
    );
    let times = result.expect("already checked");

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Southern polar winter: Fajr before Sunrise"
    );
    assert!(
        times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
        "Southern polar winter: Sunrise before Dhuhr"
    );
    assert!(
        times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
        "Southern polar winter: Dhuhr before Asr"
    );
    assert!(
        times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
        "Southern polar winter: Asr before Maghrib"
    );
    assert!(
        times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
        "Southern polar winter: Maghrib before Isha"
    );
}

/// Polar circle (67°N) on summer solstice: try_new fails at actual lat,
/// succeeds with NearestLatitude.
#[test]
fn polar_circle_solstice_nearest_latitude() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(67.0, 0.0);
    let params = Configuration::new(18.0, 17.0)
        .polar_fallback(PolarFallback::NearestLatitude)
        .done();

    let result = PrayerTimes::try_new(date, coords, params);
    assert!(
        result.is_ok(),
        "NearestLatitude should work at 67°N: {:?}",
        result.err()
    );
}

/// Verify that Dhuhr times differ for two polar cities at different
/// longitudes with NearestLatitude (longitude should be preserved).
#[test]
fn different_longitudes_different_dhuhr() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords_east = Coordinates::new(70.0, 20.0);
    let coords_west = Coordinates::new(70.0, -20.0);

    let params_east = Configuration::new(18.0, 17.0)
        .polar_fallback(PolarFallback::NearestLatitude)
        .done();
    let params_west = Configuration::new(18.0, 17.0)
        .polar_fallback(PolarFallback::NearestLatitude)
        .done();

    let east_times = PrayerTimes::try_new(date, coords_east, params_east).expect("east ok");
    let west_times = PrayerTimes::try_new(date, coords_west, params_west).expect("west ok");

    // Dhuhr at different longitudes should differ (~4 min/degree)
    let dhuhr_diff = (east_times.time(Prayer::Dhuhr) - west_times.time(Prayer::Dhuhr))
        .num_minutes()
        .abs();
    assert!(
        dhuhr_diff > 0,
        "Dhuhr at 20°E and 20°W should differ, got {dhuhr_diff} min"
    );
}
