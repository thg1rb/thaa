import { readFile, readdir, stat } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const repositoryRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);
const ignoredDirectories = new Set([
  ".git",
  ".agents",
  "node_modules",
  "dist",
  "coverage",
  "target",
]);
const markdownFiles = [];

async function collectMarkdown(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.isDirectory() && ignoredDirectories.has(entry.name)) continue;

    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      await collectMarkdown(entryPath);
    } else if (entry.isFile() && entry.name.endsWith(".md")) {
      markdownFiles.push(entryPath);
    }
  }
}

function linkTargets(markdown) {
  const targets = [];
  const expression = /\]\(\s*(?:<([^>]+)>|([^\s)]+))(?:\s+[^)]*)?\)/g;
  for (const match of markdown.matchAll(expression)) {
    targets.push(match[1] ?? match[2]);
  }
  return targets;
}

await collectMarkdown(repositoryRoot);

const missingLinks = [];
for (const markdownFile of markdownFiles) {
  const contents = await readFile(markdownFile, "utf8");
  for (const target of linkTargets(contents)) {
    if (
      !target ||
      target.startsWith("#") ||
      /^[a-z][a-z\d+.-]*:/i.test(target) ||
      target.startsWith("//")
    ) {
      continue;
    }

    const [pathname] = target.split(/[?#]/, 1);
    if (!pathname) continue;

    let decodedPath;
    try {
      decodedPath = decodeURIComponent(pathname);
    } catch {
      missingLinks.push(
        `${path.relative(repositoryRoot, markdownFile)}: malformed URL encoding in ${target}`,
      );
      continue;
    }

    const resolvedPath = path.resolve(path.dirname(markdownFile), decodedPath);
    try {
      await stat(resolvedPath);
    } catch {
      missingLinks.push(
        `${path.relative(repositoryRoot, markdownFile)}: missing ${target}`,
      );
    }
  }
}

if (missingLinks.length > 0) {
  process.stderr.write(`${missingLinks.join("\n")}\n`);
  process.exitCode = 1;
} else {
  process.stdout.write(
    `Checked ${markdownFiles.length} Markdown files; local link targets exist.\n`,
  );
}
