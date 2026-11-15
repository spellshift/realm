// Headless render check. Mirrors the artifact preview frame: preload React /
// ReactDOM UMD globals + bundle.css + bundle.js, then mount each component
// inside window.Realm.PreviewShell and collect console errors / uncaught throws.
//
// Usage:
//   node .ds-build/render-check.mjs Name1,Name2         (bare mount, no props)
//   node .ds-build/render-check.mjs --stories A,B,C      (load _preview/<N>.js stories)
//   node .ds-build/render-check.mjs --stories-all        (every _preview/*.js)
import { chromium } from "playwright";
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

// A real http origin so localStorage/sessionStorage (FilterContext/SortContext) work.
const server = http.createServer((_req, res) => {
  res.writeHead(200, { "content-type": "text/html" });
  res.end('<!doctype html><html data-theme="light"><head><meta charset="utf-8"></head><body><div id="root"></div></body></html>');
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const ORIGIN = "http://127.0.0.1:" + server.address().port + "/";

const here = path.dirname(fileURLToPath(import.meta.url));
const www = path.resolve(here, "..");
const out = path.join(here, "out");
const previewDir = path.join(out, "project", "_preview");

const reactUmd = path.join(www, "node_modules/react/umd/react.development.js");
const reactDomUmd = path.join(www, "node_modules/react-dom/umd/react-dom.development.js");
const bundleJs = path.join(out, "bundle.js");
const bundleCss = path.join(out, "bundle.css");

const args = process.argv.slice(2);
const useStories = args.includes("--stories") || args.includes("--stories-all");
let names = [];
if (args.includes("--stories-all")) {
  names = fs.readdirSync(previewDir).filter((f) => f.endsWith(".js")).map((f) => f.slice(0, -3));
} else {
  const list = args.find((a) => !a.startsWith("--"));
  names = list ? list.split(",") : [];
}

const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--no-sandbox", "--headless=new"] });
const results = [];

for (const name of names) {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  const errors = [];
  page.on("console", (msg) => { if (msg.type() === "error") errors.push(msg.text()); });
  page.on("pageerror", (err) => errors.push("THROW: " + (err.message || String(err))));
  try {
    await page.goto(ORIGIN);
    await page.addStyleTag({ path: bundleCss });
    await page.addScriptTag({ path: reactUmd });
    await page.addScriptTag({ path: reactDomUmd });
    await page.addScriptTag({ path: bundleJs });
    if (useStories) {
      const sp = path.join(previewDir, name + ".js");
      if (fs.existsSync(sp)) await page.addScriptTag({ path: sp });
    }
    const info = await page.evaluate((args) => {
      const { name, useStories } = args;
      const R = window.Realm;
      if (!R) return { fatal: "window.Realm missing" };
      if (!R.PreviewShell) return { fatal: "PreviewShell missing" };
      const Comp = R[name];
      if (!Comp) return { fatal: "component " + name + " not on window.Realm" };
      const h = window.React.createElement;
      let stories = [];
      if (useStories && window.__dsPreview) {
        for (const k in window.__dsPreview)
          if (typeof window.__dsPreview[k] === "function" && /^[A-Z]/.test(k)) stories.push(k);
      }
      let node;
      if (stories.length) {
        node = h(R.PreviewShell, null, stories.map((k, i) => h("div", { key: i, style: { marginBottom: 16 } }, h(window.__dsPreview[k]))));
      } else {
        node = h(R.PreviewShell, null, h(Comp));
      }
      try {
        window.ReactDOM.createRoot(document.getElementById("root")).render(node);
      } catch (e) {
        return { fatal: "render threw: " + (e && e.message || e) };
      }
      return { stories };
    }, { name, useStories });
    await page.waitForTimeout(700);
    const rootText = await page.evaluate(() => (document.getElementById("root")?.innerText || "").trim().slice(0, 60));
    const rootHtmlLen = await page.evaluate(() => (document.getElementById("root")?.innerHTML || "").length);
    results.push({ name, fatal: info.fatal || null, stories: info.stories || [], errors: errors.slice(0, 6), rootHtmlLen, rootText });
  } catch (e) {
    results.push({ name, fatal: "harness: " + (e.message || String(e)), errors });
  }
  await page.close();
}

await browser.close();
server.close();

let ok = 0, bad = 0;
for (const r of results) {
  const status = r.fatal ? "FATAL" : r.errors.length ? "ERRORS" : (r.rootHtmlLen > 30 ? "OK" : "BLANK");
  if (status === "OK") ok++; else bad++;
  console.log(`\n[${status}] ${r.name}  (html=${r.rootHtmlLen}, stories=${(r.stories||[]).join("|")||"-"})`);
  if (r.fatal) console.log("   fatal: " + r.fatal);
  for (const e of (r.errors || [])) console.log("   err: " + e.slice(0, 220));
}
console.log(`\n=== ${ok} OK / ${bad} not-ok of ${results.length} ===`);
