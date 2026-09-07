const fs = require('node:fs');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const { test, expect } = require('@playwright/test');

const fixture = path.resolve(__dirname, 'fixtures/example-viewer.json');
const hostileFixture = path.resolve(__dirname, 'fixtures/hostile-viewer.json');
const indexUrl = '/index.html';
const offlineUrl = pathToFileURL(path.resolve(__dirname, '../../web/index.html')).href;

async function loadOffline(page, file = fixture, expectedGraph = '8 nodes · 9 edges') {
  await page.goto(offlineUrl);
  await page.locator('#artifact-file').setInputFiles(file);
  await expect(page.locator('#graph-count')).toHaveText(expectedGraph);
}

test.describe('offline viewer', () => {
  test('opens viewer.json, fits, zooms, pans, searches, and inspects without actions', async ({ page }) => {
    const consoleErrors = [];
    page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
    await loadOffline(page);

    await expect(page.locator('#connection-badge')).toHaveText('Offline');
    await expect(page.locator('.node-group')).toHaveCount(8);
    await expect(page.locator('.edge-group')).toHaveCount(9);
    await expect(page.locator('.node-group').first()).toHaveAttribute('role', 'button');

    await page.locator('[data-action="zoom-in"]').click();
    await expect(page.locator('#zoom-readout')).toHaveText('122%');
    await page.locator('[data-action="fit"]').click();
    await expect(page.locator('#zoom-readout')).toHaveText('100%');
    await page.locator('#map-stage').hover();
    await page.mouse.wheel(0, -500);
    await expect(page.locator('#zoom-readout')).not.toHaveText('100%');

    await page.locator('#node-search').fill('N0007');
    await expect(page.locator('.search-result')).toHaveCount(1);
    await page.locator('.search-result').click();
    await expect(page.locator('#inspector-heading')).toHaveText('Node inspection');
    await expect(page.locator('#inspector-body')).toContainText('late');

    await page.locator('.edge-group').nth(1).click();
    await expect(page.locator('#inspector-heading')).toHaveText('Edge inspection');
    await expect(page.locator('#inspector-body')).toContainText('Directed connection');
    await expect(page.locator('button', { hasText: /dispatch|act|travel/i })).toHaveCount(0);

    const before = await page.locator('.node-group').count();
    await page.locator('[data-overlay="reachable"]').click();
    await expect(page.locator('.node-group')).toHaveCount(before);
    await expect(page.locator('[data-overlay="reachable"]')).toHaveAttribute('aria-pressed', 'false');
    expect(consoleErrors).toEqual([]);
  });

  test('renders hostile labels as text and does not execute markup', async ({ page }) => {
    let dialogs = 0;
    page.on('dialog', dialog => { dialogs += 1; dialog.dismiss(); });
    await loadOffline(page, hostileFixture, '1 nodes · 0 edges');
    await expect(page.locator('.node-label')).toHaveText('<script>alert(1)</script>');
    await expect(page.locator('.empty-state')).toBeHidden();
    await expect(page.locator('script')).toHaveCount(1);
    expect(dialogs).toBe(0);
  });

  test('loads the bundled example with the same validated graph surface', async ({ page }) => {
    await page.goto(indexUrl);
    await page.locator('[data-action="example"]').first().click();
    await expect(page.locator('#bundle-label')).toContainText('EXAMPLE');
    await expect(page.locator('.node-group')).toHaveCount(21);
    await expect(page.locator('.edge-group')).toHaveCount(32);
    await expect(page.locator('#overview-svg')).toBeVisible();
  });
});

test.describe('live and replay adapters', () => {
  test('uses only fixed routes, reconnects atomically, and marks historical replay', async ({ page }) => {
    const requests = [];
    const currentPayload = JSON.parse(fs.readFileSync(fixture, 'utf8'));
    const replayPayload = { ...currentPayload, bundle_id: 'fixture-replay-001', historical: true, map: { ...currentPayload.map, identity: 'fixture-map-historical' } };
    await page.route('**/api/current', async route => {
      requests.push(route.request().url());
      await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(currentPayload) });
    });
    await page.route('**/api/replay', async route => {
      requests.push(route.request().url());
      await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ frames: [{ bundle_id: 'fixture-replay-001', viewer: 'frames/one/viewer.json' }] }) });
    });
    await page.route('**/api/frame/fixture-replay-001', async route => {
      requests.push(route.request().url());
      await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(replayPayload) });
    });
    await page.goto(indexUrl);
    await expect(page.locator('#connection-badge')).toHaveText('Live');
    await expect(page.locator('#graph-count')).toHaveText('8 nodes · 9 edges');
    await expect(page.locator('#replay-select')).toHaveValue('');
    await page.locator('#replay-select').selectOption('0');
    await expect(page.locator('#connection-badge')).toHaveText('Historical');
    await expect(page.locator('#map-status-badge')).toContainText('HISTORICAL');
    await expect(page.locator('#replay-note')).toContainText('Historical frame');
    expect(requests.every(url => /\/api\/(current|replay|frame\/fixture-replay-001)$/.test(new URL(url).pathname))).toBeTruthy();
  });

  test('keeps the previous complete view after a failed current poll', async ({ page }) => {
    let calls = 0;
    const currentPayload = fs.readFileSync(fixture, 'utf8');
    await page.route('**/api/current', async route => {
      calls += 1;
      if (calls === 1) await route.fulfill({ status: 200, contentType: 'application/json', body: currentPayload });
      else await route.fulfill({ status: 503, body: 'offline' });
    });
    await page.route('**/api/replay', route => route.fulfill({ status: 404, body: 'none' }));
    await page.goto(indexUrl);
    await expect(page.locator('#graph-count')).toHaveText('8 nodes · 9 edges');
    await page.evaluate(() => window.__ASCENSION_MAP_VIEWER__.getState().live.connected = false);
    await page.evaluate(() => window.__ASCENSION_MAP_VIEWER__.getState().live.failureCount += 1);
    await page.waitForTimeout(50);
    await expect(page.locator('#graph-count')).toHaveText('8 nodes · 9 edges');
  });
});

test('works in the mobile layout and keeps controls reachable', async ({ page }) => {
  await loadOffline(page);
  await expect(page.locator('.map-stage')).toBeVisible();
  await expect(page.locator('.overview-panel')).toBeVisible();
  await page.locator('.node-group').first().focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('#inspector-heading')).toHaveText('Node inspection');
});
