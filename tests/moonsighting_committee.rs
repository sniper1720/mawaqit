use chrono::NaiveDate;
use mawaqit::prelude::*;

fn moonsighting_committee_parameters() -> Parameters {
    Configuration::with(Method::MoonsightingCommittee, Madhab::Shafi)
}

const MAIN_PRAYERS: [Prayer; 6] = [
    Prayer::Fajr,
    Prayer::Sunrise,
    Prayer::Dhuhr,
    Prayer::Asr,
    Prayer::Maghrib,
    Prayer::Isha,
];

/// Moonsighting Committee Zone C Summer (|lat| > 60°, astronomical
/// summer): the schedule slides down to
/// 60° and applies Sab'u Lail there. Hammerfest (70.66°N) on the summer
/// solstice has a 24h day — try_new() previously errored because sunrise
/// does not occur at the true latitude. It must now succeed and exactly
/// match the 60°N schedule for the same date.
#[test]
fn hammerfest_summer_anchors_to_60_degrees() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
    let hammerfest = Coordinates::new(70.66, 23.68);
    let at_60_north = Coordinates::new(60.0, 23.68);

    let hammerfest_times =
        PrayerTimes::try_new(date, hammerfest, moonsighting_committee_parameters())
            .expect("Zone C summer must succeed via the 60° anchor");
    let anchor_times =
        PrayerTimes::try_new(date, at_60_north, moonsighting_committee_parameters()).unwrap();

    for prayer in MAIN_PRAYERS {
        assert_eq!(
            hammerfest_times.time(prayer),
            anchor_times.time(prayer),
            "Hammerfest summer must match the 60°N schedule for {prayer:?}"
        );
    }

    assert!(
        hammerfest_times.time(Prayer::Fajr) < hammerfest_times.time(Prayer::Sunrise),
        "Fajr before Sunrise"
    );
    assert!(
        hammerfest_times.time(Prayer::Maghrib) < hammerfest_times.time(Prayer::Isha),
        "Maghrib before Isha"
    );
}

/// Moonsighting Committee Zone C, perpetual night outside summer
/// (|lat| > 60°): the FAQ's nearest-latitude rule (Aqrabul-Bilaad)
/// governs — the schedule walks
/// to the nearest working latitude on the same meridian and computes the
/// full seasonal schedule there (wholesale substitution; nothing anchors
/// to 60°). Hammerfest (70.66°N) on 1 January is in polar night —
/// previously an error, then a 60° anchor, now a nearest-latitude walk (Aqrabul-Bilaad)
/// substitution.
#[test]
fn hammerfest_perpetual_night_walks_to_the_nearest_working_latitude() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date");
    let hammerfest = Coordinates::new(70.66, 23.68);
    let at_60_north = Coordinates::new(60.0, 23.68);

    let hammerfest_times =
        PrayerTimes::try_new(date, hammerfest, moonsighting_committee_parameters())
            .expect("perpetual night must succeed via nearest-latitude substitution");

    let substitute_latitude = hammerfest_times.reference_latitude();
    assert!(
        60.0 < substitute_latitude && substitute_latitude < 70.66,
        "substitute must lie strictly between 60° and the true latitude, got {substitute_latitude}"
    );

    // Wholesale substitution: the schedule must be exactly what a
    // resident of the substitute latitude computes (real days there ⇒
    // dynamic seasonal path).
    let substitute_times = PrayerTimes::try_new(
        date,
        Coordinates::new(substitute_latitude, 23.68),
        moonsighting_committee_parameters(),
    )
    .expect("substitute latitude has normal days");
    for prayer in MAIN_PRAYERS {
        assert_eq!(
            hammerfest_times.time(prayer),
            substitute_times.time(prayer),
            "perpetual-night schedule must equal the substitute-latitude schedule for {prayer:?}"
        );
    }

    let anchor_times =
        PrayerTimes::try_new(date, at_60_north, moonsighting_committee_parameters()).unwrap();
    assert_ne!(
        hammerfest_times.time(Prayer::Fajr),
        anchor_times.time(Prayer::Fajr),
        "perpetual night outside summer must not anchor to 60°"
    );

    assert!(
        hammerfest_times.time(Prayer::Fajr) < hammerfest_times.time(Prayer::Sunrise),
        "Fajr before Sunrise"
    );
    assert!(
        hammerfest_times.time(Prayer::Maghrib) < hammerfest_times.time(Prayer::Isha),
        "Maghrib before Isha"
    );
}

/// Moonsighting Committee Zone C Spring/Fall (|lat| > 60°, real days): the seasonal functions
/// are used directly at the reference latitude, not at the 60° anchor.
/// Tromsø (69.65°N) on the spring equinox has sunrise and sunset, so the
/// schedule must compute at the true latitude — Fajr/Sunrise must differ
/// from what the 60° anchor would produce.
#[test]
fn tromso_spring_equinox_uses_dynamic_seasonal_directly() {
    let date = NaiveDate::from_ymd_opt(2026, 3, 20).expect("valid date");
    let tromso = Coordinates::new(69.65, 18.96);
    let at_60_north = Coordinates::new(60.0, 18.96);

    let tromso_times = PrayerTimes::try_new(date, tromso, moonsighting_committee_parameters())
        .expect("dynamic Zone C must succeed");
    let anchor_times =
        PrayerTimes::try_new(date, at_60_north, moonsighting_committee_parameters()).unwrap();

    assert_ne!(
        tromso_times.time(Prayer::Fajr),
        anchor_times.time(Prayer::Fajr),
        "Dynamic Zone C must use the reference latitude, not the 60° anchor"
    );
    assert_ne!(
        tromso_times.time(Prayer::Sunrise),
        anchor_times.time(Prayer::Sunrise),
        "Sunrise must be computed at the true latitude"
    );

    assert!(
        tromso_times.time(Prayer::Fajr) < tromso_times.time(Prayer::Sunrise),
        "Fajr before Sunrise"
    );
    assert!(
        tromso_times.time(Prayer::Maghrib) < tromso_times.time(Prayer::Isha),
        "Maghrib before Isha"
    );
}

/// Moonsighting Committee Zone C, spring-gap perpetual day (|lat| > 60°):
/// around mid-May Hammerfest's sun already never climbs through the
/// −0.833° refraction horizon, but mid-May is SPRING — outside
/// astronomical summer, so the season-scoped 60° slide does not apply.
/// The FAQ's nearest-latitude rule (Aqrabul-Bilaad) governs: walk to the nearest working latitude
/// and compute the full seasonal schedule there.
#[test]
fn hammerfest_spring_gap_perpetual_day_walks_to_the_nearest_working_latitude() {
    let date = NaiveDate::from_ymd_opt(2026, 5, 15).expect("valid date");
    let hammerfest = Coordinates::new(70.66, 23.68);
    let at_60_north = Coordinates::new(60.0, 23.68);

    let hammerfest_times =
        PrayerTimes::try_new(date, hammerfest, moonsighting_committee_parameters())
            .expect("spring-gap perpetual day must succeed via nearest-latitude substitution");

    let substitute_latitude = hammerfest_times.reference_latitude();
    assert!(
        60.0 < substitute_latitude && substitute_latitude < 70.66,
        "substitute must lie strictly between 60° and the true latitude, got {substitute_latitude}"
    );

    let substitute_times = PrayerTimes::try_new(
        date,
        Coordinates::new(substitute_latitude, 23.68),
        moonsighting_committee_parameters(),
    )
    .expect("substitute latitude has normal days");
    for prayer in MAIN_PRAYERS {
        assert_eq!(
            hammerfest_times.time(prayer),
            substitute_times.time(prayer),
            "spring-gap schedule must equal the substitute-latitude schedule for {prayer:?}"
        );
    }

    let anchor_times =
        PrayerTimes::try_new(date, at_60_north, moonsighting_committee_parameters()).unwrap();
    assert_ne!(
        hammerfest_times.time(Prayer::Fajr),
        anchor_times.time(Prayer::Fajr),
        "spring gap is not summer: must not anchor to 60°"
    );
}

/// The post-solstice perpetual tail (Jul 21–25 at Hammerfest) lies INSIDE
/// astronomical summer (solstice → equinox), so the season-scoped slide
/// still governs there: anchor to 60° + Sab'u Lail, not a walk.
#[test]
fn hammerfest_july_perpetual_tail_still_inside_summer_anchors() {
    let date = NaiveDate::from_ymd_opt(2026, 7, 23).expect("valid date");
    let hammerfest = Coordinates::new(70.66, 23.68);
    let at_60_north = Coordinates::new(60.0, 23.68);

    let hammerfest_times =
        PrayerTimes::try_new(date, hammerfest, moonsighting_committee_parameters())
            .expect("July perpetual tail is inside astronomical summer");
    let anchor_times =
        PrayerTimes::try_new(date, at_60_north, moonsighting_committee_parameters()).unwrap();

    for prayer in MAIN_PRAYERS {
        assert_eq!(
            hammerfest_times.time(prayer),
            anchor_times.time(prayer),
            "July tail must match the 60°N schedule for {prayer:?}"
        );
    }
}

/// Moonsighting Committee Zone C seasonal window: early September is
/// still astronomical summer (declination shrinking toward the equinox
/// from the summer side), so the schedule anchors to 60°; by October the
/// magnitude grows again toward winter and the seasonal functions run at
/// the true latitude instead.
#[test]
fn hammerfest_summer_window_closes_after_the_equinox() {
    let summer_date = NaiveDate::from_ymd_opt(2026, 9, 10).expect("valid date");
    let autumn_date = NaiveDate::from_ymd_opt(2026, 10, 5).expect("valid date");
    let hammerfest = Coordinates::new(70.66, 23.68);
    let at_60_north = Coordinates::new(60.0, 23.68);

    let summer_times =
        PrayerTimes::try_new(summer_date, hammerfest, moonsighting_committee_parameters())
            .expect("September is astronomical summer");
    let summer_anchor = PrayerTimes::try_new(
        summer_date,
        at_60_north,
        moonsighting_committee_parameters(),
    )
    .unwrap();
    assert_eq!(
        summer_times.time(Prayer::Fajr),
        summer_anchor.time(Prayer::Fajr),
        "Early September must anchor to the 60°N schedule"
    );

    let autumn_times =
        PrayerTimes::try_new(autumn_date, hammerfest, moonsighting_committee_parameters())
            .expect("October has real days at the true latitude");
    let autumn_anchor = PrayerTimes::try_new(
        autumn_date,
        at_60_north,
        moonsighting_committee_parameters(),
    )
    .unwrap();
    assert_ne!(
        autumn_times.time(Prayer::Fajr),
        autumn_anchor.time(Prayer::Fajr),
        "Post-equinox October must compute at the true latitude, not the 60° anchor"
    );
}

/// Southern-hemisphere symmetry: Concordia Station (−75.1°S) sits in
/// polar night every July — outside astronomical summer (southern summer
/// is December→March), so the FAQ's nearest-latitude rule governs: walk toward
/// the equator on the same meridian and compute there wholesale.
#[test]
fn concordia_july_polar_night_walks_to_the_nearest_working_latitude() {
    let date = NaiveDate::from_ymd_opt(2026, 7, 15).expect("valid date");
    let concordia = Coordinates::new(-75.1, 123.35);

    let concordia_times =
        PrayerTimes::try_new(date, concordia, moonsighting_committee_parameters())
            .expect("southern polar night must succeed via nearest-latitude substitution");

    let substitute_latitude = concordia_times.reference_latitude();
    assert!(
        -70.66 < substitute_latitude && substitute_latitude < -60.0,
        "substitute must lie strictly between −60° and the true latitude, got {substitute_latitude}"
    );

    let substitute_times = PrayerTimes::try_new(
        date,
        Coordinates::new(substitute_latitude, 123.35),
        moonsighting_committee_parameters(),
    )
    .expect("substitute latitude has normal days");
    for prayer in MAIN_PRAYERS {
        assert_eq!(
            concordia_times.time(prayer),
            substitute_times.time(prayer),
            "southern substitution must be wholesale for {prayer:?}"
        );
    }
}

/// Moonsighting Committee Zone C Summer, southern hemisphere (|lat| > 60°,
/// astronomical summer): the schedule slides down to 60°S. Palmer Station (−64.77°S)
/// on the December solstice has a ~22h day — try_new() previously returned
/// garbage Fajr/Isha in the southern hemisphere (the `days_since_solstice`
/// southern branch underflowed / was off by one year). It must now succeed
/// via the −60° anchor and exactly match the 60°S schedule for the same
/// date. The Sab'u Lail (1/7th) rule is hemisphere-symmetric (the committee's
/// `|latitude| >= 55` gate), so the −60° anchor applies it exactly as the +60°
/// anchor does in the northern summer.
#[test]
fn palmer_station_summer_anchors_to_minus_60_degrees() {
    let date = NaiveDate::from_ymd_opt(2026, 12, 21).expect("valid date");
    let palmer_station = Coordinates::new(-64.77, -64.05);
    let at_60_south = Coordinates::new(-60.0, -64.05);

    let palmer_times =
        PrayerTimes::try_new(date, palmer_station, moonsighting_committee_parameters())
            .expect("southern Zone C summer must succeed via the −60° anchor");
    let anchor_times =
        PrayerTimes::try_new(date, at_60_south, moonsighting_committee_parameters()).unwrap();

    for prayer in MAIN_PRAYERS {
        assert_eq!(
            palmer_times.time(prayer),
            anchor_times.time(prayer),
            "Palmer Station summer must match the 60°S schedule for {prayer:?}"
        );
    }

    assert!(
        palmer_times.time(Prayer::Fajr) < palmer_times.time(Prayer::Sunrise),
        "Fajr before Sunrise"
    );
    assert!(
        palmer_times.time(Prayer::Maghrib) < palmer_times.time(Prayer::Isha),
        "Maghrib before Isha"
    );
}

/// Transition day: on the day polar night ends at Hammerfest the
/// sun peeks above the horizon for ~35 minutes, but an adjacent day still
/// has none — the three-day solar set cannot build at the true latitude.
/// The nearest-latitude walk governs here too, so the method never errors
/// and reports the substitute via `reference_latitude()`.
#[test]
fn hammerfest_january_transition_day_walks_instead_of_erroring() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 20).expect("valid date");
    let hammerfest = Coordinates::new(70.66, 23.68);

    let hammerfest_times =
        PrayerTimes::try_new(date, hammerfest, moonsighting_committee_parameters())
            .expect("transition days must succeed via the nearest-latitude walk, not error");

    let substitute_latitude = hammerfest_times.reference_latitude();
    assert!(
        60.0 < substitute_latitude && substitute_latitude < 70.66,
        "substitute must lie strictly between 60° and the true latitude, got {substitute_latitude}"
    );

    let substitute_times = PrayerTimes::try_new(
        date,
        Coordinates::new(substitute_latitude, 23.68),
        moonsighting_committee_parameters(),
    )
    .expect("substitute latitude has normal days");
    for prayer in MAIN_PRAYERS {
        assert_eq!(
            hammerfest_times.time(prayer),
            substitute_times.time(prayer),
            "transition-day substitution must be wholesale for {prayer:?}"
        );
    }
}

/// Totality proof: with anchor, substitution, transition-day walk, and
/// dynamic routing combined, Moonsighting Committee must produce a
/// schedule for every single day of the year at extreme latitudes in
/// both hemispheres.
#[test]
fn every_day_of_2026_succeeds_at_extreme_latitudes_in_both_hemispheres() {
    let hammerfest = Coordinates::new(70.66, 23.68);
    let concordia = Coordinates::new(-75.1, 123.35);

    let mut date = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date");
    while date.year() == 2026 {
        for (place, coordinates) in [("Hammerfest", hammerfest), ("Concordia", concordia)] {
            assert!(
                PrayerTimes::try_new(date, coordinates, moonsighting_committee_parameters())
                    .is_ok(),
                "{place} must resolve on {date}"
            );
        }
        date = date.succ_opt().expect("date advances");
    }
}

/// The committee's Fajr and Isha come from its own rules; MWL's
/// LocalRelativeEstimation is simply inert for this method — including
/// its 66.6° guard. Non-MC methods keep both.
#[test]
fn local_relative_estimation_is_inert_for_moonsighting_committee() {
    let date = NaiveDate::from_ymd_opt(2026, 7, 15).expect("valid date");
    let hammerfest = Coordinates::new(70.66, 23.68);

    let default_times =
        PrayerTimes::try_new(date, hammerfest, moonsighting_committee_parameters()).unwrap();
    let mut lre_parameters = moonsighting_committee_parameters();
    lre_parameters.high_latitude_rule = HighLatitudeRule::LocalRelativeEstimation;
    let lre_times = PrayerTimes::try_new(date, hammerfest, lre_parameters)
        .expect("LRE must not trip its own 66.6° guard under Moonsighting Committee");
    for prayer in MAIN_PRAYERS {
        assert_eq!(
            default_times.time(prayer),
            lre_times.time(prayer),
            "LRE must not alter any prayer time for {prayer:?}"
        );
    }

    // Control: the guard stays live for methods that do use LRE.
    let mut mwl_with_lre = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);
    mwl_with_lre.high_latitude_rule = HighLatitudeRule::LocalRelativeEstimation;
    assert!(
        PrayerTimes::try_new(date, Coordinates::new(67.0, 23.68), mwl_with_lre).is_err(),
        "MWL + LRE above 66.6° must still error"
    );
}

/// Moonsighting Committee routes every day above 60° through its own
/// Zone C rules, so `polar_estimation` is simply inert for this method:
/// same schedule whether it is set or not.
#[test]
fn polar_estimation_setting_does_not_touch_moonsighting_committee() {
    for (month, day) in [(1, 20), (3, 15), (7, 15), (12, 21)] {
        let date = NaiveDate::from_ymd_opt(2026, month, day).expect("valid date");
        let hammerfest = Coordinates::new(70.66, 23.68);

        let without_estimation =
            PrayerTimes::try_new(date, hammerfest, moonsighting_committee_parameters()).unwrap();
        let mut estimation_parameters = moonsighting_committee_parameters();
        estimation_parameters.polar_estimation = Some(PolarEstimation::NearestLatitude);
        let with_estimation =
            PrayerTimes::try_new(date, hammerfest, estimation_parameters).unwrap();
        for prayer in MAIN_PRAYERS {
            assert_eq!(
                without_estimation.time(prayer),
                with_estimation.time(prayer),
                "polar_estimation must not alter MC times on {date} for {prayer:?}"
            );
        }
    }
}
