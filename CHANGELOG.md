# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] — 2026-07-11

### Added
- `HighLatitudeRule::Recommended` variant — defers rule selection to `try_new()` so `recommended()` evaluates against the fallback-resolved latitude.
- `PolarFallback::resolve_latitude()` now accepts `Madhab` — Asr guard uses the actual shadow length instead of hardcoded Shafi.

### Fixed
- `setting_hour` minute-rounding no longer produces invalid 24:00; wraps cleanly to next day.

## [0.1.0] — 2026-07-02

- Initial release
