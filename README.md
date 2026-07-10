# Mawaqit — مواقيت

[![CI](https://img.shields.io/github/actions/workflow/status/sniper1720/mawaqit/ci.yml?style=flat-square&logo=github)](https://github.com/sniper1720/mawaqit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/mawaqit.svg?style=flat-square&logo=rust)](https://crates.io/crates/mawaqit)
[![Docs.rs](https://img.shields.io/docsrs/mawaqit?style=flat-square&logo=docsdotrs)](https://docs.rs/mawaqit/)

Islamic prayer times for Rust with high-latitude & polar-region support.

## Quick start

Add `mawaqit` under `[dependencies]` in your `Cargo.toml`:

```toml
[dependencies]
mawaqit = "0.2"
```

Then set your location, date, and calculation method to get prayer times:

```rust
use mawaqit::prelude::*;

// your location (e.g., Brussels — a high-latitude example)
let brussels = Coordinates::new(50.85, 4.35);

// target date
let date = NaiveDate::from_ymd_opt(2026, 6, 21).unwrap();

// calculation parameters
let mut params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);
params.high_latitude_rule = HighLatitudeRule::Recommended;

// compute prayer times
let prayers = PrayerSchedule::new()
    .on(date)
    .for_location(brussels)
    .with_configuration(params)
    .calculate()
    .unwrap();
```

## Initialization parameters

### Coordinates

Create a `Coordinates` struct with the latitude and longitude for the location.

```rust
let coordinates = Coordinates::new(31.7683, 35.2137);
```

### Date

Use `NaiveDate` to avoid timezone confusion. Only the year, month, and day are used  (All other components will be ignored).

```rust
let date = NaiveDate::from_ymd_opt(2026, 6, 21).unwrap();
```

### Configuration

The `Configuration` builder constructs a `Parameters` struct. Use `Configuration::with(method, madhab)` to start from a preset, or `Configuration::new(fajr_angle, isha_angle)` for fully custom settings.

```rust
let params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);
```

#### Parameters

| Field               | Type               | Description                                                              |
| ------------------- | ------------------ | ------------------------------------------------------------------------ |
| `method`            | `Method`           | Preset used to populate default angles and adjustments.                  |
| `fajr_angle`        | `f64`              | Sun angle below horizon for Fajr (degrees).                              |
| `maghrib_angle`     | `f64`              | Sun angle below horizon for Maghrib (used by some methods).              |
| `isha_angle`        | `f64`              | Sun angle below horizon for Isha (degrees).                              |
| `isha_interval`     | `i32`              | Minutes after Maghrib to set Isha (overrides angle when > 0).            |
| `madhab`            | `Madhab`           | Asr calculation (Shafi or Hanafi).                                       |
| `high_latitude_rule`| `HighLatitudeRule` | Fallback rule for high latitudes. Default: `MiddleOfTheNight`.           |
| `polar_fallback`    | `PolarFallback`    | Fallback rule for polar (>66.5°). Default: `None`.                  |
| `adjustments`       | `TimeAdjustment`   | Custom per-prayer offsets in minutes (user adjustments).                 |
| `method_adjustments`| `TimeAdjustment`   | Built-in per-prayer offsets from the method preset.                      |
| `rounding`          | `Rounding`         | Rounding behavior: `Nearest`, `Up`, or `None`.                           |
| `shafaq`            | `Shafaq`           | Twilight type for MoonsightingCommittee method.                          |

### Method

Provides preset configuration for calculating prayer times.

| Value                    | Description                                                                                                                      |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------- |
| `MuslimWorldLeague`      | Muslim World League. Fajr 18°, Isha 17°.                                                                                        |
| `Egyptian`               | Egyptian General Authority of Survey. Fajr 19.5°, Isha 17.5°.                                                                   |
| `Karachi`                | University of Islamic Sciences, Karachi. Fajr 18°, Isha 18°.                                                                     |
| `UmmAlQura`              | Umm al-Qura University, Makkah. Fajr 18.5°, Isha fixed 90 min after Maghrib. Add +30 min during Ramadan.                        |
| `Dubai`                  | Used in UAE. Fajr 18.2°, Isha 18.2°, with 3 min offsets for sunrise, Dhuhr, Asr, Maghrib.                                       |
| `Qatar`                  | Same Isha interval as UmmAlQura, Fajr 18°.                                                                                      |
| `Kuwait`                 | Fajr 18°, Isha 17.5°.                                                                                                           |
| `MoonsightingCommittee`  | Khalid Shaukat's method. Uses seasonal adjustments and 1/7 rule above 55° latitude. Recommended for North America and UK.        |
| `Singapore`              | Used in Singapore, Malaysia, Indonesia. Fajr 20°, Isha 18°.                                                                      |
| `Turkey`                 | Diyanet approximation. Less accurate outside Turkey.                                                                             |
| `Tehran`                 | Institute of Geophysics, University of Tehran. Fajr 17.7°, Isha 14°, Maghrib at 4.5°.                                            |
| `NorthAmerica`           | ISNA method. Fajr 15°, Isha 15°. MoonsightingCommittee is preferred for NA.                                                      |
| `France`                 | Union des Organisations Islamiques de France (UOIF). Fajr 12°, Isha 12°.                                                   |
| `Algeria`                | Algerian Ministry of Religious Affairs. Fajr 18°, Isha 17°, Maghrib +3 min.                                                       |
| `Other`                  | Defaults to 0° angles. Use for fully custom parameters.                                                                          |

### Madhab

Setting for Asr prayer time.

| Value    | Description                                                   |
| -------- | ------------------------------------------------------------- |
| `Shafi`  | Earlier Asr (Shafi, Maliki, Hanbali, Jafari).                 |
| `Hanafi` | Later Asr.                                                    |

### HighLatitudeRule

Fallback approximations for Fajr and Isha when the sun does not reach the required angle.

| Value               | Description                                                                                                                          |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `MiddleOfTheNight`  | Fajr won't be earlier than the midpoint of the night; Isha won't be later. Prevents Fajr and Isha from crossing boundaries. Default. |
| `SeventhOfTheNight` | The night is divided into seven equal parts. Isha begins after the first seventh; Fajr at the beginning of the last seventh. |
| `TwilightAngle`     | The fajr/isha angle α determines a fraction t = α ÷ 60 of the night. Isha begins after the first t part; Fajr is calculated similarly. Example: 15° → t = 0.25 → Isha after the first quarter of the night. |
| `LocalRelativeEstimation` | Scans the year to compute the average Fajr/Isha proportion of the night from days where the angle is reachable. Applies that proportion as fallback with ±5 min/day smoothing at transitions. Adopted by MWL Fiqh Council, August 2009. Recommended for Zone 2 (48.6–66.5°). |

**Defer to `try_new()`** — set the variant on `params`:
```rust
params.high_latitude_rule = HighLatitudeRule::Recommended;
```

**Inspect directly** — call the static method:
```rust
let rule = HighLatitudeRule::recommended(Coordinates::new(50.85, 4.35));
```

### PolarFallback

Fallback for polar latitudes (>66.5°) where the sun may not rise or set for extended periods. All prayer times use the resolved (nearest/reference) latitude.

| Value              | Description                                                                                                                                |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `None`             | No polar adjustment. `try_new()` returns `Err` at polar latitudes — use for custom fallback or notification. Default.          |
| `NearestLatitude`  | Affected prayers calculated using the nearest latitude where astronomical signs are distinguishable. Based on the Board of Grand Scholars of Saudi Arabia and MWL 1986 Resolution 7. |
| `Reference45`      | Affected prayers calculated using a fixed reference latitude of 45°. Based on MWL 1986 Resolution 7, confirmed by the European Council for Fatwa and Research. |

### Shafaq

Used by the MoonsightingCommittee method to determine Isha twilight.

| Value     | Description                                                                                         |
| --------- | --------------------------------------------------------------------------------------------------- |
| `General` | Combination of Ahmer and Abyad. Default. Reasonable times for higher latitudes.                     |
| `Ahmer`   | Red twilight. Earlier Isha. Used by Shafi, Maliki, Hanbali.                                         |
| `Abyad`   | White twilight. Later Isha. Used by Hanafi.                                                         |

```rust
use mawaqit::Shafaq;

params.shafaq = Shafaq::Ahmer;
```

### Rounding

| Value     | Description              |
| --------- | ------------------------ |
| `Nearest` | Round to nearest minute. |
| `Up`      | Round up to next minute. |
| `None`    | No rounding.             |

```rust
use mawaqit::Rounding;

params.rounding = Rounding::Nearest;
```

### Adjustments

Add or subtract minutes per prayer using the `Adjustment` builder.

```rust
let adj = Adjustment::new()
    .fajr(-2)
    .sunrise(1)
    .dhuhr(0)
    .asr(0)
    .maghrib(2)
    .isha(4)
    .done();

params.adjustments = adj;
```

## Prayer schedule

### PrayerTimes

The `PrayerTimes` struct holds all computed prayer times as `DateTime<Utc>`. Convert to local time using chrono's timezone methods.

| Method            | Returns             | Description                                      |
| ----------------- | ------------------- | ------------------------------------------------ |
| `time(prayer)`    | `DateTime<Utc>`     | Time of a specific prayer.                       |
| `current()`       | `Prayer`            | The prayer currently in effect.                  |
| `next()`          | `Prayer`            | The upcoming prayer.                             |
| `time_remaining()`| `(u32, u32)`        | Hours and minutes until the next prayer.         |

```rust
let (hours, minutes) = schedule.time_remaining();
println!("Next: {} in {}h {}m", schedule.next().name(), hours, minutes);
```

### Prayer

| Variant         | Description              | `name()` returns                               |
|-----------------|--------------------------|------------------------------------------------|
| `Fajr`          | Dawn                     | `"Fajr"`                                       |
| `Sunrise`       | Sunrise                  | `"Sunrise"`                                    |
| `Dhuhr`         | Noon                     | `"Dhuhr"` (or `"Jumua"` on Friday)             |
| `Asr`           | Afternoon                | `"Asr"`                                        |
| `Maghrib`       | Sunset                   | `"Maghrib"`                                    |
| `Isha`          | Night                    | `"Isha"`                                       |
| `Qiyam`         | Last third of the night  | `"Qiyam"`                                      |
| `FajrTomorrow`  | Next day's Fajr          | `"Fajr"`                                       |

## Qibla direction

Get the direction to the Kaaba from any location, in degrees from true north.

```rust
let brussels  = Coordinates::new(50.85, 4.35);
let qiblah    = Qiblah::new(brussels);

println!("{}", qiblah);           // Display: 123.4783
println!("{}", qiblah.value());   // Raw f64: 123.4783420
```

## Full example

```rust
use mawaqit::prelude::*;

let brussels = Coordinates::new(50.85, 4.35);
let date    = NaiveDate::from_ymd_opt(2026, 6, 21).unwrap();
let mut params = Configuration::with(Method::MuslimWorldLeague, Madhab::Shafi);
params.high_latitude_rule = HighLatitudeRule::SeventhOfTheNight;
let result  = PrayerSchedule::new()
    .on(date)
    .for_location(brussels)
    .with_configuration(params)
    .calculate();

match result {
    Ok(prayer) => {
        println!("{}: {}", Prayer::Fajr.name(),    prayer.time(Prayer::Fajr).format("%-l:%M %p"));
        println!("{}: {}", Prayer::Sunrise.name(),  prayer.time(Prayer::Sunrise).format("%-l:%M %p"));
        println!("{}: {}", Prayer::Dhuhr.name(),    prayer.time(Prayer::Dhuhr).format("%-l:%M %p"));
        println!("{}: {}", Prayer::Asr.name(),      prayer.time(Prayer::Asr).format("%-l:%M %p"));
        println!("{}: {}", Prayer::Maghrib.name(),  prayer.time(Prayer::Maghrib).format("%-l:%M %p"));
        println!("{}: {}", Prayer::Isha.name(),     prayer.time(Prayer::Isha).format("%-l:%M %p"));
        println!("{}: {}", Prayer::Qiyam.name(),    prayer.time(Prayer::Qiyam).format("%-l:%M %p"));
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

Output (in UTC):

```
Fajr:     2:25 AM
Sunrise:  3:29 AM
Dhuhr:    11:45 AM
Asr:      4:06 PM
Maghrib:  8:00 PM
Isha:     9:04 PM
Qiyam:    12:17 AM
```

## Code of Conduct

This project is governed by the [Code of Conduct](code-of-conduct.md).

## Contribute & Support

- **Report Bugs** — Found an issue? [Open a ticket](https://github.com/sniper1720/mawaqit/issues).
- **Suggest Features** — Have an idea? Let me know!
- **Share** — Tell others about the project.

> Ibn Mas'ud (RAA) narrated that the Messenger of Allah (ﷺ) said:
> *"He who guides (others) to an act of goodness will have a reward similar to that of its doer."*
> — Related by Muslim

## Acknowledgement

This library was inspired by the [salah](https://crates.io/crates/salah) crate by Farhan Ahmed.
May Allah reward him for his work and admit him to Jannah.

Astronomical calculations use high-precision equations from
Jean Meeus' *Astronomical Algorithms*.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
