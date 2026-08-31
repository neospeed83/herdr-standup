# Herdr Standup

Herdr Standup turns your Git commits and a couple of optional notes into a focused three-question standup.

Open the report from any Herdr workspace with `prefix+u`. With Herdr's default prefix, press `Ctrl+B`, release it, then press `U`.

## What it does

- Uses your local timezone to select yesterday.
- Reads your commits (matching each repository's `user.email`) from every branch and worktree in configured repositories.
- Shows only: what you did yesterday, what you will do today, and any blockers.
- Saves dated Markdown reports in Herdr's plugin state directory.
- Runs locally with no network requests, telemetry, or AI API key.
- Installs its `prefix+u` shortcut automatically without overwriting conflicts.

Yesterday is based on committed work. Today and blockers come from your optional notes.

## Runtime

Herdr Standup is a native Rust plugin. Installation downloads the matching prebuilt binary; Node.js and Rust are not required.

## Requirements

- Herdr 0.8.0 or newer
- Git
- macOS/Linux installation: `curl`, `awk`, and either `sha256sum` or `shasum`

Prebuilt Linux binaries target GNU libc.

## Install

Install directly from GitHub:

```bash
herdr plugin install neospeed83/herdr-standup
herdr server reload-config
```

The installer adds the `prefix+u` binding to Herdr's `config.toml` and creates `config.toml.bak-herdr-standup` before its first change. If that shortcut is already assigned, installation stops instead of overwriting your configuration.

Then press `prefix+u` (`Ctrl+B`, release, then `U`).

You can also invoke it without a shortcut:

```bash
herdr plugin action invoke herdr-standup.generate
```

## Local development

```bash
cargo build --release
mkdir -p bin
cp target/release/herdr-standup bin/herdr-standup
herdr plugin link "$PWD"
herdr plugin action list --plugin herdr-standup
herdr plugin action invoke herdr-standup.generate
```

Open the popup report inside Herdr:

```bash
herdr plugin pane open --plugin herdr-standup --entrypoint standup
```

## Configuration

Find the configuration directory:

```bash
herdr plugin config-dir herdr-standup
```

Create `config.json` there:

```json
{
  "repositories": [
    "/Users/you/projects/api",
    "/Users/you/projects/web"
  ],
  "today": ["Finish the OAuth rollout"],
  "blockers": ["Need a decision on session expiry"]
}
```

The active workspace repository is always included automatically. Duplicate paths are ignored.

## Build and test

```bash
cargo run -- generate
cargo test
```

Without Herdr, reports are saved under `.standup/` in the current repository.

## Privacy

Herdr Standup reads local Git metadata and writes a local Markdown file. On Unix it creates report and replacement files with mode `0600`; Windows uses the current account's inherited ACL. It does not send repository data anywhere. Reports contain repository names and commit subjects, but omit hashes, timestamps, filenames, and absolute paths. Reports and plugin configuration are retained after uninstall unless you delete them explicitly.

The three-track plugin review and its remaining platform verification items are documented in [REVIEW.md](REVIEW.md).

## Updating

```bash
herdr plugin install neospeed83/herdr-standup
herdr server reload-config
```

The keybinding installer is idempotent, so updates preserve a single binding.

## Uninstall

```bash
herdr plugin action invoke herdr-standup.remove-keybinding
herdr plugin uninstall herdr-standup
herdr server reload-config
```

The explicit removal action ensures Herdr Standup deletes only the config block it owns.

## License

MIT — see [LICENSE](LICENSE).
