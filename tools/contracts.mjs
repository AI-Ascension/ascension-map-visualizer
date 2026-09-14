// SPDX-License-Identifier: MIT
import fs from 'node:fs';
import { createHash } from 'node:crypto';

const manifest = JSON.parse(fs.readFileSync('contract-artifacts/harness/provenance.json', 'utf8'));
if (manifest.repository !== 'https://github.com/AI-Ascension/sts2-harness'
    || manifest.license !== 'MIT'
    || !Array.isArray(manifest.files) || manifest.files.length !== 13) throw new Error('invalid contract provenance');
const assembly = JSON.parse(fs.readFileSync('docs/ASSEMBLY_MANIFEST.json', 'utf8'));
const artifactRevision = assembly.contracts?.harness_artifact_revision;
if (!/^[0-9a-f]{40}$/.test(artifactRevision ?? '')) throw new Error('assembly manifest lacks the harness artifact revision');
const copies = new Set();
const revisions = { fixture: new Set(), contract: new Set() };
for (const entry of manifest.files) {
  if (!/^(contract-artifacts\/harness|fixtures\/(demo|conformance))\/[a-z-]+(?:\.schema)?\.json$/.test(entry.copy)
      || !/^(docs|crates\/harness\/tests\/fixtures)\/[A-Za-z0-9/_.-]+$/.test(entry.source)
      || entry.source.split('/').some(part => part === '..' || part === '.')
      || !/^[0-9a-f]{40}$/.test(entry.commit) || !/^[0-9a-f]{64}$/.test(entry.sha256) || copies.has(entry.copy)) throw new Error('invalid contract file record');
  copies.add(entry.copy);
  const bytes = fs.readFileSync(entry.copy);
  if (createHash('sha256').update(bytes).digest('hex') !== entry.sha256) throw new Error(`contract copy changed: ${entry.copy}`);
  revisions[entry.copy.startsWith('fixtures/') ? 'fixture' : 'contract'].add(entry.commit);
}
// A copy must cite the owner revision that still contains it. Schema copies form one reviewed group,
// and the fixture group must be the revision recorded for the assembly, so a stale or invented pin fails here.
if (revisions.contract.size !== 1) throw new Error('contract copies must record one owner revision');
if (revisions.fixture.size !== 1 || !revisions.fixture.has(artifactRevision)) throw new Error('fixture provenance does not match the recorded harness artifact revision');
process.stdout.write(`Verified ${copies.size} inert owner artifacts at harness revisions ${[...revisions.contract, ...revisions.fixture].join(', ')}.\n`);
