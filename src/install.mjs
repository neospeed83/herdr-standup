#!/usr/bin/env node
import { constants, copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import process from "node:process";
import { installBinding, removeBinding, shortcut } from "./config.mjs";

function configPath() {
  if (process.platform === "win32") {
    const appData = process.env.APPDATA;
    if (!appData) throw new Error("APPDATA is unavailable; cannot locate Herdr config.toml.");
    return join(appData, "herdr", "config.toml");
  }
  const base = process.env.XDG_CONFIG_HOME || join(process.env.HOME || "", ".config");
  if (!base) throw new Error("HOME and XDG_CONFIG_HOME are unavailable; cannot locate Herdr config.toml.");
  return join(base, "herdr", "config.toml");
}

const path = configPath();
const source = existsSync(path) ? readFileSync(path, "utf8") : "";
const mode = process.argv[2] || "install";
const result = mode === "remove" ? removeBinding(source) : installBinding(source);

if (result.changed) {
  mkdirSync(dirname(path), { recursive: true });
  if (existsSync(path)) {
    try { copyFileSync(path, `${path}.bak-herdr-standup`, constants.COPYFILE_EXCL); } catch (error) {
      if (error.code !== "EEXIST") throw error;
    }
  }
  writeFileSync(path, result.content, "utf8");
}

if (mode === "remove") {
  process.stdout.write(result.changed ? `Removed ${shortcut} from ${path}.\n` : "Herdr Standup keybinding was not present.\n");
} else {
  process.stdout.write(result.changed ? `Installed ${shortcut} in ${path}.\n` : `Herdr Standup keybinding already exists in ${path}.\n`);
}
