#!/usr/bin/env node

const { chromium } = require('playwright-core');

const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  args.set(process.argv[index], process.argv[index + 1]);
}

const baseUrl = args.get('--base-url');
const screen = args.get('--screen');
const outPath = args.get('--out');

if (!baseUrl || !screen || !outPath) {
  console.error('usage: node ui_capture_review.js --base-url URL --screen SCREEN --out FILE');
  process.exit(2);
}

const readySelectors = {
  dashboard: '.review-project-list-panel',
  editor: '.review-editor-canvas',
  tilesets: '.review-tileset-sheet',
  layers: '.review-layer-row',
  objects: '.review-object-grid',
  settings: '.review-settings-card',
};

const readySelector = readySelectors[screen];
if (!readySelector) {
  console.error(`unknown screen: ${screen}`);
  process.exit(2);
}

(async () => {
  const browser = await chromium.launch({
    executablePath: '/usr/bin/chromium',
    headless: true,
    args: ['--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage'],
  });

  const context = await browser.newContext({
    viewport: { width: 384, height: 688 },
    screen: { width: 384, height: 688 },
    deviceScaleFactor: 2,
    isMobile: true,
    hasTouch: true,
    userAgent:
      'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Mobile Safari/537.36',
  });
  const page = await context.newPage();

  page.on('console', (msg) => console.log(`[console:${screen}] ${msg.type()} ${msg.text()}`));
  page.on('pageerror', (err) => console.error(`[pageerror:${screen}] ${err.message}`));
  page.on('requestfailed', (req) => {
    console.error(`[requestfailed:${screen}] ${req.url()} ${req.failure()?.errorText ?? ''}`);
  });

  const url = `${baseUrl}/?screen=${screen}`;
  await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30000 });
  await page.waitForSelector(readySelector, { timeout: 30000 });
  await page.waitForFunction(
    () => Array.from(document.images).every((img) => img.complete && img.naturalWidth > 0),
    { timeout: 30000 }
  );
  await page.waitForTimeout(900);
  await page.screenshot({ path: outPath, fullPage: true });
  await context.close();
  await browser.close();
})();
