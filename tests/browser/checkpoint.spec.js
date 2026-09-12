// SPDX-License-Identifier: MIT
const fs = require('node:fs');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const { test, expect } = require('@playwright/test');

function payload() {
  const value = JSON.parse(fs.readFileSync(path.join(__dirname, 'fixtures/example-viewer.json'), 'utf8'));
  value.historical = true;
  value.schema = 'ascension-map-viewer-v2';
  value.checkpoint = { reference: {
    schema: 'ascension.exact_checkpoint_reference.v1', reference_version: 'exact-checkpoint-reference-v1',
    handle: 'ckpt-h1:' + 'a'.repeat(64), occurrence: 'occurrence:test', boundary_kind: 'stable_decision',
    boundary_phase: 'map', assurance: 'restore_verified', restore_verified: true
  }, run_id: 'run:test', episode_id: 'episode:test', trajectory_id: 'trajectory:test', dispatchable: false };
  return value;
}

async function load(page, value) {
  await page.locator('#artifact-file').setInputFiles({ name: 'viewer.json', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify(value)) });
}

test('checkpoint evidence displays public lineage and remains historical and read only', async ({ page }) => {
  await page.goto(pathToFileURL(path.resolve(__dirname, '../../web/index.html')).href);
  const value = payload();
  value.checkpoint.reference.boundary_phase = '<img src=x onerror=alert(1)>';
  await load(page, value);
  const evidence = page.locator('#checkpoint-evidence');
  await expect(evidence).toBeVisible();
  for (const text of [value.checkpoint.reference.handle, 'occurrence:test', 'run:test', 'episode:test', 'trajectory:test', 'Producer assurance: restore verified', 'no current action or restore authority', '<img src=x onerror=alert(1)>']) {
    await expect(evidence).toContainText(text);
  }
  await expect(evidence.locator('img, button, a')).toHaveCount(0);
  await expect(page.locator('#map-status-badge')).toContainText('HISTORICAL');
  const legacy = payload();
  delete legacy.checkpoint;
  legacy.schema = 'ascension-map-viewer-v1';
  await load(page, legacy);
  await expect(evidence).toBeHidden();
});

test('browser rejects checkpoint escalation, unknown fields and versions before replacing the view', async ({ page }) => {
  await page.goto(pathToFileURL(path.resolve(__dirname, '../../web/index.html')).href);
  await load(page, payload());
  const mutations = [
    value => { value.checkpoint.reference.reference_version = 'future'; },
    value => { value.checkpoint.reference.exact_state_id = 'private'; },
    value => { value.checkpoint.reference.blob_digest = 'private'; },
    value => { value.checkpoint.reference.restore_verified = false; },
    value => { value.checkpoint.dispatchable = true; }
  ];
  for (const mutate of mutations) {
    const value = payload();
    mutate(value);
    await load(page, value);
    await expect(page.locator('#notice-region')).toContainText(/unsupported|inconsistent|read only/);
    await expect(page.locator('#checkpoint-evidence')).toContainText('occurrence:test');
    await expect(page.locator('.node-group')).toHaveCount(8);
  }
});
