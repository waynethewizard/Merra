import fs from "node:fs";
import path from "node:path";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const openNextSource = path.join(repositoryRoot, "site", ".open-next");
const workerSource = path.join(
  repositoryRoot,
  "site",
  ".worker-dist",
  "worker.js"
);
const assetsSource = path.join(openNextSource, "assets");
const destination = path.join(repositoryRoot, "dist");
const serverDestination = path.join(destination, "server");
const assetsDestination = path.join(destination, "assets");
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

if (!fs.existsSync(workerSource)) {
  throw new Error("The final Workers bundle is missing.");
}
if (!fs.existsSync(hostingSource)) {
  throw new Error("The connected site descriptor is missing.");
}

fs.rmSync(destination, { recursive: true, force: true });
fs.mkdirSync(serverDestination, { recursive: true });
fs.cpSync(assetsSource, assetsDestination, { recursive: true });
const bundledWorker = fs
  .readFileSync(workerSource, "utf8")
  .replace(/\n\/\/# sourceMappingURL=worker\.js\.map\s*$/, "\n");
fs.writeFileSync(
  path.join(serverDestination, "index.js"),
  bundledWorker
);
fs.writeFileSync(
  path.join(destination, "package.json"),
  `${JSON.stringify({ type: "module" }, null, 2)}\n`
);
fs.writeFileSync(
  path.join(serverDestination, "package.json"),
  `${JSON.stringify({ type: "module" }, null, 2)}\n`
);
fs.mkdirSync(path.dirname(hostingDestination), { recursive: true });
fs.copyFileSync(hostingSource, hostingDestination);

console.log("Packaged the OpenNext worker bundle in dist/.");
