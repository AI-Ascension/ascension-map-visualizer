import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

const root = process.cwd();
const script = 'tools/verify-harness-provenance.mjs';
const manifestPath = 'contract-artifacts/harness/provenance.json';

function fixture() {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'map-provenance-fixture-'));
  fs.mkdirSync(path.join(temporary, 'tools'), {recursive: true});
  fs.mkdirSync(path.join(temporary, 'contract-artifacts'), {recursive: true});
  fs.copyFileSync(path.join(root, script), path.join(temporary, script));
  fs.cpSync(path.join(root, 'contract-artifacts/harness'), path.join(temporary, 'contract-artifacts/harness'), {recursive: true});
  fs.cpSync(path.join(root, 'fixtures'), path.join(temporary, 'fixtures'), {recursive: true});
  return temporary;
}

function validate(cwd) {
  return spawnSync(process.execPath, [script, '--validate-only'], {cwd, encoding: 'utf8'});
}

test('validates committed provenance before network fetch', () => {
  const result = validate(root);
  assert.equal(result.status, 0, result.stderr);
});

test('rejects duplicate, escaping, malformed and symlinked provenance inputs', () => {
  const temporary = fixture();
  try {
    const mutate = (change, expected) => {
      const file = path.join(temporary, manifestPath);
      const manifest = JSON.parse(fs.readFileSync(file, 'utf8'));
      change(manifest);
      fs.writeFileSync(file, JSON.stringify(manifest));
      const result = validate(temporary);
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, expected);
      fs.copyFileSync(path.join(root, manifestPath), file);
    };
    mutate((value) => value.files.push({...value.files[0]}), /duplicate/);
    mutate((value) => { value.files[0].copy = '../outside.json'; }, /unsafe copied/);
    mutate((value) => { value.files[0].source = 'docs/../outside.json'; }, /unsafe source/);
    mutate((value) => { value.files[0].sha256 = 'x'.repeat(64); }, /file entry schema/);
    const copied = path.join(temporary, 'contract-artifacts/harness/analysis.schema.json');
    fs.unlinkSync(copied);
    fs.symlinkSync('/dev/null', copied);
    const result = validate(temporary);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /symlink copied/);
  } finally {
    fs.rmSync(temporary, {recursive: true, force: true});
  }
});

test('rejects a file entry that carries no usable immutable owner revision', () => {
  const temporary = fixture();
  try {
    const file = path.join(temporary, manifestPath);
    const committed = fs.readFileSync(file, 'utf8');
    const mutate = (change, expected) => {
      const manifest = JSON.parse(committed);
      change(manifest);
      fs.writeFileSync(file, JSON.stringify(manifest));
      const result = validate(temporary);
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, expected);
      fs.writeFileSync(file, committed);
    };
    mutate((value) => { delete value.files[0].commit; }, /file entry schema/);
    mutate((value) => { value.files[0].commit = 'unknown'; }, /file entry schema/);
    mutate((value) => { value.files[0].commit = value.files[0].commit.slice(0, 39); }, /file entry schema/);
    mutate((value) => { value.commit = value.files[0].commit; }, /provenance: schema\b/);
  } finally {
    fs.rmSync(temporary, {recursive: true, force: true});
  }
});

test('attributes each copy to one immutable owner revision group', () => {
  const manifest = JSON.parse(fs.readFileSync(path.join(root, manifestPath), 'utf8'));
  for (const entry of manifest.files) {
    assert.match(entry.commit, /^[0-9a-f]{40}$/);
  }
  const group = (prefix) => new Set(
    manifest.files.filter((entry) => entry.copy.startsWith(prefix)).map((entry) => entry.commit),
  );
  assert.equal(group('contract-artifacts/harness/').size, 1, 'schema copies must share one owner revision');
  assert.equal(group('fixtures/').size, 1, 'fixture copies must share one owner revision');
  const count = (prefix) => manifest.files.filter((entry) => entry.copy.startsWith(prefix)).length;
  assert.equal(count('contract-artifacts/harness/') + count('fixtures/'), manifest.files.length,
    'every copied file belongs to an attributed group');
});
