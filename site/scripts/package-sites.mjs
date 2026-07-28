import fs from "node:fs";
import path from "node:path";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const source = path.join(repositoryRoot, "site", "dist");
const destination = path.join(repositoryRoot, "dist");
const hostingSource = path.join(
  repositoryRoot,
  ".openai",
  "hosting.json"
);
const hostingDestination = path.join(
  destination,
  ".openai",
  "hosting.json"
);
const serverSource = ["index.js", "index.mjs"]
  .map((fileName) => path.join(source, "server", fileName))
  .find((filePath) => fs.existsSync(filePath));
const serverDestination = path.join(destination, "server", "index.js");

if (!serverSource) {
  throw new Error("Vinext server bundle is missing from site/dist/.");
}
if (!fs.existsSync(hostingSource)) {
  throw new Error("The connected site descriptor is missing.");
}

fs.rmSync(destination, { recursive: true, force: true });
fs.cpSync(source, destination, { recursive: true });
if (!fs.existsSync(serverDestination)) {
  fs.copyFileSync(
    path.join(destination, "server", path.basename(serverSource)),
    serverDestination
  );
}
fs.mkdirSync(path.dirname(hostingDestination), { recursive: true });
fs.copyFileSync(hostingSource, hostingDestination);

console.log("Packaged the Vinext site bundle in dist/.");
