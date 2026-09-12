#!/usr/bin/env node
// Regenerates public/THIRD_PARTY_LICENSES.txt from the current dependency trees.
//
//   pnpm gen:licenses
//
// Output = scripts/licenses-preamble.txt  +  Rust crate licenses (cargo-about)  +
// npm production dependency licenses (pnpm + each package's own LICENSE file, falling
// back to the canonical text for common SPDX ids when a package ships none).
//
// Requires `cargo-about` on PATH (cargo install cargo-about --features cli). The CI
// licenses workflow runs this and fails if the committed file is out of date.

import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync, readdirSync } from "node:fs";
import { join, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const out = join(root, "public", "THIRD_PARTY_LICENSES.txt");
const templatesDir = join(root, "scripts", "license-templates");

// Matches a package's own LICENSE/LICENCE/COPYING/NOTICE file. Deliberately excludes
// `.spdx` - some packages (the @tauri-apps/plugin-* family, among others) ship only a
// `LICENSE.spdx` *metadata* manifest (SPDXVersion/PackageLicenseDeclared fields), not
// license terms; matching that file was silently substituting SPDX metadata for the
// actual text it's supposed to represent.
const LICENSE_FILE_RE = /^(license|licence|copying|notice)([-_.].*)?\.(md|txt|rst)$/i;
const LICENSE_FILE_RE_NO_EXT = /^(license|licence|copying|notice)$/i;

function isLicenseFile(name) {
  return LICENSE_FILE_RE.test(name) || LICENSE_FILE_RE_NO_EXT.test(name);
}

function run(cmd, args, cwd) {
  return execFileSync(cmd, args, { cwd, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
}

// Canonical text for common short permissive licenses, used only when a package ships no
// license file of its own - reproducing the actual terms instead of just naming them, per
// https://spdx.org/licenses/. Apache-2.0 reuses the repo's own LICENSE file (identical
// text, no point duplicating it as a separate template).
const STANDARD_LICENSE_TEXT = Object.fromEntries(
  [
    ["MIT", join(templatesDir, "MIT.txt")],
    ["ISC", join(templatesDir, "ISC.txt")],
    ["0BSD", join(templatesDir, "0BSD.txt")],
    ["BSD-2-Clause", join(templatesDir, "BSD-2-Clause.txt")],
    ["BSD-3-Clause", join(templatesDir, "BSD-3-Clause.txt")],
    ["Apache-2.0", join(root, "LICENSE")],
  ].map(([id, path]) => [id, readFileSync(path, "utf8").trimEnd()]),
);

/** Resolves a (possibly compound, e.g. "Apache-2.0 OR MIT") SPDX expression to canonical
 * text for whichever disjunct we have a template for. */
function standardTextFor(licenseExpr) {
  for (const candidate of licenseExpr.split(/\s+OR\s+/i).map((s) => s.trim())) {
    if (STANDARD_LICENSE_TEXT[candidate]) return STANDARD_LICENSE_TEXT[candidate];
  }
  return null;
}

// --- 1. hand-written preamble (app license + Linux bundled-library / LGPL notice) ---
const preamble = readFileSync(join(root, "scripts", "licenses-preamble.txt"), "utf8").trimEnd();

// --- 2. Rust crates, via cargo-about ---
console.error("Generating Rust crate licenses (cargo about)...");
const rust = run("cargo", ["about", "generate", "about.hbs"], join(root, "src-tauri")).trimEnd();

// --- 3. npm production dependencies ---
console.error("Collecting npm dependency licenses...");
const pnpmJson = JSON.parse(run("pnpm", ["licenses", "list", "--prod", "--json"], root));

// Keyed by `${name}@@${license}`, not just `${name}`: `pnpm licenses list` already merges
// same-name-same-license installs (e.g. two versions of one package) into a single entry,
// but if a name ever legitimately carried *different* licenses across versions, keying by
// name alone would silently drop one of them.
/** @type {Map<string, {name: string, license: string, author?: string, text?: string}>} */
const pkgs = new Map();
for (const [license, entries] of Object.entries(pnpmJson)) {
  for (const e of entries) {
    let text;
    for (const path0 of e.paths || []) {
      try {
        const licFile = readdirSync(path0).find(isLicenseFile);
        if (licFile) {
          text = readFileSync(join(path0, licFile), "utf8").trimEnd();
          break;
        }
      } catch {
        /* this install path is missing on disk - try the next one, if any */
      }
    }
    text ??= standardTextFor(license) ?? undefined;
    pkgs.set(`${e.name}@@${license}`, { name: e.name, license, author: e.author, text });
  }
}

let npm = [
  "================================================================================",
  "NPM PACKAGES (production dependencies)",
  "================================================================================",
  "",
  "The k8sman UI bundles the following npm packages. Each entry lists the package's",
  "SPDX license identifier followed by its license text: the package's own bundled",
  "license file where it ships one, otherwise the canonical text for that license.",
  "",
];
const sorted = [...pkgs.values()].sort(
  (a, b) => a.name.localeCompare(b.name) || a.license.localeCompare(b.license),
);
for (const p of sorted) {
  npm.push("-".repeat(80));
  npm.push(`${p.name}  -  ${p.license}${p.author ? `  (${p.author})` : ""}`);
  npm.push("-".repeat(80));
  npm.push("");
  npm.push(
    p.text ??
      `[No license file included in this package and no canonical text template for ` +
        `"${p.license}" here. The ${p.license} terms apply; see https://spdx.org/licenses/ ` +
        `for the reference text.]`,
  );
  npm.push("");
}

// --- assemble ---
const header =
  `k8sman - THIRD-PARTY LICENSE NOTICES\n` +
  `Generated by scripts/gen-licenses.mjs - do not edit by hand.\n` +
  `Run \`pnpm gen:licenses\` to regenerate.\n`;

writeFileSync(out, [header, preamble, "", rust, "", npm.join("\n"), ""].join("\n"));
console.error(`Wrote ${out}`);
