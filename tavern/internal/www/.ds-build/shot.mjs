// Screenshot a component's preview (loads bundle.css + UMD React + bundle.js +
// its _preview story, mounts inside PreviewShell) to diagnose backgrounds.
import { chromium } from "playwright";
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const www = path.resolve(here, "..");
const out = path.join(here, "out");
const names = (process.argv[2] || "AssetsTable").split(",");

const server = http.createServer((_req, res) => {
  res.writeHead(200, { "content-type": "text/html" });
  res.end('<!doctype html><html data-theme="light"><head><meta charset="utf-8"></head><body><div id="root"></div></body></html>');
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const ORIGIN = "http://127.0.0.1:" + server.address().port + "/";

const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--no-sandbox", "--headless=new"] });
for (const name of names) {
  const page = await browser.newPage({ viewport: { width: 1200, height: 700 } });
  await page.goto(ORIGIN);
  await page.addStyleTag({ path: path.join(out, "bundle.css") });
  await page.addScriptTag({ path: path.join(www, "node_modules/react/umd/react.development.js") });
  await page.addScriptTag({ path: path.join(www, "node_modules/react-dom/umd/react-dom.development.js") });
  await page.addScriptTag({ path: path.join(out, "bundle.js") });
  await page.addScriptTag({ path: path.join(out, "project/_preview", name + ".js") });
  await page.evaluate((n) => {
    const h = window.React.createElement, R = window.Realm;
    const keys = Object.keys(window.__dsPreview || {}).filter((k) => /^[A-Z]/.test(k));
    window.ReactDOM.createRoot(document.getElementById("root")).render(
      h(R.PreviewShell, null, keys.map((k, i) => h("div", { key: i }, h(window.__dsPreview[k]))))
    );
  }, name);
  await page.waitForTimeout(800);
  const bg = await page.evaluate(() => {
    const root = document.querySelector(".ds-preview-root");
    return { bodyBg: getComputedStyle(document.body).backgroundColor, rootBg: root ? getComputedStyle(root).backgroundColor : "none" };
  });
  const file = path.join(out, "shot-" + name + ".png");
  await page.screenshot({ path: file });
  console.log(name, "bodyBg=" + bg.bodyBg, "rootBg=" + bg.rootBg, "->", file);
  await page.close();
}
await browser.close();
server.close();
