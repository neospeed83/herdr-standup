use chrono::{Days, Local, TimeZone};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Output},
};
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, value};

const SHORTCUT: &str = "prefix+u";
const COMMAND: &str = "herdr-standup.open";

#[derive(Debug, Clone, Deserialize)]
struct Config {
    #[serde(default)]
    repositories: Vec<PathBuf>,
    #[serde(default)]
    today: Vec<String>,
    #[serde(default)]
    blockers: Vec<String>,
}
#[derive(Debug, Clone, PartialEq)]
struct Commit {
    subject: String,
    author_email: String,
}
#[derive(Debug)]
struct Activity {
    repo: PathBuf,
    commits: Vec<Commit>,
}

fn parse_records(bytes: &[u8]) -> io::Result<Vec<Commit>> {
    let mut fields: Vec<_> = bytes.split(|byte| *byte == 0).collect();
    if fields.last().is_some_and(|field| field.is_empty()) {
        fields.pop();
    }
    let (pairs, remainder) = fields.as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Git returned an incomplete commit record",
        ));
    }
    let mut commits = Vec::with_capacity(fields.len() / 2);
    for pair in pairs {
        let subject = pair[0];
        let email = pair[1];
        let subject = subject.strip_prefix(b"\n").unwrap_or(subject);
        commits.push(Commit {
            subject: String::from_utf8_lossy(subject).into_owned(),
            author_email: String::from_utf8_lossy(email).into_owned(),
        });
    }
    Ok(commits)
}

fn context_cwd() -> io::Result<PathBuf> {
    if let Ok(raw) = env::var("HERDR_PLUGIN_CONTEXT_JSON") {
        let value: Value = serde_json::from_str(&raw).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid Herdr context: {error}"),
            )
        })?;
        if let Some(path) = value.pointer("/worktree/path").and_then(Value::as_str) {
            return Ok(path.into());
        }
        if let Some(path) = ["worktree_path", "workspace_cwd", "focused_pane_cwd"]
            .into_iter()
            .find_map(|key| value.get(key).and_then(Value::as_str))
        {
            return Ok(path.into());
        }
    }
    env::current_dir().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("current directory unavailable: {error}"),
        )
    })
}

fn config(cwd: &Path) -> io::Result<Config> {
    let mut value = match env::var_os("HERDR_PLUGIN_CONFIG_DIR") {
        Some(dir) => {
            let path = PathBuf::from(dir).join("config.json");
            match fs::read_to_string(&path) {
                Ok(text) => serde_json::from_str(&text).map_err(|error| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid {}: {error}", path.display()),
                    )
                })?,
                Err(error) if error.kind() == io::ErrorKind::NotFound => Config {
                    repositories: vec![],
                    today: vec![],
                    blockers: vec![],
                },
                Err(error) => {
                    return Err(io::Error::new(
                        error.kind(),
                        format!("unable to read {}: {error}", path.display()),
                    ));
                }
            }
        }
        None => Config {
            repositories: vec![],
            today: vec![],
            blockers: vec![],
        },
    };
    value.repositories.insert(0, cwd.into());
    let mut seen = BTreeSet::new();
    value.repositories.retain(|path| seen.insert(path.clone()));
    Ok(value)
}

fn git(repo: &Path, args: &[&str]) -> io::Result<Output> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("unable to run Git for {}: {error}", repo.display()),
            )
        })?;
    if output.status.success() {
        return Ok(output);
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(io::Error::other(format!(
        "Git failed for {}: {}",
        repo.display(),
        if stderr.is_empty() {
            "unknown error"
        } else {
            &stderr
        }
    )))
}

fn is_git_repo(path: &Path) -> io::Result<bool> {
    Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|output| output.status.success())
        .map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("unable to run Git for {}: {error}", path.display()),
            )
        })
}

fn activity(repo: &Path, start: &str, end: &str) -> io::Result<Activity> {
    let top = git(repo, &["rev-parse", "--show-toplevel"])?;
    let root = PathBuf::from(String::from_utf8_lossy(&top.stdout).trim());
    let identity = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["config", "--get", "user.email"])
        .output()
        .map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "unable to read Git identity for {}: {error}",
                    repo.display()
                ),
            )
        })?;
    if !identity.status.success() {
        return Err(io::Error::other(format!(
            "Git user.email is not configured for {}",
            repo.display()
        )));
    }
    let email = String::from_utf8_lossy(&identity.stdout).trim().to_owned();
    if email.is_empty() {
        return Err(io::Error::other(format!(
            "Git user.email is not configured for {}",
            repo.display()
        )));
    }
    let output = git(
        repo,
        &[
            "log",
            "--all",
            "--no-merges",
            &format!("--since={start}"),
            &format!("--until={end}"),
            "--pretty=format:%s%x00%ae%x00",
        ],
    )?;
    let commits = parse_records(&output.stdout)?
        .into_iter()
        .filter(|commit| commit.author_email.eq_ignore_ascii_case(&email))
        .collect();
    Ok(Activity {
        repo: root,
        commits,
    })
}

fn markdown_text(value: &str) -> String {
    let mut escaped = String::new();
    for c in value.chars().map(|c| {
        if c.is_control()
            || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        {
            ' '
        } else {
            c
        }
    }) {
        if matches!(
            c,
            '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '<' | '>' | '#' | '|' | '!'
        ) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}
fn render(activities: &[Activity], today: &[String], blockers: &[String]) -> String {
    let mut report = String::from("## What did you do yesterday?\n");
    let active: Vec<_> = activities
        .iter()
        .filter(|activity| !activity.commits.is_empty())
        .collect();
    if active.is_empty() {
        report.push_str("- Nothing recorded.\n");
    } else {
        for activity in &active {
            let name = markdown_text(
                &activity
                    .repo
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
            );
            let mut seen = BTreeSet::new();
            for commit in &activity.commits {
                if seen.insert(&commit.subject) {
                    report.push_str(&format!(
                        "- **{name}:** {}\n",
                        markdown_text(&commit.subject)
                    ));
                }
            }
        }
    }
    report.push_str("\n## What will you do today?\n");
    if today.is_empty() {
        report.push_str("- Nothing recorded.\n")
    } else {
        for item in today {
            report.push_str(&format!("- {}\n", markdown_text(item)))
        }
    }
    report.push_str("\n## Any blockers?\n");
    if blockers.is_empty() {
        report.push_str("- None recorded.\n")
    } else {
        for item in blockers {
            report.push_str(&format!("- {}\n", markdown_text(item)))
        }
    }
    report
}

fn config_path() -> io::Result<PathBuf> {
    if cfg!(windows) {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|path| path.join("herdr/config.toml"))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "APPDATA unavailable"))
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .map(|path| path.join("herdr/config.toml"))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "HOME and XDG_CONFIG_HOME unavailable",
                )
            })
    }
}

fn command_matches(table: &Table, command: Option<&str>) -> bool {
    table.get("key").and_then(Item::as_str) == Some(SHORTCUT)
        && command
            .is_none_or(|expected| table.get("command").and_then(Item::as_str) == Some(expected))
}

fn edit_binding(text: &str, remove: bool) -> io::Result<Option<String>> {
    let mut document = if text.trim().is_empty() {
        DocumentMut::new()
    } else {
        text.parse::<DocumentMut>().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid Herdr config.toml: {error}"),
            )
        })?
    };
    let commands = document
        .get("keys")
        .and_then(Item::as_table)
        .and_then(|keys| keys.get("command"))
        .and_then(Item::as_array_of_tables);
    let owned = commands.and_then(|tables| {
        tables.iter().position(|table| {
            command_matches(table, Some(COMMAND))
                && table.get("type").and_then(Item::as_str) == Some("plugin_action")
        })
    });
    if remove {
        let Some(index) = owned else { return Ok(None) };
        document["keys"]["command"]
            .as_array_of_tables_mut()
            .expect("validated array of tables")
            .remove(index);
        return Ok(Some(document.to_string()));
    }
    if owned.is_some() {
        return Ok(None);
    }
    if commands.is_some_and(|tables| tables.iter().any(|table| command_matches(table, None))) {
        return Err(io::Error::other(format!("{SHORTCUT} is already assigned")));
    }
    let keys = document
        .entry("keys")
        .or_insert_with(|| {
            let mut table = Table::new();
            table.set_implicit(true);
            Item::Table(table)
        })
        .as_table_mut()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Herdr config keys must be a table",
            )
        })?;
    let commands = keys
        .entry("command")
        .or_insert_with(|| Item::ArrayOfTables(ArrayOfTables::new()))
        .as_array_of_tables_mut()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Herdr keys.command must be an array of tables",
            )
        })?;
    let mut binding = Table::new();
    binding.insert("key", value(SHORTCUT));
    binding.insert("type", value("plugin_action"));
    binding.insert("command", value(COMMAND));
    binding.insert("description", value("open Herdr Standup"));
    commands.push(binding);
    Ok(Some(document.to_string()))
}

fn atomic_write(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("output path has no parent"))?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    ));
    let result = (|| {
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temp)?;
        file.write_all(contents)?;
        file.sync_all()?;
        #[cfg(windows)]
        {
            if path.exists() {
                use std::os::windows::ffi::OsStrExt;
                use windows_sys::Win32::Storage::FileSystem::ReplaceFileW;
                let target: Vec<_> = path.as_os_str().encode_wide().chain(Some(0)).collect();
                let replacement: Vec<_> = temp.as_os_str().encode_wide().chain(Some(0)).collect();
                if unsafe {
                    ReplaceFileW(
                        target.as_ptr(),
                        replacement.as_ptr(),
                        std::ptr::null(),
                        0,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                } == 0
                {
                    return Err(io::Error::last_os_error());
                }
                return Ok(());
            }
        }
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn binding(remove: bool) -> io::Result<()> {
    let path = config_path()?;
    let old = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };
    let Some(next) = edit_binding(&old, remove)? else {
        println!(
            "{}",
            if remove {
                "No Herdr Standup keybinding was installed."
            } else {
                "Herdr Standup keybinding is already installed."
            }
        );
        return Ok(());
    };
    if path.exists() {
        let backup = path.with_file_name("config.toml.bak-herdr-standup");
        if !backup.exists() {
            fs::copy(&path, backup)?;
        }
    }
    let write_path = if path
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        fs::canonicalize(&path)?
    } else {
        path.clone()
    };
    atomic_write(&write_path, next.as_bytes())?;
    println!(
        "{} {SHORTCUT}.",
        if remove { "Removed" } else { "Installed" }
    );
    Ok(())
}

fn generate(show: bool) -> io::Result<()> {
    let now = Local::now();
    let day = now
        .date_naive()
        .checked_sub_days(Days::new(1))
        .ok_or_else(|| io::Error::other("date underflow"))?;
    let start = Local
        .from_local_datetime(
            &day.and_hms_opt(0, 0, 0)
                .ok_or_else(|| io::Error::other("invalid start time"))?,
        )
        .earliest()
        .ok_or_else(|| io::Error::other("local start time does not exist"))?;
    let next_day = day
        .checked_add_days(Days::new(1))
        .ok_or_else(|| io::Error::other("date overflow"))?;
    let end = Local
        .from_local_datetime(
            &next_day
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| io::Error::other("invalid end time"))?,
        )
        .latest()
        .ok_or_else(|| io::Error::other("local end time does not exist"))?;
    let config = config(&context_cwd()?)?;
    let mut activities = vec![];
    let mut warnings = vec![];
    let mut roots = BTreeSet::new();
    for (index, repo) in config.repositories.iter().enumerate() {
        if index == 0 {
            match is_git_repo(repo) {
                Ok(false) => continue,
                Err(error) => {
                    warnings.push(error.to_string());
                    continue;
                }
                Ok(true) => {}
            }
        }
        match activity(repo, &start.to_rfc3339(), &end.to_rfc3339()) {
            Ok(activity) => {
                let root =
                    fs::canonicalize(&activity.repo).unwrap_or_else(|_| activity.repo.clone());
                if roots.insert(root) {
                    activities.push(activity);
                }
            }
            Err(error) => warnings.push(error.to_string()),
        }
    }
    if !warnings.is_empty() {
        for warning in &warnings {
            eprintln!("Collection error: {warning}");
        }
        return Err(io::Error::other(format!(
            "standup not saved because {} repository collection(s) failed",
            warnings.len()
        )));
    }
    let markdown = render(&activities, &config.today, &config.blockers);
    let dir = env::var_os("HERDR_PLUGIN_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| ".standup".into());
    let out = dir.join(format!("{}.md", day.format("%Y-%m-%d")));
    atomic_write(&out, markdown.as_bytes())?;
    print!("{markdown}");
    if !show {
        println!("\nSaved: {}", out.display());
    }
    if show {
        let _ = io::stdin().read(&mut [0]);
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<_> = env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        None | Some("generate") => generate(false),
        Some("install") => binding(false),
        Some("remove-keybinding") => binding(true),
        Some("open") => Command::new(env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".into()))
            .args([
                "plugin",
                "pane",
                "open",
                "--plugin",
                "herdr-standup",
                "--entrypoint",
                "standup",
                "--placement",
                "popup",
                "--focus",
            ])
            .status()
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    Err(io::Error::other("unable to open Standup popup"))
                }
            }),
        Some("show") => generate(true),
        Some(_) => {
            eprintln!("Usage: herdr-standup [generate|install|remove-keybinding|open|show]");
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unknown command",
            ))
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn commit(subject: &str) -> Commit {
        Commit {
            subject: subject.into(),
            author_email: "me@example.com".into(),
        }
    }
    #[test]
    fn records_include_author() {
        let records =
            parse_records(b"Fix\x1f safely\0me@example.com\0\nNext\0me@example.com\0").unwrap();
        assert_eq!(records[0].subject, "Fix\x1f safely");
        assert_eq!(records[0].author_email, "me@example.com");
        assert_eq!(records[1].subject, "Next");
    }
    #[test]
    fn report_contains_only_the_three_questions() {
        let report = render(&[], &[], &[]);
        assert_eq!(
            report,
            "## What did you do yesterday?\n- Nothing recorded.\n\n## What will you do today?\n- Nothing recorded.\n\n## Any blockers?\n- None recorded.\n"
        );
    }
    #[test]
    fn untrusted_markdown_is_escaped_and_paths_are_private() {
        let activity = Activity {
            repo: "/secret/repo".into(),
            commits: vec![commit("<img src=x> [click](url)\x1b")],
        };
        let report = render(&[activity], &["*priority*".into()], &[]);
        assert!(!report.contains("/secret/repo"));
        assert!(report.contains("\\<img src=x\\>"));
        assert!(report.contains("\\*priority\\*"));
        assert!(!report.contains('\x1b'));
    }
    #[test]
    fn keybinding_edits_preserve_unrelated_tables() {
        let text = "[[keys.command]]\nkey = \"prefix+u\"\ntype = \"plugin_action\"\ncommand = \"herdr-standup.open\"\n\n[server]\nname = \"keep me\"\n";
        let edited = edit_binding(text, true).unwrap().unwrap();
        assert!(!edited.contains("herdr-standup.open"));
        assert!(edited.contains("[server]") && edited.contains("keep me"));
        edited.parse::<DocumentMut>().unwrap();
    }
    #[test]
    fn inline_commented_keybinding_is_still_a_conflict() {
        let text = "[[keys.command]]\nkey = \"prefix+u\" # reserved\ntype = \"plugin_action\"\ncommand = \"other.open\"\n";
        assert!(
            edit_binding(text, false)
                .unwrap_err()
                .to_string()
                .contains("already assigned")
        );
    }
}
