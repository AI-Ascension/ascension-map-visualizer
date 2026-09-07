const fs = require('node:fs');
const path = require('node:path');
const { spawn } = require('node:child_process');
const { pathToFileURL } = require('node:url');
const { test, expect } = require('@playwright/test');

const smallBundle = process.env.ASCENSION_MAP_SMALL_BUNDLE || '';
const denseBundle = process.env.ASCENSION_MAP_DENSE_BUNDLE || '';
const visualizerBinary = process.env.ASCENSION_MAP_VISUALIZER_BIN || '';
const screenshots = path.resolve(__dirname, '../../artifacts/browser');
const generatedInputsPresent = Boolean(smallBundle && denseBundle) && [smallBundle, denseBundle].every((directory) => fs.existsSync(path.join(directory, 'index.html')) && fs.existsSync(path.join(directory, 'viewer.json')));
const cliInputPresent = generatedInputsPresent && fs.existsSync(visualizerBinary);
const generatedSkipMessage = 'generated bundle exports are not present; set ASCENSION_MAP_SMALL_BUNDLE and ASCENSION_MAP_DENSE_BUNDLE to enable this integration check';
const cliSkipMessage = 'release CLI or generated bundle is not present; set ASCENSION_MAP_VISUALIZER_BIN, ASCENSION_MAP_SMALL_BUNDLE, and ASCENSION_MAP_DENSE_BUNDLE to enable this integration check';

function graphText(nodes, edges) {
  return `${nodes} nodes · ${edges} edges`;
}

function screenshotPath(name) {
  fs.mkdirSync(screenshots, { recursive: true });
  return path.join(screenshots, `${name}-${test.info().project.name}.png`);
}

function observePage(page) {
  const consoleErrors = [];
  const pageErrors = [];
  const requests = [];
  page.on('console', (message) => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', (error) => pageErrors.push(String(error)));
  page.on('request', (request) => requests.push(request.url()));
  return { consoleErrors, pageErrors, requests };
}

async function loadExportedArtifact(page, directory, nodes, edges, candidates) {
  const observations = observePage(page);
  await page.goto(pathToFileURL(path.join(directory, 'index.html')).href);
  await page.locator('#artifact-file').setInputFiles(path.join(directory, 'viewer.json'));
  await expect(page.locator('#graph-count')).toHaveText(graphText(nodes, edges));
  await expect(page.locator('.node-group')).toHaveCount(nodes);
  await expect(page.locator('.edge-group')).toHaveCount(edges);
  await expect(page.locator('.candidate-button')).toHaveCount(candidates);
  await expect(page.locator('.candidate-summary')).toHaveCount(candidates);
  return observations;
}

async function inspectGraph(page, edgeIndex) {
  const node = page.locator('.node-group').first();
  await node.focus();
  await node.press('Enter');
  await expect(page.locator('#inspector-heading')).toHaveText('Node inspection');
  await page.keyboard.press('ArrowRight');
  expect(await page.evaluate(() => window.__ASCENSION_MAP_VIEWER__.getState().selected.kind)).toBe('node');
  const edge = page.locator('.edge-group').nth(edgeIndex);
  await edge.press('Enter');
  await expect(page.locator('#inspector-heading')).toHaveText('Edge inspection');
  await expect(page.locator('#inspector-body')).toContainText('Directed connection');
  await expect(page.locator('button', { hasText: /dispatch|act|travel/i })).toHaveCount(0);
}

function assertNoHttpRequests(observations) {
  expect(observations.consoleErrors).toEqual([]);
  expect(observations.pageErrors).toEqual([]);
  expect(observations.requests.filter((url) => /^https?:/u.test(url))).toEqual([]);
}

test.describe.configure({ mode: 'serial' });

test.describe('actual CLI-generated bundle exports', () => {
  if (process.env.CI && !generatedInputsPresent) {
    test('requires fresh generated bundle exports in CI', () => {
      throw new Error(generatedSkipMessage);
    });
  } else {
    test.skip(!generatedInputsPresent, generatedSkipMessage);

    test('opens both exported index files through file mode and inspects their complete graphs', async ({ page }) => {
    const small = await loadExportedArtifact(page, smallBundle, 4, 4, 2);
    await expect(page.locator('#connection-badge')).toHaveText('Historical');
    await expect(page.locator('#map-status-badge')).toHaveText('HISTORICAL · READ ONLY');
    await expect(page.locator('.candidate-summary').first()).toContainText('3 nodes; rest');
    await page.locator('.candidate-button').last().click();
    await expect(page.locator('.candidate-button').last()).toHaveAttribute('aria-pressed', 'true');
    await inspectGraph(page, 1);
    await expect(page.locator('#inspector-body')).toContainText('Recorded source status: AVAILABLE / COMPLETE / CURRENT');
    assertNoHttpRequests(small);

    const dense = await loadExportedArtifact(page, denseBundle, 76, 182, 8);
    await expect(page.locator('#connection-badge')).toHaveText('Historical');
    await expect(page.locator('#map-status-badge')).toHaveText('HISTORICAL · READ ONLY');
    await expect(page.locator('.candidate-summary').first()).toContainText('13 nodes');
    await page.locator('.candidate-button').nth(7).click();
    await expect(page.locator('.candidate-button').nth(7)).toHaveAttribute('aria-pressed', 'true');
    await inspectGraph(page, 91);
    await expect(page.locator('#inspector-body')).toContainText('Recorded source status: AVAILABLE / COMPLETE / CURRENT');
    await page.screenshot({ path: screenshotPath('map-generated-dense-file'), fullPage: true });
    assertNoHttpRequests({
      consoleErrors: [...small.consoleErrors, ...dense.consoleErrors],
      pageErrors: [...small.pageErrors, ...dense.pageErrors],
      requests: [...small.requests, ...dense.requests],
    });
    });
  }
});

function startCliServer() {
  return new Promise((resolve, reject) => {
    const child = spawn(visualizerBinary, ['serve', '--bundle', denseBundle, '--port', '0'], {
      cwd: path.resolve(__dirname, '../..'),
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let output = '';
    let settled = false;
    const timer = setTimeout(() => {
      if (settled) return;
      settled = true;
      child.kill('SIGTERM');
      reject(new Error(`CLI serve did not announce a loopback URL: ${output}`));
    }, 15_000);
    const read = (chunk) => {
      output += chunk.toString();
      const match = output.match(/http:\/\/127\.0\.0\.1:(\d+)\//u);
      if (!match || settled) return;
      settled = true;
      clearTimeout(timer);
      resolve({ child, url: `http://127.0.0.1:${match[1]}` });
    };
    child.stdout.on('data', read);
    child.stderr.on('data', read);
    child.once('error', (error) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      reject(error);
    });
    child.once('exit', (code, signal) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      reject(new Error(`CLI serve exited before announcing a URL (${code || signal}): ${output}`));
    });
  });
}

let cliServer;
test.describe('actual release CLI serve', () => {
  if (process.env.CI && !cliInputPresent) {
    test('requires the release CLI and fresh generated bundles in CI', () => {
      throw new Error(cliSkipMessage);
    });
  } else {
    test.skip(!cliInputPresent, cliSkipMessage);

    test.beforeAll(async () => {
      cliServer = await startCliServer();
    });

    test.afterAll(async () => {
      if (!cliServer) return;
      if (cliServer.child.exitCode === null) {
        cliServer.child.kill('SIGTERM');
        await new Promise((resolve) => cliServer.child.once('exit', resolve));
      }
      cliServer = null;
    });

    test('serves the dense graph, exposes replay, and marks the frame historical', async ({ page }) => {
    const observations = observePage(page);
    await page.goto(`${cliServer.url}/index.html`);
    await expect(page.locator('#graph-count')).toHaveText(graphText(76, 182));
    await expect(page.locator('.node-group')).toHaveCount(76);
    await expect(page.locator('.edge-group')).toHaveCount(182);
    await expect(page.locator('#connection-badge')).toHaveText('Historical');
    await expect(page.locator('#map-status-badge')).toHaveText('HISTORICAL · READ ONLY');
    await expect(page.locator('.candidate-button')).toHaveCount(8);
    await expect(page.locator('.candidate-summary').first()).toContainText('13 nodes');
    await inspectGraph(page, 91);
    await expect(page.locator('#inspector-body')).toContainText('Recorded source status: AVAILABLE / COMPLETE / CURRENT');

    await expect(page.locator('#replay-select option')).toHaveCount(1);
    await expect(page.locator('#replay-count')).toHaveText('1 frame');
    await page.locator('#replay-select').selectOption('0');
    await expect(page.locator('#connection-badge')).toHaveText('Historical');
    await expect(page.locator('#map-status-badge')).toHaveText('HISTORICAL · READ ONLY');
    await expect(page.locator('#replay-note')).toContainText('Historical frame 1');
    await page.screenshot({ path: screenshotPath('map-generated-dense-live'), fullPage: true });

    const bundleId = JSON.parse(fs.readFileSync(path.join(denseBundle, 'manifest.json'), 'utf8')).bundle_digest;
    const allowedPaths = new Set(['/index.html', '/style.css', '/app.js', '/api/current', '/api/replay', `/api/frame/${bundleId}`]);
    const httpPaths = observations.requests.filter((url) => /^https?:/u.test(url)).map((url) => new URL(url).pathname);
    expect(httpPaths.length).toBeGreaterThanOrEqual(3);
    expect(httpPaths.every((requestPath) => allowedPaths.has(requestPath))).toBeTruthy();
    expect(httpPaths).toContain('/api/current');
    expect(httpPaths).toContain('/api/replay');
    expect(httpPaths).toContain(`/api/frame/${bundleId}`);
    expect(observations.consoleErrors).toEqual([]);
    expect(observations.pageErrors).toEqual([]);
    });
  }
});
