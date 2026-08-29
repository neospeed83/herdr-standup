# Herdr Standup

Herdr Standup answers **“What did I work on yesterday?”** from evidence instead of memory. It turns Git commits from one or more repositories into a concise Markdown standup and keeps the underlying commit/file evidence collapsible beneath the summary.

Open the report from any Herdr workspace with `prefix+u`. With Herdr's default prefix, press `Ctrl+B`, release it, then press `U`.

## What it does

- Uses your local timezone to select yesterday.
- Reads commits from every branch and worktree in configured repositories.
- Produces `Yesterday`, `Today`, and `Blockers` sections.
- Saves dated Markdown reports in Herdr's plugin state directory.
- Runs locally with no network requests, telemetry, or AI API key.
- Installs its `prefix+u` shortcut automatically without overwriting conflicts.

The first release intentionally reports only verifiable committed work. Uncommitted investigation, meetings, and future priorities can be supplied as notes; richer Herdr session evidence is planned next.

## Runtime

Herdr Standup is a native Rust plugin. Installation downloads the matching prebuilt binary; Node.js and Rust are not required.

## Requirements

- Herdr 0.8.0 or newer
- Node.js 20 or newer
- Git

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

## Run without Herdr

```bash
npm run standup
npm test
```

Without Herdr, reports are saved under `.standup/` in the current repository.

## Privacy

Herdr Standup reads local Git metadata and writes a local Markdown file. It does not send repository data anywhere.

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
