const { test, expect, devices } = require("@playwright/test");

const baseURL = process.env.TALED_PERF_BASE_URL || "http://127.0.0.1:8155";
const sampleTitle = process.env.TALED_PERF_SAMPLE || "Theater";
const openLongTaskBudgetMs = Number(process.env.TALED_PERF_OPEN_BUDGET_MS || 800);
const panLongTaskBudgetMs = Number(process.env.TALED_PERF_PAN_BUDGET_MS || 120);
const paintLongTaskBudgetMs = Number(process.env.TALED_PERF_PAINT_BUDGET_MS || 120);

test.use({
  ...devices["Pixel 5"],
  browserName: "chromium",
  launchOptions: {
    executablePath: "/usr/bin/chromium",
    args: ["--disable-gpu"],
  },
});

test("mobile sample perf", async ({ page }) => {
  const consoleMessages = [];
  page.on("console", (message) => {
    const text = message.text();
    if (text.includes("perf:") || text.includes("boot:")) {
      consoleMessages.push(text);
    }
  });

  await page.addInitScript(() => {
    window.__taledPerf = {
      longTasks: [],
      rafDeltas: [],
      phase: "boot",
      lastRaf: 0,
    };

    if ("PerformanceObserver" in window) {
      try {
        const observer = new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            window.__taledPerf.longTasks.push({
              phase: window.__taledPerf.phase,
              duration: entry.duration,
            });
          }
        });
        observer.observe({ entryTypes: ["longtask"] });
      } catch (_) {}
    }

    const tick = (timestamp) => {
      if (window.__taledPerf.lastRaf !== 0) {
        window.__taledPerf.rafDeltas.push({
          phase: window.__taledPerf.phase,
          delta: timestamp - window.__taledPerf.lastRaf,
        });
      }
      window.__taledPerf.lastRaf = timestamp;
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });

  await page.goto(baseURL, { waitUntil: "networkidle" });
  await page.getByText(sampleTitle, { exact: true }).click();
  await page.locator(".review-editor-page").waitFor();
  await page.waitForTimeout(1200);

  const openMetrics = await page.evaluate(() => {
    const perf = window.__taledPerf;
    const longTasks = perf.longTasks.filter((entry) => entry.phase === "boot");
    perf.phase = "pan";
    return {
      longTaskMax: longTasks.reduce(
        (max, entry) => Math.max(max, entry.duration),
        0,
      ),
      nodeCount: document.querySelectorAll("*").length,
    };
  });

  const canvas = page.locator(".canvas-stage");
  const box = await canvas.boundingBox();
  expect(box).not.toBeNull();
  const startX = box.x + box.width * 0.55;
  const startY = box.y + box.height * 0.55;
  await page.mouse.move(startX, startY);
  await page.mouse.down();
  await page.mouse.move(startX - 96, startY - 32, { steps: 10 });
  await page.mouse.up();
  await page.waitForTimeout(400);

  const panMetrics = await page.evaluate(() => {
    const perf = window.__taledPerf;
    const longTasks = perf.longTasks.filter((entry) => entry.phase === "pan");
    perf.phase = "paint";
    return {
      longTaskMax: longTasks.reduce(
        (max, entry) => Math.max(max, entry.duration),
        0,
      ),
      worstRafDelta: perf.rafDeltas
        .filter((entry) => entry.phase === "pan")
        .reduce((max, entry) => Math.max(max, entry.delta), 0),
    };
  });

  await page.mouse.move(startX, startY);
  await page.mouse.down();
  await page.mouse.up();
  await page.waitForTimeout(400);

  const paintMetrics = await page.evaluate(() => {
    const perf = window.__taledPerf;
    const longTasks = perf.longTasks.filter((entry) => entry.phase === "paint");
    return {
      longTaskMax: longTasks.reduce(
        (max, entry) => Math.max(max, entry.duration),
        0,
      ),
      worstRafDelta: perf.rafDeltas
        .filter((entry) => entry.phase === "paint")
        .reduce((max, entry) => Math.max(max, entry.delta), 0),
    };
  });

  console.log(
    JSON.stringify(
      {
        sampleTitle,
        open: openMetrics,
        pan: panMetrics,
        paint: paintMetrics,
        consoleMessages,
      },
      null,
      2,
    ),
  );

  expect(openMetrics.longTaskMax).toBeLessThan(openLongTaskBudgetMs);
  expect(panMetrics.longTaskMax).toBeLessThan(panLongTaskBudgetMs);
  expect(paintMetrics.longTaskMax).toBeLessThan(paintLongTaskBudgetMs);
});
