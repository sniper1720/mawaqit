use chrono::Local;
use mawaqit::prelude::*;

fn main() {
    let oslo = Coordinates::new(59.91, 10.75);
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).expect("invalid date");
    let base_params = Configuration::with(Method::MuslimWorldLeague, Madhab::Hanafi);

    let rules = [
        ("Middle of the Night", HighLatitudeRule::MiddleOfTheNight),
        ("Seventh of the Night", HighLatitudeRule::SeventhOfTheNight),
        ("Twilight Angle", HighLatitudeRule::TwilightAngle),
    ];

    println!("Oslo (59.9°N) on 2026-01-01 — Fajr/Isha by HighLatitudeRule:");
    for (label, rule) in &rules {
        let mut params = base_params;
        params.high_latitude_rule = *rule;

        let prayers = PrayerSchedule::new()
            .on(date)
            .for_location(oslo)
            .with_configuration(params)
            .calculate()
            .expect("calculation failed");

        let fajr = prayers.time(Prayer::Fajr).with_timezone(&Local);
        let isha = prayers.time(Prayer::Isha).with_timezone(&Local);

        println!(
            "  {label:>25}  Fajr {fajr}  Isha {isha}",
            fajr = fajr.format("%H:%M"),
            isha = isha.format("%H:%M"),
        );
    }
}
