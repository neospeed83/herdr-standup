import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

export function dayWindow(now = new Date(), offset = -1) {
  const start = new Date(now);
  start.setHours(0, 0, 0, 0);
  start.setDate(start.getDate() + offset);
  const end = new Date(start);
  end.setDate(end.getDate() + 1);
  return { start, end };
}

export function parseGitRecords(text) {
  if (!text.trim()) return [];
  return text
    .split("\x1e")
    .map((record) => record.trim())
    .filter(Boolean)
    .map((record) => {
      const [header, ...files] = record.split("\n").filter(Boolean);
      const [hash, timestamp, subject, author] = header.split("\x1f");
      return { hash, timestamp, subject, author, files };
    });
}

export function collectGitActivity(repo, window, runner = execFileSync) {
  const format = "%x1e%H%x1f%cI%x1f%s%x1f%an";
  const args = [
    "-C", repo, "log", "--all", "--no-merges",
    `--since=${window.start.toISOString()}`,
    `--until=${window.end.toISOString()}`,
    `--pretty=format:${format}`, "--name-only"
  ];
  try {
    const top = runner("git", ["-C", repo, "rev-parse", "--show-toplevel"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"]
    }).trim();
    const output = runner("git", args, {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"]
    });
    return { repo: top, commits: parseGitRecords(output) };
  } catch {
    return null;
  }
}

export function summarizeRepo(activity) {
  const commits = activity.commits;
  const files = [...new Set(commits.flatMap((commit) => commit.files))];
  const subjects = [...new Set(commits.map((commit) => commit.subject))];
  return {
    repo: activity.repo,
    commitCount: commits.length,
    files,
    subjects,
  };
}

export function renderStandup({ date, activities, today = [], blockers = [] }) {
  const summaries = activities.map(summarizeRepo).filter((item) => item.commitCount > 0);
  const lines = [`# Standup — ${date}`, "", "## Yesterday"];

  if (summaries.length === 0) {
    lines.push("- No committed Git activity was found. Add notes below for research, meetings, or uncommitted work.");
  } else {
    for (const summary of summaries) {
      const repoName = summary.repo.split(/[\\/]/).filter(Boolean).at(-1);
      for (const subject of summary.subjects) lines.push(`- **${repoName}:** ${subject}`);
    }
  }

  lines.push("", "## Today");
  lines.push(...(today.length ? today.map((item) => `- ${item}`) : ["- _Add today's priorities._"]));
  lines.push("", "## Blockers");
  lines.push(...(blockers.length ? blockers.map((item) => `- ${item}`) : ["- None recorded."]));
  lines.push("", "<details>", "<summary>Evidence</summary>", "");

  if (summaries.length === 0) {
    lines.push("No commit evidence was found for this period.");
  } else {
    for (const activity of activities.filter((item) => item.commits.length > 0)) {
      lines.push(`### ${activity.repo}`, "");
      for (const commit of activity.commits) {
        lines.push(`- \`${commit.hash.slice(0, 8)}\` ${commit.subject} — ${commit.timestamp}`);
        if (commit.files.length) lines.push(`  - Files: ${commit.files.slice(0, 8).map((f) => `\`${f}\``).join(", ")}${commit.files.length > 8 ? ` +${commit.files.length - 8} more` : ""}`);
      }
      lines.push("");
    }
  }
  lines.push("</details>", "");
  return lines.join("\n");
}

export function loadConfig(configDir, cwd) {
  const path = configDir ? resolve(configDir, "config.json") : null;
  let config = {};
  if (path && existsSync(path)) {
    try { config = JSON.parse(readFileSync(path, "utf8")); } catch { /* use defaults */ }
  }
  const repos = Array.isArray(config.repositories) ? config.repositories : [];
  return {
    repositories: [...new Set([cwd, ...repos].filter(Boolean).map((item) => resolve(item)))],
    today: Array.isArray(config.today) ? config.today : [],
    blockers: Array.isArray(config.blockers) ? config.blockers : [],
  };
}

export function cwdFromHerdrContext(raw, fallback = process.cwd()) {
  try {
    const context = typeof raw === "string" ? JSON.parse(raw || "{}") : (raw || {});
    return context.worktree?.path
      || context.worktree_path
      || context.workspace?.cwd
      || context.workspace_cwd
      || context.focused_pane_cwd
      || fallback;
  } catch {
    return fallback;
  }
}
