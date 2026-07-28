import fs from "node:fs";
import path from "node:path";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const source = path.join(repositoryRoot, "site", "out");
const destination = path.join(repositoryRoot, "dist");

if (!fs.existsSync(source)) {
  throw new Error("Static site export is missing. Run the site build first.");
}

fs.rmSync(destination, { recursive: true, force: true });
fs.cpSync(source, destination, { recursive: true });

console.log("Copied the static site export to dist/.");
