# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2025-05-09

### Added

- Recipe-based build system replacing the sbuild packager
- Pipeline-based build architecture
- Version check for tool compatibility
- Image download retry logic
- License and documentation updates

### Changed

- Simplified CLI parsing and entry point
- Removed sbuild packager in favor of new pipeline
- Moved Debian-specific configs into deb package
- Reorganized crate structure with workspace versioning
- Replaced eyre with thiserror for error handling
- Rewrote debcrafter integration

### Fixed

- Submodule checkout for recursive submodules
- Autopkgtest version handling and image creation
- Piuparts and autopkgtest execution
- Java Gradle version resolution
- Normalize versions to always use semver

## [0.2.11] - 2025-02-17

### Fixed

- Dotnet backup packages no longer require unnecessary dependencies
- Made deps optional for dotnet packages

## [0.2.10] - 2025-02-17

### Fixed

- Problem with using ubuntu-latest and piuparts missing
- Allow older packages to be installed

## [0.2.9] - 2025-01-07

### Fixed

- npm update message no longer shows in output
- Autopkgtest reliability improvements
- Failing Java download URL
- Updated backported dotnet packages

## [0.2.8] - 2024-09-09

### Added

- Multiple debcrafter version support

### Changed

- Updated debcrafter to latest version

## [0.2.7] - 2024-07-22

### Added

- Initial Python runtime support

## [0.2.6] - 2024-07-09

### Fixed

- Removed curl dependency

## [0.2.5] - 2024-06-06

### Added

- Autopkgtest support for Ubuntu 24.04 (Noble)

## [0.2.4] - 2024-05-29

### Fixed

- Dotnet builds and backup hash/URL handling
- Noble dependencies for dotnet packages
- Microsoft dropped Ubuntu 24.04 support, added workaround

## [0.2.3] - 2024-05-14

### Fixed

- Autopkgtest on Ubuntu
- Updated Noble package example hashes

## [0.2.2] - 2024-05-10

### Added

- Ubuntu Noble (24.04) support
- Ubuntu Jammy (22.04) support
- Git source package support
- Reproducible build verification
- Lintian, piuparts, and autopkgtest integration
- Test beds for all supported languages
- Dotnet build support under sbuild
- Java Gradle example

### Changed

- Improved CLI usage arguments
- Improved error reporting
- Fail build if lintian, piuparts, or autopkgtest fails

### Fixed

- Reproducible builds for Go, Nim, Rust, JavaScript, TypeScript, and virtual packages
- Lintian errors across distributions

## [0.1.0] - 2024-04-16

### Added

- Initial release
- Debian package building with sbuild
- Support for C, Go, Java, JavaScript, Nim, Rust, TypeScript, and virtual packages
- Declarative TOML-based package configuration
- Bookworm (Debian 12) support
- Integration tests and CI pipeline
