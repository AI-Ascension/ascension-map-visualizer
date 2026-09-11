// SPDX-License-Identifier: MIT
import fs from 'node:fs';
import { createHash } from 'node:crypto';

const manifest = JSON.parse(fs.readFileSync('contract-artifacts/harness/provenance.json', 'utf8'));
if (manifest.repository !== 'https://github.com/AI-Ascension/sts2-harness'
    || !/^[0-9a-f]{40}$/.test(manifest.commit) || manifest.license !== 'MIT'
    || !Array.isArray(manifest.files) || manifest.files.length !== 13) throw new Error('invalid contract provenance');
const copies = new Set();
for (const entry of manifest.files) {
  if (!/^(contract-artifacts\/harness|fixtures\/(demo|conformance))\/[a-z-]+(?:\.schema)?\.json$/.test(entry.copy)
      || !/^(docs|crates\/harness\/tests\/fixtures)\/[A-Za-z0-9/_.-]+$/.test(entry.source)
      || entry.source.split('/').some(part => part === '..' || part === '.')
      || !/^[0-9a-f]{64}$/.test(entry.sha256) || copies.has(entry.copy)) throw new Error('invalid contract file record');
  copies.add(entry.copy);
  const bytes = fs.readFileSync(entry.copy);
  if (createHash('sha256').update(bytes).digest('hex') !== entry.sha256) throw new Error(`contract copy changed: ${entry.copy}`);
}
process.stdout.write(`Verified ${copies.size} inert owner artifacts at ${manifest.commit}.\n`);
