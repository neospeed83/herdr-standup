export const shortcut = "prefix+u";

export const bindingBlock = `[[keys.command]]
key = "${shortcut}"
type = "plugin_action"
command = "herdr-standup.open"
description = "open Herdr Standup"`;

export function installBinding(source) {
  if (/command\s*=\s*["']herdr-standup\.open["']/.test(source)) {
    return { content: source, changed: false, reason: "already-installed" };
  }

  const escaped = shortcut.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const shortcutPattern = new RegExp(`key\\s*=\\s*["']${escaped}["']`);
  if (shortcutPattern.test(source)) {
    throw new Error(`${shortcut} is already assigned in config.toml; Herdr Standup did not overwrite it.`);
  }

  const trimmed = source.trimEnd();
  return {
    content: `${trimmed}${trimmed ? "\n\n" : ""}${bindingBlock}\n`,
    changed: true,
    reason: "installed",
  };
}

export function removeBinding(source) {
  const escaped = bindingBlock.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const pattern = new RegExp(`(?:^|\\n\\n)${escaped}\\n?`, "m");
  const content = source.replace(pattern, (match) => match.startsWith("\n\n") ? "\n" : "");
  return { content, changed: content !== source };
}
