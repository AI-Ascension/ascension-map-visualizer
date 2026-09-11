// SPDX-License-Identifier: MIT
// Package only explicit distributable files; execute from the repository root.
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { buildRelease, rejectLocalBuildPaths } from './build-release.mjs';

const output = process.argv[2];
if (!output || process.argv.length !== 3) throw new Error('usage: node tools/package.mjs NEW_DIRECTORY');
if (fs.existsSync(output) || fs.existsSync(`${path.resolve(output)}.tar.gz`)) throw new Error('distribution output already exists');
const target = /^host: (.+)$/m.exec(execFileSync('rustc', ['-vV'], { encoding: 'utf8' }))?.[1];
if (!['x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc'].includes(target)) throw new Error('unsupported native packaging target');
const executable = target.includes('windows') ? 'map-visualizer.exe' : 'map-visualizer';
const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--locked', '--no-deps', '--format-version', '1'], { maxBuffer: 1024 * 1024 }));
buildRelease();
rejectLocalBuildPaths(fs.readFileSync(path.join(metadata.target_directory, 'release', executable)));
fs.mkdirSync(output);
fs.copyFileSync(path.join(metadata.target_directory, 'release', executable), path.join(output, executable), fs.constants.COPYFILE_EXCL);
for (const file of ['README.md', 'LICENSE', 'THIRD_PARTY_NOTICES.md']) fs.copyFileSync(file, path.join(output, file), fs.constants.COPYFILE_EXCL);
execFileSync(process.execPath, ['tools/licenses.mjs', path.join(output, 'licenses'), target], { stdio: 'inherit' });
const artifacts = [];
function scan(directory, relative = '') {
  for (const item of fs.readdirSync(directory, { withFileTypes: true }).sort((a,b) => a.name.localeCompare(b.name, 'en'))) {
    const name = relative ? `${relative}/${item.name}` : item.name;
    if (item.isDirectory()) scan(path.join(directory,item.name), name);
    else if (item.isFile()) {
      const bytes = fs.readFileSync(path.join(directory,item.name));
      rejectLocalBuildPaths(bytes);
      artifacts.push({ file: name, sha256: createHash('sha256').update(bytes).digest('hex') });
    }
    else throw new Error('non-regular distribution file');
  }
}
const commit = execFileSync('git', ['rev-parse','HEAD'], { encoding:'utf8' }).trim();
const dirty = execFileSync('git', ['status','--porcelain'], { encoding:'utf8' }).trim().length > 0;
fs.writeFileSync(path.join(output,'build.json'), `${JSON.stringify({ target, commit, dirty, native: true, version: metadata.packages.find(p => p.name === 'map-visualizer').version }, null, 2)}\n`, { flag:'wx' });
scan(output);
fs.writeFileSync(path.join(output, 'SHA256SUMS'), artifacts.map(item => `${item.sha256}  ${item.file}\n`).join(''), { flag: 'wx' });
const archive = `${path.resolve(output)}.tar.gz`;
if (fs.existsSync(archive)) throw new Error('distribution archive already exists');
execFileSync('tar', ['-czf', archive, '-C', path.resolve(output), '.'], { stdio: 'inherit' });
process.stdout.write(`Packaged ${executable} for ${target}.\n`);
