// Build the Realm design-system bundle (IIFE) + stylesheet from the www package.
// Usage: node .ds-build/build.mjs
import * as esbuild from "esbuild";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const www = path.resolve(here, "..");
const out = path.join(here, "out");
fs.mkdirSync(out, { recursive: true });

// --- component export order from design-system.ts (for the @ds-bundle header)
const dsSrc = fs.readFileSync(path.join(www, "src/design-system.ts"), "utf8");
const names = [];
for (const m of dsSrc.matchAll(/^export \{\s*([A-Za-z0-9_]+)\s*\}/gm)) names.push(m[1]);
const header =
  "/* @ds-bundle: " +
  JSON.stringify({ format: 4, namespace: "Realm", components: names.map((n) => ({ name: n })) }) +
  " */\n";

// --- list-typed field names from schema.graphql (so the mock returns arrays
// for GraphQL lists, e.g. metrics.beaconTimelineChart, not single objects).
const schema = fs.readFileSync(path.join(www, "schema.graphql"), "utf8");
const listFields = new Set();
for (const m of schema.matchAll(/^\s*([A-Za-z_]\w*)\s*(?:\([^)]*\))?\s*:\s*\[/gm)) listFields.add(m[1]);
fs.mkdirSync(path.join(here, "gen"), { recursive: true });
fs.writeFileSync(path.join(here, "gen", "list-fields.json"), JSON.stringify([...listFields]));

const shim = (p) => path.join(here, "shims", p);
const DATAURL = {};
for (const e of ["png", "jpg", "jpeg", "gif", "svg", "webp", "woff", "woff2", "ttf", "eot", "otf"]) DATAURL["." + e] = "dataurl";

const result = await esbuild.build({
  entryPoints: [path.join(here, "entry.tsx")],
  bundle: true,
  format: "iife",
  minify: true,
  sourcemap: false,
  treeShaking: true,
  jsx: "automatic",
  target: "es2020",
  legalComments: "none",
  logLevel: "silent",
  metafile: true,
  outfile: path.join(out, "entry.js"),
  loader: { ".js": "jsx", ...DATAURL },
  resolveExtensions: [".tsx", ".ts", ".jsx", ".js", ".json", ".mjs"],
  alias: {
    react: shim("react.js"),
    "react-dom": shim("react-dom.js"),
    "react-dom/client": shim("react-dom-client.js"),
    "react/jsx-runtime": shim("jsx-runtime.js"),
    "react/jsx-dev-runtime": shim("jsx-runtime.js"),
  },
  define: {
    "process.env.NODE_ENV": '"production"',
    "process.env.REACT_APP_API_ENDPOINT": '"https://realm.preview"',
    "process.env.PUBLIC_URL": '""',
    global: "globalThis",
  },
}).catch((e) => {
  console.error("ESBUILD_FAILED");
  if (e.errors) for (const err of e.errors.slice(0, 25)) {
    const loc = err.location ? `${err.location.file}:${err.location.line}` : "";
    console.error("  - " + err.text + (loc ? "  [" + loc + "]" : ""));
  }
  process.exit(1);
});

// --- post-process -> bundle.js: escape html-sensitive sequences, prepend header
let js = fs.readFileSync(path.join(out, "entry.js"), "utf8");
js = js.split("</script").join("<\\/script").split("<!--").join("<\\!--");
fs.writeFileSync(path.join(out, "bundle.js"), header + js);
fs.rmSync(path.join(out, "entry.js"), { force: true });

// --- tailwind utilities + custom css, then append any esbuild-collected css
const tw = path.join(out, "tw.css");
execFileSync(
  path.join(www, "node_modules/.bin/tailwindcss"),
  ["-c", path.join(www, "tailwind.config.js"), "-i", path.join(www, "src/style.css"), "-o", tw, "--minify"],
  { cwd: www, stdio: ["ignore", "ignore", "inherit"] }
);
let css = fs.readFileSync(tw, "utf8");
const esbuildCss = path.join(out, "entry.css");
if (fs.existsSync(esbuildCss)) { css += "\n" + fs.readFileSync(esbuildCss, "utf8"); fs.rmSync(esbuildCss, { force: true }); }
css = css.split("</style").join("<\\/style");
fs.writeFileSync(path.join(out, "bundle.css"), css);

const kb = (p) => (fs.statSync(p).size / 1024).toFixed(0);
console.log("OK bundle.js=" + kb(path.join(out, "bundle.js")) + "KB bundle.css=" + kb(path.join(out, "bundle.css")) + "KB components=" + names.length);
