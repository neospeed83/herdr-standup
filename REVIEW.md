# Herdr plugin tournament review

Reviewed 2026-08-30 using three independent tracks: security and privacy,
Herdr plugin lifecycle and UX, and portability, releases, and tests.
The final pass was run through the installed Herdr Tournament plugin with three
high-effort Codex reviewers, adversarial peer review, and a separate high-effort
Codex judge. Tournament cleaned up every isolated review workspace afterward.

## Accepted findings and resolutions

| Priority | Finding | Resolution |
| --- | --- | --- |
| Critical | CI still ran `npm test` after the Rust migration. | Replaced it with a three-OS Rust matrix running format, Clippy, tests, and version-consistency checks. |
| High | Installers fetched mutable `latest` binaries directly and did not verify integrity. | Pin downloads to the plugin version, publish per-asset SHA-256 files, verify before installation, and stage downloads before replacement. |
| High | Reports included every author's commits while claiming to show the user's work. | Resolve each repository's `user.email` and include only matching commits. |
| High | Git and configuration failures were silently presented as no activity. | Invalid configuration now fails clearly; repository-specific Git errors go to stderr and an incomplete report never replaces the prior report. |
| High | Git/config content could inject Markdown, HTML, or terminal controls. | Sanitize all rendered external text, escape Markdown, neutralize code-span backticks, and omit absolute repository paths. |
| Medium | Keybinding edits depended on one exact text layout and could consume unrelated tables. | Parse TOML structurally, remove only the exact owned table, preserve unrelated tables and comments, report no-ops accurately, and stage config/report writes before replacement. |
| Medium | Unknown CLI commands generated and overwrote a report. | Reject unknown commands with usage and a nonzero exit status. |
| Medium | Release jobs published independently without full validation or checksums. | Validate formatting, Clippy, tests, tag/version agreement, and all matrix builds before one aggregate release publishes binaries and checksums. |
| Product focus | The report included a title, date, diagnostics, and expandable commit/file evidence. | The UI now contains only the three standup questions and their answers. Operational diagnostics remain outside the report. |

## Tournament judge follow-up

The judge found the first patch was not release-ready. The follow-up fixes:

- bumped every embedded version to `0.4.0`, so installers no longer request
  nonexistent checksum assets from the already-published `v0.3.3` release;
- made release publication depend on formatting, Clippy, tests, tag/version
  agreement, and completion of every platform build before one aggregate upload;
- replaced delimiter-sensitive commit parsing with exact NUL-delimited pairs;
- replaced manual keybinding slicing with structural TOML editing;
- skip an automatically detected non-Git workspace, deduplicate resolved Git
  roots, and preserve the previous report if any configured repository fails;
- use Windows `ReplaceFileW` for failure-safe replacement and preserve Unix
  symlinked Herdr configuration targets;
- strengthened the report regression test to assert the complete output exactly.

## Open verification items

- Installed actions and the popup still need a native Windows smoke test before
  release. The Windows target compile-check passes locally.
- Git identity matching uses the repository's configured `user.email`. Users who
  commit with several addresses need an explicit aliases configuration in a
  future schema.
- GitHub Actions remain pinned to major release tags. Pinning audited commit SHAs
  and adding artifact attestations would further harden the release pipeline.

## Positive controls retained

The runtime remains local-only, subprocess arguments do not pass through a shell,
dependencies are locked, keybinding conflicts are not overwritten, configuration
is backed up before its first edit, commit records are NUL-delimited, and release
builds cover five native targets.
