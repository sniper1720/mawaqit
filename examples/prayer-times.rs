use chrono::Local;
use mawaqit::prelude::*;

fn main() {
    let prayers = PrayerSchedule::new()
        .on(NaiveDate::from_ymd_opt(2026, 6, 21).expect("invalid date"))
        .for_location(Coordinates::new(50.85, 4.35)) // Brussels
        .with_configuration(Configuration::with(
            Method::MuslimWorldLeague,
            Madhab::Shafi,
        ))
        .calculate()
        .expect("prayer times calculation failed");

    println!("Prayer times for Brussels on 2026-06-21:");
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
