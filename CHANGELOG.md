# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions CI for Linux x86_64 builds (`.deb`, AppImage)
- Release workflow that attaches bundles and changelog to GitHub Releases
- MIT License

### Fixed
- WireGuard connection state sync between Home and Config pages
- Status polling via elevated `wgctl` for installed packages
- Idempotent route handling (`ip route replace`) and cleanup of stale tunnels

## [0.1.0] - 2026-07-15

### Added
- Initial release: workspace-based WireGuard manager for Linux
- Native Rust config generator for WiFi/Ethernet split configs
- System tray quick connect/disconnect
- Optional one-time passwordless elevation for `wgctl.sh`

[Unreleased]: https://github.com/AliSawari/aether/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/AliSawari/aether/releases/tag/v0.1.0
