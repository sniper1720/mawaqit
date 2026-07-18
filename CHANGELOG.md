# Changelog

All notable changes to this project will be documented in this file.

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
