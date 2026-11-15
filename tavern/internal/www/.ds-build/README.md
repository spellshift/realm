# Design-system producer harness (`.ds-build`)

Rebuilds the **Realm** Claude Design-System artifact
(`claude.ai/artifact/46BX21nZGiS3LP8taciwqt`) from this package's source. It
bundles every export of `src/design-system.ts` into an IIFE that assigns
`window.Realm`, wraps each component in a `PreviewShell` carrying the real app
providers over a mocked GraphQL API, and generates the per-component preview
cards / stories / READMEs the artifact serves.

> Run it against the branch being synced (the one whose `src/design-system.ts`
> lists the components you want), e.g. `ds/add-missing-components`. It reads
> `../src/**` and `../schema.graphql` at build time.

## Run

```bash
cd tavern/internal/www
npm ci                      # or: npm install --no-save esbuild@0.25.5
node .ds-build/build.mjs    # -> .ds-build/out/bundle.{js,css}  (+ gen/list-fields.json)
node .ds-build/gen.mjs      # -> .ds-build/out/project/**       (cards, stories, READMEs)
node .ds-build/render-check.mjs --stories-all   # headless render check (no console errors)
```

Outputs land in `out/` (gitignored). Publish `out/project/**` + the bundle into
the artifact with the Artifact tool (index `project/design-system.json` last).

## Files

| File | Purpose |
|---|---|
| `build.mjs` | esbuild IIFE bundle (`window.Realm`) + Tailwind `bundle.css`; derives the `@ds-bundle` header and the list-typed-field set from `schema.graphql`. |
| `entry.tsx` | Assembles `window.Realm` = all `design-system.ts` exports + `PreviewShell` (+ `__gql`, `__Tab` preview helpers). |
| `preview-shell.tsx` | Provider stack: Chakra, MemoryRouter, Apollo(mock), Authorization/UserPreferences/Polling/Tag/Filter/Sort/Host/CreateQuestModal. |
| `mock-link.ts` | Schema-free Apollo link: resolves each query's selection set (relay connections, lists, field-name sample values). |
| `gen.mjs` | Per-component `_preview/<N>.js`, `components/<N>/{preview.html,README.md}` from a story spec + sample data; extracts `Props` for the READMEs. |
| `render-check.mjs` | Loads React/ReactDOM UMD + the bundle in headless Chromium and mounts each component in `PreviewShell`, reporting console errors. |
| `shims/` | CJS shims mapping `react`, `react-dom`, `react/jsx-runtime` to the frame's UMD globals. |

`AssetFilterBar` is in the published artifact but has no source in this package,
so it is **not** in the rebuilt bundle; its card is carried as-is and will not
mount until a source export is added to `src/design-system.ts`.
