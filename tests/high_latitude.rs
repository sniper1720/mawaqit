use chrono::{DateTime, NaiveDate, Utc};
use mawaqit::prelude::*;

/// Oslo (59.9°N, 10.7°E) on summer solstice with LRE (Zone 2).
/// This should work identically to before — polar_fallback is not involved
/// since try_new() succeeds at the original latitude.
#[test]
fn oslo_lre_unchanged() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let coords = Coordinates::new(59.9094, 10.7349);
    let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);
    let lre_rule = HighLatitudeRule::mwl_2009(coords, &params);

    let lre_params = Configuration::new(18.0, 17.0)
        .high_latitude_rule(lre_rule)
        .done();

    let times = PrayerTimes::try_new(date, coords, lre_params).unwrap();

    assert!(
        times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
        "Fajr must be before Sunrise (Oslo LRE)"
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

/// LRE day-to-day smoothing: Fajr time must never jump > 10 min between
/// consecutive days during the LRE transition (Brussels, June).
#[test]
fn lre_ramp_smooth_transition() {
    let coords = Coordinates::new(50.85, 4.35);
    let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);
    let lre_rule = HighLatitudeRule::mwl_2009(coords, &params);
    let lre_params = Configuration::new(18.0, 17.0)
        .high_latitude_rule(lre_rule)
        .done();

    let start = NaiveDate::from_ymd_opt(2026, 6, 10).expect("valid date");
    let mut prev_fajr: Option<DateTime<Utc>> = None;

    let clock_dist = |a: DateTime<Utc>, b: DateTime<Utc>| -> f64 {
        let raw = a.signed_duration_since(b).num_seconds() as f64 / 60.0;
        let wrapped = if raw >= 720.0 {
            raw - 1440.0
        } else if raw <= -720.0 {
            raw + 1440.0
        } else {
            raw
        };
        wrapped.abs()
    };

    for day_offset in 0..14 {
        let date = start + Duration::days(day_offset);
        let times = PrayerTimes::try_new(date, coords, lre_params).unwrap();

        // Ordering check on every day
        assert!(
            times.time(Prayer::Fajr) < times.time(Prayer::Sunrise),
            "Day {}: Fajr before Sunrise",
            day_offset
        );
        assert!(
            times.time(Prayer::Sunrise) < times.time(Prayer::Dhuhr),
            "Day {}: Sunrise before Dhuhr",
            day_offset
        );
        assert!(
            times.time(Prayer::Dhuhr) < times.time(Prayer::Asr),
            "Day {}: Dhuhr before Asr",
            day_offset
        );
        assert!(
            times.time(Prayer::Asr) < times.time(Prayer::Maghrib),
            "Day {}: Asr before Maghrib",
            day_offset
        );
        assert!(
            times.time(Prayer::Maghrib) < times.time(Prayer::Isha),
            "Day {}: Maghrib before Isha",
            day_offset
        );

        // Day-to-day Fajr change must be ≤ 10 min (LRE entry/exit ramp ±5 + snap ≤5)
        if let Some(prev) = prev_fajr {
            let diff = clock_dist(times.time(Prayer::Fajr), prev);
            assert!(
                diff <= 10.0,
                "Day {}→{}: Fajr jump {:.1} min exceeds LRE smoothing limit (max 10)",
                day_offset - 1,
                day_offset,
                diff
            );
        }
        prev_fajr = Some(times.time(Prayer::Fajr));
    }
}
