import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = process.cwd();
const paths = {
  pkg: resolve(root, 'package.json'),
  frontend: resolve(root, 'frontend/package.json'),
  tauri: resolve(root, 'tauri.conf.json'),
  cargo: resolve(root, 'Cargo.toml'),
};

try {
  const pkg = JSON.parse(readFileSync(paths.pkg, 'utf8'));
  const newVersion = pkg.version;
  console.log(`Syncing version: ${newVersion}...`);

  // 1. frontend/package.json
  const frontendPkg = JSON.parse(readFileSync(paths.frontend, 'utf8'));
  frontendPkg.version = newVersion;
  writeFileSync(paths.frontend, JSON.stringify(frontendPkg, null, 2) + '\n');
  console.log(`✓ frontend/package.json updated`);

  // 2. tauri.conf.json
  // WiX MSI explicitly forbids non-numeric build identifiers like -rc1
  const tauriVersion = newVersion.split('-')[0];
  const tauri = JSON.parse(readFileSync(paths.tauri, 'utf8'));
  tauri.version = tauriVersion;
  writeFileSync(paths.tauri, JSON.stringify(tauri, null, 2) + '\n');
  console.log(`✓ tauri.conf.json updated to ${tauriVersion}`);

  // 3. Cargo.toml
  let cargo = readFileSync(paths.cargo, 'utf8');
  cargo = cargo.replace(/^(version\s*=\s*")([^"]*)(")/m, `$1${newVersion}$3`);
  writeFileSync(paths.cargo, cargo);
  console.log(`✓ Cargo.toml updated`);

} catch (err) {
  console.error("Error syncing versions:", err);
  process.exit(1);
}
