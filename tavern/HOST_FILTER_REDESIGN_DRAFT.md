# Hosts Filter Redesign — Draft

Status: **Draft** — do not implement. Formal docs location TBD later.

## Goal
Replace the confusing multi-dropdown Filters UI with a search-bar model built from Field + Predicate + Value triples.
Start with the Hosts menu only.

Example: `Platform` + `is` / `is in` / `is not` + value.

## Non-goals (confirmed by scope acceptance)
- No changes to Quests, Tasks, Assets, or host-detail pages yet.
- No backend / GraphQL schema changes.
- No change to sort, pagination, or table layout.
- No formal `docs/_docs` site update in this stage (this Draft is the record; formalization is a later stage).

## Settled decisions
1. Scope starts with Hosts menu filters first.
2. Predicates should come from the GraphQL `WhereInput` items (e.g. `HostWhereInput`, plus related `BeaconWhereInput` where the Hosts page already filters through beacons).
3. This file is the temporary Draft record. It will be formalized into repo docs later.
4. Hosts MVP keeps full parity with everything the current dropdown supports: Platform, Transport, Status (online beacons / offline hosts / recently lost), Service tags, Group tags, Principal, Primary IP, External IP, Host, Beacon. No currently supported field is dropped.
5. Predicate set (MVP): `is`, `is not`, `is in`, `is not in` on every field, plus `contains` and `does not contain` on free-text fields (Host name, Beacon name, Principal, Primary IP, External IP). This mirrors existing `WhereInput` predicates (`*In`, `*NotIn`, `*Contains`) without exposing the full low-value surface (prefix/suffix/equal-fold).
6. Combination logic (MVP): AND across rows by default, OR inside `is in` / `is not in` value lists, negation only through predicates. No explicit OR-groups or row-level NOT toggle. Matches current `constructBeaconFilterQuery` behavior.
7. Value input UX (MVP): Platform, Transport, Status, and Tags (group/service) are suggestion-only picks from server/fixed lists — `is` takes one, `is in` / `is not in` take multiple pills. Host, Beacon, Principal, Primary IP, External IP, and names are free dynamic fields with autocomplete suggestions plus free-text entry — `is` / `is not` / `contains` / `does not contain` take one typed value, `is in` / `is not in` take multiple typed values.
8. State migration: void old stored Hosts filter data on upgrade and use only the new Field / Predicate / Value state. Keep the existing lock/persist mechanism itself unchanged.

## Current state (from repo research)
- `/hosts` renders only `FilterFieldType.BEACON_FIELDS` via `BeaconFilterBar` (`internal/www/src/context/FilterContext/FilterControls.tsx`).
- `BeaconFilterBar` is a multi-select `react-select` with groups: Platform, Transport, Status, Service, Group, Principal, PrimaryIPs, ExternalIPs, Host, Beacon (`internal/www/src/components/beacon-filter-bar/useBeaconFilterBar.ts`).
- State is `FilterContext.beaconFields: FilterBarOption[]` (`internal/www/src/context/FilterContext/FilterContext.tsx`).
- `useHostIds` builds `HostWhereInput` via `constructBeaconFilterQuery` using `hasBeaconsWith` / `hasHostWith` with `*In` predicates (`internal/www/src/pages/hosts/useHostIds.ts`, `internal/www/src/utils/constructQueryUtils.ts`).
- `HostWhereInput` supports `and` / `or` / `not` plus per-field predicates such as `nameContains`, `primaryIPContains`, `platformIn`, `platformNotIn` (`internal/www/schema.graphql`).

## Open questions
- [x] Exact field set for the Hosts MVP — settled: full parity (see decision 4).
- [x] Predicate set per field type — settled: recommended set (see decision 5).
- [x] AND/OR/NOT semantics between triples — settled: AND across rows (see decision 6).
- [x] Value input UX — settled: tags enum-like, host/beacon/principal/IPs/names free dynamic (see decision 7).
- [x] Migration of saved/locked filter state — settled: void old data, use new only (see decision 8).
- [x] Scope contract — accepted (see below).

## Scope contract — ACCEPTED
User acceptance quoted verbatim: "Yes" (chat reply, 2026-11-01 session, after "Yrd" confirm). Prior scope question proposed the text below; the user accepted it as written.

- Boundary (in scope): this Draft record; Hosts page filter path only — `FilterControls` hosts branch, `useHostIds` + `constructBeaconFilterQuery` host/beacon mapping, `FilterContext` Hosts state, and one new Hosts search-bar component (Field / Predicate / Value rows).
- Boundary (out of scope): Quests, Tasks, Assets, host-detail, and all other `FilterFieldType` pages; backend GraphQL schema changes; sort, pagination, and table-layout changes; formal `docs/_docs` site update (deferred).
- Done means: (a) Draft lists all decisions above; (b) Hosts bar supports the parity field set with the decision 5 predicates; (c) rows AND together; (d) old stored Hosts filter values are voided while lock behavior stays; (e) existing Hosts/FilterContext frontend tests pass.
- Staged designs: Quests/Tasks/Assets bars are deferred proposals. Accepting this record never approves those stages. Each returns for its own interview.

## Constraints
- Frontend-only change; must build against the existing generated `HostWhereInput` / `BeaconWhereInput` predicates.
- Must keep `FilterContext` lock/persist behavior working for the new state shape.
- Must follow repo frontend conventions (reusable components, utils for logic) and pass existing Hosts/FilterContext tests.

## Risks
- Status rows (online/offline/recently-lost) are time-derived queries, not plain field predicates — the new bar must preserve that mapping or Status will silently change meaning.
- Suggestion lists for Host/Beacon come from live queries; large fleets may need limits or debounce so the Value autocomplete stays fast.
- Voiding stored filters is a one-way reset; users with locked Hosts filters will need to rebuild them once.

## Validation
- Render the Hosts bar for each parity field and confirm the generated `where` matches today's `constructBeaconFilterQuery` output for the equivalent selection.
- Exercise `contains` on a name/principal/IP and confirm `*Contains` predicates reach the server.
- Confirm AND-across-rows with two rows and an `is not` row.
- Confirm a fresh session drops old stored values but keeps the lock toggle working.

## Unresolved items
- None for the Hosts design. Next stages (Quests/Tasks/Assets bars, formal docs-site write-up, implementation) each need their own request and interview.
