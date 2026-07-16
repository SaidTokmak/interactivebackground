import { readdir, stat } from "node:fs/promises";
import { extname, join } from "node:path";

const budgets = new Map([[".js", 400 * 1024], [".css", 80 * 1024]]);
const totals = new Map([...budgets.keys()].map((extension) => [extension, 0]));

async function walk(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) await walk(path);
    else if (totals.has(extname(entry.name))) totals.set(extname(entry.name), totals.get(extname(entry.name)) + (await stat(path)).size);
  }
}

await walk("dist");
let failed = false;
for (const [extension, budget] of budgets) {
  const total = totals.get(extension);
  console.log(`${extension.slice(1).toUpperCase()}: ${(total / 1024).toFixed(1)} KiB / ${(budget / 1024).toFixed(0)} KiB`);
  if (total > budget) failed = true;
}
if (failed) {
  console.error("Frontend bundle beta performans bütçesini aşıyor.");
  process.exitCode = 1;
}
