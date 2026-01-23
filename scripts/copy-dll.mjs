import { copyFile, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "..");
const source = resolve(repoRoot, "CommDllv2.dll");
const target = resolve(repoRoot, "src-tauri", "CommDllv2.dll");

try {
  await stat(source);
} catch {
  throw new Error(`CommDllv2.dll not found at ${source}`);
}

await copyFile(source, target);
console.log(`Copied ${source} -> ${target}`);
