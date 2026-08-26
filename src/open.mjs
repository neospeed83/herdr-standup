#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import process from "node:process";

const herdr = process.env.HERDR_BIN_PATH || "herdr";
const result = spawnSync(herdr, [
  "plugin", "pane", "open",
  "--plugin", "herdr-standup",
  "--entrypoint", "standup",
  "--placement", "popup",
  "--focus"
], { stdio: "inherit" });

process.exit(result.status ?? 1);
