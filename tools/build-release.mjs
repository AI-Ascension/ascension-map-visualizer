// SPDX-License-Identifier: MIT
// Build distributable binaries without embedding the operator's source/cache paths.
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export function localBuildPaths() {
  const roots = [
    [os.homedir(), '/build/home'],
    [process.env.CARGO_HOME || path.join(os.homedir(), '.cargo'), '/build/cargo'],
    [process.env.RUSTUP_HOME || path.join(os.homedir(), '.rustup'), '/build/toolchain'],
    [process.cwd(), '/build/source'],
    [process.env.CARGO_TARGET_DIR || process.env.CARGO_BUILD_TARGET_DIR || path.join(process.cwd(), 'target'), '/build/target'],
  ];
  const mappings = new Map();
  for (const [root, replacement] of roots) {
    const absolute = path.resolve(root);
    if (absolute === path.parse(absolute).root) throw new Error('build path must not be a filesystem root');
    for (const spelling of [absolute, fs.existsSync(absolute) ? fs.realpathSync(absolute) : absolute]) {
      mappings.set(spelling, replacement);
      if (process.platform === 'win32') mappings.set(spelling.replaceAll('\\', '/'), replacement);
    }
  }
  // rustc uses the last matching rule; retain the most specific prefix last.
  return [...mappings].sort(([left], [right]) => left.length - right.length);
}

export function rejectLocalBuildPaths(bytes, mappings = localBuildPaths()) {
  for (const [prefix] of mappings) {
    for (const encoding of ['utf8', 'utf16le']) {
      if (bytes.includes(Buffer.from(prefix, encoding))) throw new Error('distribution contains a local build path');
    }
  }
}

export function buildRelease() {
  const inherited = process.env.CARGO_ENCODED_RUSTFLAGS !== undefined
    ? process.env.CARGO_ENCODED_RUSTFLAGS.split('\x1f').filter(Boolean)
    : (process.env.RUSTFLAGS || '').split(/\s+/).filter(Boolean);
  const flags = [...inherited, ...localBuildPaths().map(([from, to]) => `--remap-path-prefix=${from}=${to}`)];
  if (process.platform === 'win32') flags.push('-Clink-arg=/PDBALTPATH:map-visualizer.pdb');
  const env = { ...process.env, CARGO_ENCODED_RUSTFLAGS: flags.join('\x1f'), CARGO_PROFILE_RELEASE_DEBUG: '0', CARGO_PROFILE_RELEASE_STRIP: 'debuginfo' };
  execFileSync('cargo', ['build', '--release', '--locked', '--package', 'map-visualizer'], { env, stdio: 'inherit' });
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) buildRelease();
