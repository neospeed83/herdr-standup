use chrono::{Days, Local, NaiveDate, TimeZone};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    env, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

const SHORTCUT: &str = "prefix+u";
const BLOCK: &str = "[[keys.command]]\nkey = \"prefix+u\"\ntype = \"plugin_action\"\ncommand = \"herdr-standup.open\"\ndescription = \"open Herdr Standup\"";

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
    hash: String,
    timestamp: String,
    subject: String,
    files: Vec<String>,
}
#[derive(Debug)]
struct Activity {
    repo: PathBuf,
    commits: Vec<Commit>,
}

fn parse_records(text: &str) -> Vec<Commit> {
    text.split('\x1e')
        .filter_map(|r| {
            let mut lines = r.trim().lines();
            let h = lines.next()?;
            let mut p = h.split('\x1f');
            Some(Commit {
                hash: p.next()?.into(),
                timestamp: p.next()?.into(),
                subject: p.next()?.into(),
                files: lines.filter(|x| !x.is_empty()).map(Into::into).collect(),
            })
        })
        .collect()
}
fn context_cwd() -> PathBuf {
    env::var("HERDR_PLUGIN_CONTEXT_JSON")
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| {
            if let Some(path) = v.pointer("/worktree/path").and_then(Value::as_str) {
                return Some(PathBuf::from(path));
            }
            ["worktree_path", "workspace_cwd", "focused_pane_cwd"]
                .into_iter()
                .find_map(|k| v.get(k).and_then(Value::as_str).map(PathBuf::from))
        })
        .unwrap_or_else(|| env::current_dir().unwrap())
}
fn config(cwd: &Path) -> Config {
    let mut c = env::var_os("HERDR_PLUGIN_CONFIG_DIR")
        .map(PathBuf::from)
        .and_then(|p| fs::read_to_string(p.join("config.json")).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Config {
            repositories: vec![],
            today: vec![],
            blockers: vec![],
        });
    c.repositories.insert(0, cwd.into());
    let mut seen = BTreeSet::new();
    c.repositories.retain(|p| seen.insert(p.clone()));
    c
}
fn activity(repo: &Path, start: &str, end: &str) -> Option<Activity> {
    let top = Command::new("git")
        .args(["-C", repo.to_str()?, "rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !top.status.success() {
        return None;
    }
    let root = PathBuf::from(String::from_utf8_lossy(&top.stdout).trim());
    let out = Command::new("git")
        .args([
            "-C",
            repo.to_str()?,
            "log",
            "--all",
            "--no-merges",
            &format!("--since={start}"),
            &format!("--until={end}"),
            "--pretty=format:%x1e%H%x1f%cI%x1f%s%x1f%an",
            "--name-only",
        ])
        .output()
        .ok()?;
    Some(Activity {
        repo: root,
        commits: parse_records(&String::from_utf8_lossy(&out.stdout)),
    })
}
fn render(date: &str, activities: &[Activity], today: &[String], blockers: &[String]) -> String {
    let mut s = format!("# Standup — {date}\n\n## Yesterday\n");
    let active: Vec<_> = activities
        .iter()
        .filter(|a| !a.commits.is_empty())
        .collect();
    if active.is_empty() {
        s.push_str("- No committed Git activity was found. Add notes below for research, meetings, or uncommitted work.\n")
    } else {
        for a in &active {
            let name = a.repo.file_name().unwrap_or_default().to_string_lossy();
            let mut seen = BTreeSet::new();
            for c in &a.commits {
                if seen.insert(&c.subject) {
                    s.push_str(&format!("- **{name}:** {}\n", c.subject));
                }
            }
        }
    }
    s.push_str("\n## Today\n");
    if today.is_empty() {
        s.push_str("- _Add today's priorities._\n")
    } else {
        for x in today {
            s.push_str(&format!("- {x}\n"))
        }
    }
    s.push_str("\n## Blockers\n");
    if blockers.is_empty() {
        s.push_str("- None recorded.\n")
    } else {
        for x in blockers {
            s.push_str(&format!("- {x}\n"))
        }
    }
    s.push_str("\n<details>\n<summary>Evidence</summary>\n\n");
    if active.is_empty() {
        s.push_str("No commit evidence was found for this period.\n")
    } else {
        for a in active {
            s.push_str(&format!("### {}\n\n", a.repo.display()));
            for c in &a.commits {
                s.push_str(&format!(
                    "- `{}` {} — {}\n",
                    &c.hash[..c.hash.len().min(8)],
                    c.subject,
                    c.timestamp
                ));
                if !c.files.is_empty() {
                    s.push_str(&format!(
                        "  - Files: {}\n",
                        c.files
                            .iter()
                            .take(8)
                            .map(|f| format!("`{f}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
            s.push('\n')
        }
    }
    s.push_str("</details>\n");
    s
}
fn config_path() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(env::var("APPDATA").expect("APPDATA unavailable")).join("herdr/config.toml")
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env::var("HOME").expect("HOME unavailable")).join(".config")
            })
            .join("herdr/config.toml")
    }
}
fn binding(remove: bool) -> io::Result<()> {
    let p = config_path();
    let old = fs::read_to_string(&p).unwrap_or_default();
    let next = if remove {
        old.replace(&format!("\n\n{BLOCK}\n"), "\n")
            .replace(&format!("{BLOCK}\n"), "")
    } else if old.contains("herdr-standup.open") {
        old.clone()
    } else if old.contains(&format!("key = \"{SHORTCUT}\"")) {
        return Err(io::Error::other(format!("{SHORTCUT} is already assigned")));
    } else {
        format!(
            "{}{}{BLOCK}\n",
            old.trim_end(),
            if old.trim().is_empty() { "" } else { "\n\n" }
        )
    };
    if next != old {
        if let Some(d) = p.parent() {
            fs::create_dir_all(d)?
        }
        let backup = p.with_file_name("config.toml.bak-herdr-standup");
        if p.exists() && !backup.exists() {
            fs::copy(&p, backup)?;
        }
        fs::write(&p, next)?
    }
    println!(
        "{} {SHORTCUT}.",
        if remove { "Removed" } else { "Installed" }
    );
    Ok(())
}
fn generate(show: bool) -> io::Result<()> {
    let now = Local::now();
    let day: NaiveDate = now.date_naive().checked_sub_days(Days::new(1)).unwrap();
    let start = Local
        .from_local_datetime(&day.and_hms_opt(0, 0, 0).unwrap())
        .earliest()
        .unwrap();
    let end = Local
        .from_local_datetime(
            &day.checked_add_days(Days::new(1))
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .latest()
        .unwrap();
    let c = config(&context_cwd());
    let acts = c
        .repositories
        .iter()
        .filter_map(|r| activity(r, &start.to_rfc3339(), &end.to_rfc3339()))
        .collect::<Vec<_>>();
    let md = render(
        &day.format("%A, %B %-d, %Y").to_string(),
        &acts,
        &c.today,
        &c.blockers,
    );
    let dir = env::var_os("HERDR_PLUGIN_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".standup"));
    fs::create_dir_all(&dir)?;
    let out = dir.join(format!("{}.md", day.format("%Y-%m-%d")));
    fs::write(&out, &md)?;
    print!("{md}\nSaved: {}\n", out.display());
    if show {
        println!("\nPress Enter to close…");
        let _ = io::stdin().read(&mut [0]);
    }
    Ok(())
}
fn main() -> ExitCode {
    let a: Vec<_> = env::args().collect();
    let result = match a.get(1).map(String::as_str).unwrap_or("generate") {
        "install" => binding(false),
        "remove-keybinding" => binding(true),
        "open" => Command::new(env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".into()))
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
            .map(|_| ()),
        "show" => generate(true),
        _ => generate(false),
    };
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn records() {
        assert_eq!(
            parse_records("\x1eabc\x1fnow\x1fFix\x1fA\na.rs\n")[0].subject,
            "Fix"
        )
    }
    #[test]
    fn sections() {
        let s = render("Today", &[], &[], &[]);
        assert!(s.contains("## Yesterday") && s.contains("## Today") && s.contains("## Blockers"));
    }
}
