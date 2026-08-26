#!/usr/bin/env node
import { mkdirSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import process from "node:process";
import { collectGitActivity, cwdFromHerdrContext, dayWindow, loadConfig, renderStandup } from "./lib.mjs";

function contextCwd() {
  return cwdFromHerdrContext(process.env.HERDR_PLUGIN_CONTEXT_JSON, process.cwd());
}

function generate() {
  const now = new Date();
  const window = dayWindow(now, -1);
  const config = loadConfig(process.env.HERDR_PLUGIN_CONFIG_DIR, contextCwd());
  const activities = config.repositories
    .map((repo) => collectGitActivity(repo, window))
    .filter(Boolean);
  const date = window.start.toLocaleDateString(undefined, {
    weekday: "long", year: "numeric", month: "long", day: "numeric"
  });
  const markdown = renderStandup({ date, activities, today: config.today, blockers: config.blockers });
  const stateDir = process.env.HERDR_PLUGIN_STATE_DIR || join(process.cwd(), ".standup");
  mkdirSync(stateDir, { recursive: true });
  const stamp = [window.start.getFullYear(), String(window.start.getMonth() + 1).padStart(2, "0"), String(window.start.getDate()).padStart(2, "0")].join("-");
  const outputPath = join(stateDir, `${stamp}.md`);
  writeFileSync(outputPath, markdown, "utf8");
  return { markdown, outputPath };
}

const command = process.argv[2] || "generate";
const result = generate();

if (command === "show") {
  process.stdout.write("\x1b[2J\x1b[H");
  process.stdout.write(`${result.markdown}\n\nSaved to ${result.outputPath}\n`);
  process.stdout.write("\nPress Enter to close…");
  if (process.stdin.isTTY) {
    process.stdin.setEncoding("utf8");
    process.stdin.once("data", () => process.exit(0));
  }
} else {
  process.stdout.write(`${result.markdown}\nSaved: ${result.outputPath}\n`);
}
