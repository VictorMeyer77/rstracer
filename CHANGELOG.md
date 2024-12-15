<!-- markdownlint-configure-file { "MD013": { "line_length": 100 }, "MD024": false } -->
# Changelog

All notable changes to this project will be documented in this file.

Log types:

- `added` for new features.
- `changed` for changes in existing functionality.
- `fixed` for any bug fixes.
- `removed` for features removed in this release.
- `security` to invite users to upgrade in case of vulnerabilities.

## Unreleased

### Added

- `netstat` crate

### Changed

- Rehab `logs` crate with journalctl
- Simplify `ps` by focusing linux
- Simplify `lsof` by removing network

### Fixed

### Removed

### Security

## [0.1.0](https://github.com/VictorMeyer77/rstracer/releases/tag/0.1.0) - 2024/11/29

### Added

- Export parquet or csv
- User dim gold table
- Logger subscriber configuration
- Add gold layer for network interface

### Changed

- Gold table schema
- Move rstracer.toml in the root
- Separate lsof / and -i
- Evaluate created_at as millis

### Fixed

### Removed

### Security

## [0.1.0-alpha](https://github.com/VictorMeyer77/rstracer/releases/tag/0.1.0-alpha) - 2024/10/31

### Added

- Initial release
