# Changelog

All notable changes to Herdr Standup are documented here.

## [0.2.0] - 2026-08-26

### Added

- Automatic, idempotent installation of the `prefix+u` keybinding.
- A one-time backup before modifying Herdr's user configuration.
- Conflict detection that refuses to overwrite an existing shortcut.
- A `remove-keybinding` action for clean uninstall.

## [0.1.2] - 2026-08-26

### Fixed

- Documented the required `config.toml` keybinding for Herdr 0.8.2 instead of claiming that plugin manifest key declarations become active automatically.

## [0.1.1] - 2026-08-26

### Fixed

- Changed the shortcut to `prefix+u`; `prefix+s` is Herdr's built-in Settings binding.

## [0.1.0] - 2026-08-26

### Added

- Evidence-backed yesterday summaries from Git commits across repositories.
- `Yesterday`, `Today`, and `Blockers` Markdown sections.
- Collapsible commit and changed-file evidence.
- Herdr popup pane and `prefix+s` shortcut.
- Local-only configuration and dated report persistence.
- Cross-platform Node.js implementation with no runtime dependencies.
