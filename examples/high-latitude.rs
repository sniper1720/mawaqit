use chrono::{DateTime, NaiveDate, Utc};
use mawaqit::prelude::*;

fn times_at(lat: f64, label: &str, date: NaiveDate, original_coords: Coordinates) {
    let params = Configuration::new(18.0, 17.0)
        .method(Method::MuslimWorldLeague)
        .madhab(Madhab::Shafi)
        .done();

    let ref_coords = Coordinates::new(lat, original_coords.longitude);
    match PrayerTimes::try_new(date, ref_coords, params) {
        Ok(t) => println!(
            "{label:>12} ({lat:.2}°): {:?} {:?} {:?} {:?} {:?} {:?}",
            t.time(Prayer::Fajr),
            t.time(Prayer::Sunrise),
            t.time(Prayer::Dhuhr),
            t.time(Prayer::Asr),
            t.time(Prayer::Maghrib),
            t.time(Prayer::Isha),
        ),
        Err(e) => println!("{label:>12} ({lat:.2}°): FAIL ({e})"),
    }
}

fn main() {
    let date = NaiveDate::from_ymd_opt(2026, 7, 5).expect("valid date");
    let dt: DateTime<Utc> = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let tromso = Coordinates::new(70.0, 20.0);

    // 1. Original 70°N — no fallback
    times_at(70.0, "Original 70°", date, tromso);

    // 2. NearestLatitude (resolved)
    let resolved = PolarFallback::NearestLatitude
        .resolve_latitude(dt, tromso, Madhab::Shafi)
        .unwrap();
    times_at(resolved, "NearestLat", date, tromso);
    println!("  → Resolved latitude: {resolved:.6}°");

    // 3. Reference45 (fixed 45°N)
    times_at(45.0, "Reference45", date, tromso);

    // 4. Also show at a clean mid-latitude
    times_at(48.0, "48° fixed", date, tromso);

    // Now show NearestLatitude via the proper API
    let params = Configuration::new(18.0, 17.0)
        .method(Method::MuslimWorldLeague)
        .madhab(Madhab::Shafi)
        .polar_fallback(PolarFallback::NearestLatitude)
        .done();
    let t = PrayerTimes::try_new(date, tromso, params).unwrap();
    println!("\nNearestLatitude via API:");
    println!(
        "  {:?} {:?} {:?} {:?} {:?} {:?}",
        t.time(Prayer::Fajr),
        t.time(Prayer::Sunrise),
        t.time(Prayer::Dhuhr),
        t.time(Prayer::Asr),
        t.time(Prayer::Maghrib),
        t.time(Prayer::Isha),
    );

    // Also for Jan 15 (winter)
    let winter = NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid date");
    let wt: DateTime<Utc> = winter.and_hms_opt(0, 0, 0).unwrap().and_utc();
    println!("\n--- Winter (Jan 15) ---");
    times_at(70.0, "Original 70°", winter, tromso);

    let winter_resolved = PolarFallback::NearestLatitude
        .resolve_latitude(wt, tromso, Madhab::Shafi)
        .unwrap();
    times_at(winter_resolved, "NearestLat", winter, tromso);
    println!("  → Resolved latitude: {winter_resolved:.6}°");
    times_at(45.0, "Reference45", winter, tromso);

    let t2 = PrayerTimes::try_new(winter, tromso, params).unwrap();
    println!("\nWinter NearestLatitude via API:");
    println!(
        "  {:?} {:?} {:?} {:?} {:?} {:?}",
        t2.time(Prayer::Fajr),
        t2.time(Prayer::Sunrise),
        t2.time(Prayer::Dhuhr),
        t2.time(Prayer::Asr),
        t2.time(Prayer::Maghrib),
        t2.time(Prayer::Isha),
    );
}
