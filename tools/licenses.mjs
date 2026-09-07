// SPDX-License-Identifier: MIT
// Collect upstream notice text without putting machine paths in artifacts.
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';

const output = process.argv[2];
if (!output || process.argv.length > 4) throw new Error('usage: node tools/licenses.mjs NEW_DIRECTORY [TARGET_TRIPLE]');
const target = process.argv[3] || /^host: (.+)$/m.exec(execFileSync('rustc', ['-vV'], { encoding: 'utf8' }))?.[1];
if (!target || !/^[A-Za-z0-9_.-]+$/.test(target)) throw new Error('invalid native target');
const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--locked', '--format-version', '1', '--filter-platform', target], { maxBuffer: 16 * 1024 * 1024 }));
const nodes = new Map(metadata.resolve.nodes.map(node => [node.id, node]));
const selected = new Set();
function visit(id) {
  if (selected.has(id)) return;
  selected.add(id);
  for (const dependency of nodes.get(id)?.deps || []) {
    if (dependency.dep_kinds.some(kind => kind.kind !== 'dev')) visit(dependency.pkg);
  }
}
visit(metadata.packages.find(pkg => pkg.name === 'map-visualizer')?.id);
const inventory = [];
fs.mkdirSync(output);
for (const pkg of metadata.packages.filter(pkg => selected.has(pkg.id)).sort((a, b) => `${a.name}@${a.version}` < `${b.name}@${b.version}` ? -1 : 1)) {
  if (!pkg.license) throw new Error(`missing declared license: ${pkg.name}@${pkg.version}`);
  const key = `${pkg.name}-${pkg.version}-${createHash('sha256').update(pkg.source || 'workspace').digest('hex').slice(0, 8)}`;
  if (!/^[A-Za-z0-9_.-]+$/.test(key)) throw new Error('unsafe package identity');
  let directory = path.dirname(pkg.manifest_path);
  let notices = [];
  for (let level = 0; level < 4 && !notices.length; level += 1) {
    notices = fs.readdirSync(directory, { withFileTypes: true })
      .filter(entry => entry.isFile() && /^(LICENSE|LICENCE|COPYING|NOTICE)([._-].*)?$/i.test(entry.name))
      .map(entry => ({ name: entry.name, file: path.join(directory, entry.name) }));
    if (!notices.length) directory = path.dirname(directory);
  }
  if (pkg.license_file) {
    const file = path.resolve(path.dirname(pkg.manifest_path), pkg.license_file);
    if (!notices.some(notice => notice.file === file)) notices.push({ name: path.basename(file), file });
  }
  let noticeSource = `${pkg.name}@${pkg.version}`;
  // These MIT workspace crates share one release commit and copyright owner;
  // their registry archives omit the workspace notice copied into jsonschema.
  if (!notices.length && ['jsonschema-regex', 'jsonschema-value', 'referencing'].includes(pkg.name)) {
    const donor = metadata.packages.find(item => item.name === 'jsonschema' && item.version === pkg.version);
    const revision = item => JSON.parse(fs.readFileSync(path.join(path.dirname(item.manifest_path), '.cargo_vcs_info.json'), 'utf8')).git.sha1;
    if (!donor || donor.repository !== pkg.repository || donor.license !== pkg.license || revision(donor) !== revision(pkg)) {
      throw new Error('workspace license provenance mismatch');
    }
    notices = [{ name: 'LICENSE', file: path.join(path.dirname(donor.manifest_path), 'LICENSE') }];
    noticeSource = `${donor.name}@${donor.version} (${revision(donor)})`;
  }
  if (!notices.length && ['uuid-simd', 'vsimd'].includes(pkg.name)) {
    const base = path.join(metadata.workspace_root, 'third-party/licenses/simd');
    const provenance = JSON.parse(fs.readFileSync(path.join(base, 'provenance.json'), 'utf8'));
    const vcs = JSON.parse(fs.readFileSync(path.join(path.dirname(pkg.manifest_path), '.cargo_vcs_info.json'), 'utf8'));
    const file = path.join(base, 'LICENSE');
    if (!provenance.packages.includes(`${pkg.name}@${pkg.version}`) || pkg.repository !== provenance.repository
      || pkg.license !== 'MIT' || vcs.git.sha1 !== provenance.revision
      || createHash('sha256').update(fs.readFileSync(file)).digest('hex') !== provenance.sha256) {
      throw new Error('curated license provenance mismatch');
    }
    notices = [{ name: 'LICENSE', file }];
    noticeSource = `${provenance.repository}@${provenance.revision}/${provenance.path}`;
  }
  if (!notices.length) throw new Error(`missing upstream notice text: ${pkg.name}@${pkg.version}`);
  fs.mkdirSync(path.join(output, key));
  const files = [];
  for (const notice of notices.sort((a, b) => a.name < b.name ? -1 : 1)) {
    const stat = fs.statSync(notice.file);
    if (!stat.isFile() || stat.size > 1024 * 1024) throw new Error('invalid upstream notice file');
    const bytes = fs.readFileSync(notice.file);
    fs.writeFileSync(path.join(output, key, notice.name), bytes, { flag: 'wx' });
    files.push({ file: `${key}/${notice.name}`, sha256: createHash('sha256').update(bytes).digest('hex') });
  }
  inventory.push({ name: pkg.name, version: pkg.version, license: pkg.license, source: pkg.source || 'workspace', target, notice_source: noticeSource, notices: files });
}
fs.writeFileSync(path.join(output, 'inventory.json'), `${JSON.stringify(inventory, null, 2)}\n`, { flag: 'wx' });
process.stdout.write(`${inventory.length} dependency notice records written.\n`);
