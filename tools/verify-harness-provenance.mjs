// Read immutable historical harness bytes; never substitute the current default branch.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const provenancePath = path.join(repositoryRoot, 'contract-artifacts/harness/provenance.json');
const sourceRepository = 'https://github.com/AI-Ascension/sts2-harness';

function fail(message) { throw new Error(`invalid historical harness provenance: ${message}`); }

function regularFile(root, name, label) {
  if (typeof name !== 'string' || !name || name.length > 500 || name.includes('\\') || name.includes('\0') || path.isAbsolute(name)) fail(`unsafe ${label} path`);
  const parts = name.split('/');
  if (parts.some((part) => !part || part === '.' || part === '..')) fail(`unsafe ${label} path`);
  let current = root;
  for (const part of parts) {
    current = path.join(current, part);
    const stat = fs.lstatSync(current, {throwIfNoEntry: false});
    if (stat?.isSymbolicLink()) fail(`symlink ${label} path`);
  }
  const stat = fs.statSync(current, {throwIfNoEntry: false});
  if (!stat?.isFile()) fail(`missing ${label} file`);
  return current;
}

export function validateManifest(raw, root = repositoryRoot) {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw) || Object.keys(raw).length !== 4
    || raw.repository !== sourceRepository || !/^[0-9a-f]{40}$/.test(raw.commit)
    || typeof raw.license !== 'string' || !raw.license || !Array.isArray(raw.files) || raw.files.length === 0) fail('schema');
  const sources = new Set();
  const copies = new Set();
  for (const entry of raw.files) {
    if (!entry || typeof entry !== 'object' || Array.isArray(entry) || Object.keys(entry).length !== 3
      || !/^[a-f0-9]{64}$/.test(entry.sha256)) fail('file entry schema');
    if (sources.has(entry.source) || copies.has(entry.copy)) fail('duplicate source or copy path');
    sources.add(entry.source); copies.add(entry.copy);
    regularFile(root, entry.copy, 'copied');
    if (typeof entry.source !== 'string' || !entry.source || entry.source.includes('\\') || entry.source.includes('\0')
      || path.isAbsolute(entry.source) || entry.source.split('/').some((part) => !part || part === '.' || part === '..')) fail('unsafe source path');
  }
  return raw;
}

const manifest = validateManifest(JSON.parse(fs.readFileSync(provenancePath, 'utf8')));
if (process.argv.includes('--validate-only')) {
  process.stdout.write(`Validated ${manifest.files.length} historical harness provenance entries.\n`);
  process.exit(0);
}

const checkout = fs.mkdtempSync(path.join(os.tmpdir(), 'map-harness-provenance-'));
try {
  execFileSync('git', ['init', '--quiet', checkout]);
  execFileSync('git', ['-C', checkout, 'remote', 'add', 'origin', manifest.repository]);
  execFileSync('git', ['-C', checkout, 'fetch', '--depth=1', 'origin', manifest.commit], {stdio: 'inherit'});
  execFileSync('git', ['-C', checkout, 'checkout', '--quiet', '--detach', 'FETCH_HEAD']);
  if (execFileSync('git', ['-C', checkout, 'rev-parse', 'HEAD'], {encoding: 'utf8'}).trim() !== manifest.commit) throw new Error('historical checkout did not resolve pinned commit');
  for (const entry of manifest.files) {
    const source = fs.readFileSync(regularFile(checkout, entry.source, 'source'));
    const copy = fs.readFileSync(regularFile(repositoryRoot, entry.copy, 'copied'));
    const sourceDigest = createHash('sha256').update(source).digest('hex');
    if (sourceDigest !== entry.sha256 || !source.equals(copy)) throw new Error(`historical provenance mismatch: ${entry.copy}`);
  }
  process.stdout.write(`Verified ${manifest.files.length} historical harness files at ${manifest.commit}.\n`);
} finally {
  fs.rmSync(checkout, {recursive: true, force: true});
}
