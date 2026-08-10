# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] — 2026-08-09

### Breaking

- Removed `PrayerTimes::time_remaining()`. Use `PrayerTimes::time_remaining_at(Utc::now())`
  instead; it returns an exact `chrono::Duration`, never emits the invalid `(1, 60)` pair,
  and goes negative once the next prayer has started.
- Renamed `PolarFallback` to `PolarEstimation`; `Parameters::polar_fallback` is now
  `polar_estimation: Option<PolarEstimation>`.
- `Prayer::name()` is now pure and returns `Dhuhr` on Fridays. Use `Prayer::name_on(date)`
  for the `Jumua` label.
- `Qiblah` now `Display`s 4 decimals (`123.4783`); `Qiblah::value()` remains exact.
- Renamed `compute_pct` to `compute_isha_night_ratio`; it now requires a `year`
  argument, so results no longer depend on the current date.
- Removed `Stride::next_date(bool)`; use the explicit `tomorrow()` or `yesterday()` instead.

### Added

- `PrayerTimes::current_at(now)`, `PrayerTimes::next_at(now)`, and
  `PrayerTimes::time_remaining_at(now)` — these take the instant as an argument instead of
  reading the system clock; `current()` and `next()` remain unchanged.
- `PrayerTimes::resolved_latitude()` — the effective latitude after polar fallback.

### Fixed

- Countdown rounding: a remaining time of 1h59m59s no longer reports `(1, 60)`.
- Countdown no longer flips a minute at prayer boundaries (single clock read instead of two).
- LocalRelativeEstimation times no longer drift with the run year.

## [0.2.4] — 2026-07-26

### Changed
- `LocalRelativeEstimation` is now a proper variant: set it directly, percentage computed automatically.

## [0.2.3] — 2026-07-20

### Fixed
- high_latitude_rule: reject LocalRelativeEstimation above 66.6° instead of panicking.

## [0.2.2] — 2026-07-18

### Fixed
- corrected_hour_angle: add >179° guard to prevent Newton divergence when sin(HA) → 0 at the polar boundary.
- resolve_latitude: check adjacent days before accepting original latitude to prevent panic on polar reappearance/disappearance day.

### Changed
- schedule: deduplicate night_yesterday computation.

## [0.2.1] — 2026-07-13

### Added
- Shafaq (twilight variant) is now auto-selected from the chosen Madhab in `Configuration::with()`.

## [0.2.0] — 2026-07-11

### Added
- `HighLatitudeRule::Recommended` variant — defers rule selection to `try_new()` so `recommended()` evaluates against the fallback-resolved latitude.
- `PolarFallback::resolve_latitude()` now accepts `Madhab` — Asr guard uses the actual shadow length instead of hardcoded Shafi.

### Fixed
- `setting_hour` minute-rounding no longer produces invalid 24:00; wraps cleanly to next day.

## [0.1.0] — 2026-07-02

- Initial release
