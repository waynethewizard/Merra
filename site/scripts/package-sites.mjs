import fs from "node:fs";
import path from "node:path";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const source = path.join(repositoryRoot, "site", ".open-next");
const workerSource = path.join(source, "worker.js");
const destination = path.join(repositoryRoot, "dist");
const serverDestination = path.join(destination, "server");
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
  throw new Error("The OpenNext worker bundle is missing.");
}
if (!fs.existsSync(hostingSource)) {
  throw new Error("The connected site descriptor is missing.");
}

fs.rmSync(destination, { recursive: true, force: true });
fs.cpSync(source, destination, { recursive: true });
fs.mkdirSync(serverDestination, { recursive: true });
fs.writeFileSync(
  path.join(serverDestination, "index.js"),
  'export { default } from "../worker.js";\n'
);
fs.writeFileSync(
  path.join(destination, "package.json"),
  `${JSON.stringify({ type: "module" }, null, 2)}\n`
);
fs.mkdirSync(path.dirname(hostingDestination), { recursive: true });
fs.copyFileSync(hostingSource, hostingDestination);

console.log("Packaged the OpenNext worker bundle in dist/.");
