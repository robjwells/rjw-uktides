## Unreleased

- Dual-license under Apache-2.0, add license files

## 0.6.0 - 2025-08-19

- **lib**: Switch from `chrono` to `jiff` for datetime types
- **lib**: Remove date field from `TidalEvent`
- **lib**: All times are now in the Europe/London timezone (not UTC)
- **lib**: Use enum for station country
- **lib**: Remove CLI-specific error cases
- **cli**: Made optional along with its dependencies (disable default features).
- **cli**: Add `details` command to view a single tide station's details.
- **cli**: Add `--json` argument to `list` command to get the returned JSON.
- **both**: Remove listing of "cached" stations (embedded at build time).
