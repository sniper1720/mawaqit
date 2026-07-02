use std::io::{Write, stdout};
use std::{thread, time::Duration};

use chrono::Local;
use mawaqit::prelude::*;

fn main() {
    let coords = Coordinates::new(50.85, 4.35);
    let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);

    loop {
        let today = Local::now().date_naive();
        let prayers = PrayerSchedule::new()
            .on(today)
            .for_location(coords)
            .with_configuration(params)
            .calculate()
            .expect("prayer times calculation failed");

        let current = prayers.current();
        let next = prayers.next();
        let (hours, minutes) = prayers.time_remaining();
        let next_local = prayers.time(next).with_timezone(&Local);
        let current_time = prayers.current_prayer_time().with_timezone(&Local);

        print!(
            "\rCurrent: {current:<10?} (since {time:<8})  \
             Next: {next:<12?} at {next_time:<8}  \
             Remaining: {hours}h {minutes:02}m   ",
            time = current_time.format("%H:%M"),
            next_time = next_local.format("%H:%M"),
        );
        stdout().flush().expect("flush failed");

        thread::sleep(Duration::from_secs(1));
    }
}
