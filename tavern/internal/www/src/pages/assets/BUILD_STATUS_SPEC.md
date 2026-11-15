# Spec: build status + ant icon on imix rows in AssetsTable

Branch: `feat/imix-agent-asset-status`

## Goal

In the Assets table, a row for a built imix agent binary (`isAgentAsset(asset.name)` in
`utils.ts`) should:

1. Show a custom **ant** icon instead of `BugAntIcon` (heroicons) as the leading glyph.
2. Show a small status marker next to that icon while the build is queued, running, or
   failed — no marker once it's built (today's icon-only look is "built").
3. On hover:
   - the **filename text** shows the existing full-name tooltip (unchanged)
   - the **copy icon** shows no tooltip
   - the **status marker** shows a tooltip with the status word ("Queued", "Building",
     …) or, for a failed build, the error message
   - all tooltips use the existing light style (`<Tooltip bg="white" color="black">`),
     already used everywhere in this file — no new tooltip component needed.

This was prototyped as a DOM overlay on top of the compiled component (a Claude artifact,
https://claude.ai/artifact/DgeVEFnz73ThhREyC58QcA) because that prototype had no access to
source. That overlay is reference only for the visual target (icon shapes, colors,
tooltip text) — none of its code should be ported. Implemented directly in
`AssetsTable.tsx`, native Chakra tooltips do all of this with no DOM hacking.

## The status data already exists — don't invent a new field

There's no `status` field on `Asset` and none is needed. Build status lives on the
`BuildTask` entity (`tavern/internal/ent/schema/buildtask.go`): `claimed_at`,
`started_at`, `finished_at`, `exit_code`, `error`, and an `artifact` edge to the `Asset`
it produces once built.

This is already queried and turned into a status today, for the "Built agents" tab:

- `tavern/internal/www/src/pages/agent-builder/queries.ts`
  - `GET_BUILD_TASKS` — fetches every `BuildTask` with `claimedAt/startedAt/finishedAt/
    exitCode/error` and `artifact { id name size hash creator { ... } }`
  - `taskStatus(task, now)` — derives one of `"Queued" | "Claimed" | "Building" |
    "Awaiting artifact" | "Built" | "Failed"` from those timestamps/exitCode (handles the
    "finished with exit 0 but no artifact yet" gap, and a stuck-extraction-failure case,
    via `AWAITING_ARTIFACT_TIMEOUT_MS`)
  - `buildTaskRows(tasks)` — maps each task to a `BuildTaskRow`, which includes
    `assetId: task.artifact?.id ?? null`
- `tavern/internal/www/src/pages/agent-builder/agents-tab/AgentsTable.tsx`
  - `STATUS_ICON: Record<BuildTaskStatus, ReactElement>` — the existing icon-per-status
    mapping (CirclePause for Queued/Claimed, a `Ring` spinner for Building/Awaiting
    artifact, CircleCheck green for Built, CircleX red for Failed)
  - Failed rows show `Build failed (exit code ${exitCode}): ${error}` as both visible
    text and a `title` tooltip.

**AssetsTable should read from this same source, not a new one**, so a row's status
always agrees with the Agents tab, and nothing has to be kept in sync by hand.

## Changes needed

### 1. Expose build-task status to AssetsTable

`GET_BUILD_TASKS`, `taskStatus`, and the `BuildTask`/`BuildTaskStatus` types currently
live under `pages/agent-builder/`, private to that feature. Move (or re-export) at least
`taskStatus`, `BuildTask`, `BuildTaskStatus` somewhere both pages can import from — e.g.
up to `utils/` — rather than importing across page boundaries or duplicating the
derivation logic.

In `Assets.tsx` (or `AssetsTable.tsx`), run `useQuery(GET_BUILD_TASKS)` and build:

```ts
const statusByAssetId = useMemo(() => {
  const map = new Map<string, { status: BuildTaskStatus; error: string | null; exitCode: number | null }>();
  for (const task of tasks) {
    if (!task.artifact) continue;
    map.set(task.artifact.id, { status: taskStatus(task), error: task.error, exitCode: task.exitCode });
  }
  return map;
}, [tasks]);
```

An asset with no entry (not produced by any `BuildTask` — a plain uploaded file, or a
legacy agent binary from before `BuildTask` existed) renders exactly as it does today: no
marker, regardless of `isAgentAsset`.

### 2. Swap the icon

Replace the `BugAntIcon` import/usage in the name column (`AssetsTable.tsx` ~line 84) with
a new local `AntIcon` — lucide has no ant glyph, so this is hand-drawn in lucide's own
style: 24×24 viewBox, `fill="none"`, `stroke="currentColor"`, `stroke-width="2"`, round
caps/joins. Suggested file: `components/icons/AntIcon.tsx`.

```tsx
export const AntIcon = ({ className }: { className?: string }) => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}
       strokeLinecap="round" strokeLinejoin="round" className={className}>
    <circle cx="12" cy="5.5" r="2" />
    <ellipse cx="12" cy="11.5" rx="1.6" ry="2" />
    <ellipse cx="12" cy="18.2" rx="2.6" ry="3.3" />
    <path d="M11 3.8 9 2M13 3.8 15 2" />
    <path d="M10.5 10 5 7.5M10.4 11.5 4 12M10.6 13 5 16.5" />
    <path d="M13.5 10 19 7.5M13.6 11.5 20 12M13.4 13 19 16.5" />
  </svg>
);
```

(Head/antennae, thorax, abdomen, six legs — same part this replaces, same `w-4 h-4`
sizing as the old `BugAntIcon`.) Update the `data-testid="agent-bug-icon"` to
`agent-ant-icon` and update any test that targets it. `AssetsHeader.tsx` also imports
`BugAntIcon` for an "agent" filter/badge button elsewhere on the page — swap that one too
so the glyph is consistent across the page (confirm with whoever owns that file;
out of scope here if it's used as a generic "agent" symbol beyond this table).

### 3. Status marker + tooltip split

Today the name column wraps the *whole* leading-icon area in one `Tooltip`, and wraps the
name span + copy icon together in a second `Tooltip`. That second one is why hovering
copy currently shows the filename tooltip — split it so only the name `<span>` is the
`Tooltip`'s child; the copy icon sits outside it, so it gets no tooltip, satisfying the
"no tooltip on copy" requirement directly, no DOM tricks required.

For the leading icon, when `isAgentAsset(asset.name)` and `statusByAssetId` has an entry
that isn't `"Built"`, render a small marker next to the ant icon:

| status | icon | color | tooltip text |
|---|---|---|---|
| Queued / Claimed | lucide `Clock` | gray-400 | "Queued" |
| Building / Awaiting artifact | lucide `Wrench` (static) **or** the `Ring` spinner already used in `AgentsTable.tsx` | gray-400 / purple | "Building" |
| Failed | lucide `CircleX` | red-600 | `` `Build failed (exit code ${exitCode}): ${error}` `` — same text `AgentsTable.tsx` already shows |

Wrap just that marker in its own `<Tooltip label={...} bg="white" color="black">`, same as
every other tooltip in this file.

**Open decision — flag, don't just pick:** the building glyph. The artifact prototype
deliberately used a static grey wrench (asked for explicitly, to avoid an animated icon in
a dense table). `AgentsTable.tsx`'s existing convention is the animated purple `Ring`
spinner. Pick one and use it in both places, or keep the inconsistency if there's a reason
rows-with-many-building-assets should look calmer than the Agents tab — this is a product
call, not an engineering one.

### 4. Row-level lock state (confirm in/out of scope)

The prototype also greyed out the row's controls, hid the links badge and expand chevron,
and blocked clicks while a row wasn't yet built — on the theory that an unfinished asset
has no links/tomes worth expanding into yet. That's a bigger change than "show a status
icon" and isn't strictly implied by this spec's three tooltip requirements. If wanted:

- `AssetsTable.tsx`'s `expandable.isExpandable` (currently `asset.links.totalCount > 0`)
  would also need `status !== undefined && status !== "Built"` to resolve to `false`.
- The actions column's Share/Delete buttons would need the same `disabled` treatment
  already used for "can't delete: has tomes" (`disabled={asset.tomes.totalCount > 0}`),
  OR'd with "not yet built."

Recommend doing this as a fast follow once the status/icon/tooltip piece lands and is
reviewed, rather than growing this change further — call it out in the PR description
either way.

## Non-goals / explicitly out of scope

- No new `Asset.status` or `Asset.buildTask` GraphQL field. `GET_BUILD_TASKS` already
  gives everything needed; adding a reverse edge so the server does this join instead of
  the client is a reasonable future optimization (today's query fetches *all* build
  tasks for every AssetsTable render), not required for this change.
- No changes to `BuildTask`, `Builder`, or the builder gRPC service.
- Nothing from the DOM-overlay prototype (shields, `MutationObserver`, manual `title`
  stripping) should be ported — it exists only to work around not having source, which
  isn't a constraint here.

## Reference

- Prototype (visual target only, not implementation): https://claude.ai/artifact/DgeVEFnz73ThhREyC58QcA
- Existing analog to mirror: `pages/agent-builder/agents-tab/AgentsTable.tsx` +
  `pages/agent-builder/queries.ts` (`taskStatus`, `buildTaskRows`, `STATUS_ICON`)
