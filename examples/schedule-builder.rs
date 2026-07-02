use chrono::Local;
use mawaqit::prelude::*;

fn print_prayers(prayers: &PrayerTimes) {
    for prayer in &[
        Prayer::Fajr,
        Prayer::Sunrise,
        Prayer::Dhuhr,
        Prayer::Asr,
        Prayer::Maghrib,
        Prayer::Isha,
    ] {
        let local = prayers.time(*prayer).with_timezone(&Local);
        println!("  {prayer:>8?}  {}", local.format("%H:%M"));
    }
}

fn main() {
    // Use the builder to configure and compute prayer times.
    let result = PrayerSchedule::new()
        .on(NaiveDate::from_ymd_opt(2026, 6, 21).expect("invalid date"))
        .for_location(Coordinates::new(50.85, 4.35)) // Brussels
        .with_configuration(Configuration::with(
            Method::MuslimWorldLeague,
            Madhab::Shafi,
        ))
        .calculate();

    match result {
        Ok(prayers) => {
            println!("Brussels — 2026-06-21 (MWL, Shafi):");
            print_prayers(&prayers);
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }

    // The builder returns an error if required fields are missing.
    let incomplete = PrayerSchedule::new()
        .on(NaiveDate::from_ymd_opt(2026, 6, 21).expect("invalid date"))
        .calculate();

    match incomplete {
        Ok(_) => println!("unexpected success"),
        Err(e) => println!("\nExpected error: {e}"),
    }
}
