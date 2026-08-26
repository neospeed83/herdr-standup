import test from "node:test";
import assert from "node:assert/strict";
import { cwdFromHerdrContext, dayWindow, parseGitRecords, renderStandup } from "../src/lib.mjs";

test("dayWindow returns local yesterday boundaries", () => {
  const now = new Date(2026, 7, 26, 15, 30);
  const { start, end } = dayWindow(now, -1);
  assert.deepEqual([start.getFullYear(), start.getMonth(), start.getDate(), start.getHours()], [2026, 7, 25, 0]);
  assert.equal(end.getTime() - start.getTime(), 24 * 60 * 60 * 1000);
});

test("parseGitRecords keeps commit evidence and files", () => {
  const input = "\x1eabc123\x1f2026-08-25T10:00:00-05:00\x1fFix auth callback\x1fAkash\nsrc/auth.ts\ntest/auth.test.ts\n";
  assert.deepEqual(parseGitRecords(input), [{
    hash: "abc123", timestamp: "2026-08-25T10:00:00-05:00", subject: "Fix auth callback", author: "Akash",
    files: ["src/auth.ts", "test/auth.test.ts"]
  }]);
});

test("renderStandup creates readable sections and evidence", () => {
  const markdown = renderStandup({
    date: "Tuesday, August 25, 2026",
    activities: [{ repo: "/work/atlas", commits: [{ hash: "abc12345", timestamp: "2026-08-25T10:00:00-05:00", subject: "Fix auth callback", author: "Akash", files: ["src/auth.ts"] }] }],
    today: ["Ship the callback fix"], blockers: ["Need redirect decision"]
  });
  assert.match(markdown, /\*\*atlas:\*\* Fix auth callback/);
  assert.match(markdown, /Ship the callback fix/);
  assert.match(markdown, /Need redirect decision/);
  assert.match(markdown, /`abc12345`/);
});

test("cwdFromHerdrContext understands Herdr CLI action context", () => {
  const raw = JSON.stringify({ workspace_cwd: "/work/atlas", focused_pane_cwd: "/work/other" });
  assert.equal(cwdFromHerdrContext(raw, "/fallback"), "/work/atlas");
  assert.equal(cwdFromHerdrContext("not-json", "/fallback"), "/fallback");
});
