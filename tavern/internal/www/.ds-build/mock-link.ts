// A schema-free mock Apollo link. It walks each operation's OWN selection set
// and returns structurally-valid data: relay connections (detected by an
// `edges`/`pageInfo` subfield) get sample edges, objects recurse, and scalar
// leaves get field-name-based sample values. No GraphQL schema is bundled.
import { ApolloClient, ApolloLink, InMemoryCache, Observable } from "@apollo/client";
import LIST_FIELD_NAMES from "./gen/list-fields.json";

type AnyObj = Record<string, any>;
const LIST_FIELDS = new Set<string>(LIST_FIELD_NAMES as string[]);

const PLATFORMS = ["Linux", "Windows", "Darwin"];
const ASSET_NAMES = [
  "imix_linux_amd64", "imix_windows_amd64.exe", "imix_darwin_arm64",
  "beacon_config.json", "privesc_linux.sh", "recon_bundle.tar.gz",
];
const HOST_NAMES = ["DESKTOP-7F3A2C", "web-prod-01", "dc01.corp.local", "bastion-eu"];
const TOME_NAMES = ["Enumerate Users", "Portscan Subnet", "Dump Credentials", "Install Persistence"];
const USER_NAMES = ["alice", "bob", "root", "operator"];
const QUEST_NAMES = ["Initial Access", "Lateral Movement", "Exfil Prep"];
const TAG_NAMES = ["production", "linux", "domain-controller", "exfil"];

let seq = 0;
function pick<T>(arr: T[], i: number): T { return arr[i % arr.length]; }
function hex(n: number): string {
  let s = ""; const c = "0123456789abcdef";
  for (let i = 0; i < n; i++) s += c[(i * 7 + 3) % 16];
  return s;
}
function iso(offsetMs: number): string { return new Date(Date.now() + offsetMs).toISOString(); }

// Produce a scalar/leaf value from the field name and context.
function leaf(name: string, ctx: Ctx): any {
  const k = name.toLowerCase();
  const idx = ctx.index || 0;
  if (k === "id") return (ctx.typename || "node") + "-" + idx;
  if (k === "__typename") return ctx.typename || "Mock";
  if (k === "totalcount" || k === "count") return ctx.count != null ? ctx.count : 3;
  if (/(^|[^a-z])id$/.test(k) || k.endsWith("id")) return "ref-" + (++seq);
  if (k === "hash" || k === "sha256" || k === "digest") return hex(64);
  if (k === "size" || k === "bytes" || k === "filesize") return 1048576 + idx * 4096;
  if (k === "downloads") return 2 + idx;
  if (k === "downloadlimit") return 25;
  if (k === "expiresat") return iso(1000 * 60 * 60 * 24 * 7);
  if (k.endsWith("time") && k !== "timestamp") return Math.floor((Date.now() - (idx + 1) * 60000) / 1000); // unix seconds
  if (k.endsWith("at") || k === "time" || k === "timestamp" || k.includes("createdat") || k.includes("modified"))
    return iso(-1000 * 60 * (idx + 1));
  if (k === "photourl" || k === "avatarurl" || k === "imageurl") return "";
  if (k === "path") return ctx.typename === "Link" ? "/l/" + hex(8) : "/usr/local/bin/sample";
  if (k === "platform" || k === "os") return pick(PLATFORMS, idx);
  if (k === "primaryip" || k === "externalip" || k === "ip" || k === "address") return "10.0.0." + (10 + idx);
  if (k === "principal" || k === "username" || k === "user" || k === "owner") return pick(USER_NAMES, idx);
  if (k === "kind") return idx % 2 ? "service" : "group";
  if (k === "color") return pick(["#6b7280", "#7c3aed", "#059669", "#dc2626"], idx);
  if (k === "status" || k === "state") return "ACTIVE";
  if (k === "isadmin" || k === "isactivated" || k.startsWith("is") || k.startsWith("has") || k.startsWith("can"))
    return true;
  if (k === "closedatisnil") return false;
  if (k === "output" || k === "content" || k === "text" || k === "eldritch" || k === "description" || k === "stderr" || k === "stdout")
    return "Sample output for preview.";
  if (k === "port" || k === "pid" || k === "ppid" || k === "uid" || k === "gid" || k === "index" || k === "num" || k.startsWith("num"))
    return 1000 + idx;
  // typename-specific display names
  if (k === "name") {
    switch (ctx.typename) {
      case "Asset": return pick(ASSET_NAMES, idx);
      case "Host": return pick(HOST_NAMES, idx);
      case "Tome": return pick(TOME_NAMES, idx);
      case "User": return pick(USER_NAMES, idx);
      case "Quest": return pick(QUEST_NAMES, idx);
      case "Tag": return pick(TAG_NAMES, idx);
      default: return (ctx.typename || "Item") + " " + (idx + 1);
    }
  }
  if (k === "value" || k === "label") return "Sample " + name;
  return "Sample " + name;
}

type Ctx = { typename?: string; index?: number; count?: number };

function cap(s: string): string { return s ? s[0].toUpperCase() + s.slice(1) : s; }
function singular(s: string): string {
  if (s.endsWith("ies")) return s.slice(0, -3) + "y";
  if (s.endsWith("sses")) return s.slice(0, -2);
  if (s.endsWith("s")) return s.slice(0, -1);
  return s;
}
// Guess the node typename from a connection/list field name.
function nodeTypename(fieldName: string): string {
  const map: AnyObj = {
    hosts: "Host", beacons: "Beacon", tasks: "Task", quests: "Quest", tomes: "Tome",
    assets: "Asset", links: "Link", tags: "Tag", files: "HostFile", processes: "Process",
    credentials: "Credential", shells: "Shell", portals: "Portal", users: "User",
    repositories: "Repository", screenshots: "HostFile", builders: "Builder",
    buildtasks: "BuildTask", activeusers: "User",
  };
  return map[fieldName.toLowerCase()] || cap(singular(fieldName));
}

function collectFields(selSet: any, fragments: AnyObj, out: any[]) {
  for (const sel of selSet.selections) {
    if (sel.kind === "Field") out.push(sel);
    else if (sel.kind === "InlineFragment") collectFields(sel.selectionSet, fragments, out);
    else if (sel.kind === "FragmentSpread") {
      const frag = fragments[sel.name.value];
      if (frag) collectFields(frag.selectionSet, fragments, out);
    }
  }
}

function argValue(field: any, name: string): any {
  if (!field.arguments) return undefined;
  for (const a of field.arguments) {
    if (a.name.value === name) {
      const v = a.value;
      if (v.kind === "IntValue") return parseInt(v.value, 10);
      if (v.kind === "ObjectValue") return {};
      return v.value;
    }
  }
  return undefined;
}

function isConnection(field: any): boolean {
  if (!field.selectionSet) return false;
  return field.selectionSet.selections.some(
    (s: any) => s.kind === "Field" && (s.name.value === "edges" || s.name.value === "pageInfo")
  );
}

function edgeCount(field: any): number {
  const first = argValue(field, "first");
  if (typeof first === "number") return Math.min(first, 4);
  const last = argValue(field, "last");
  if (typeof last === "number") return Math.min(last, 4);
  if (argValue(field, "where") !== undefined) return 1; // filtered -> single match
  return 3;
}

function resolveObject(selSet: any, fragments: AnyObj, ctx: Ctx): AnyObj {
  const obj: AnyObj = {};
  const fields: any[] = [];
  collectFields(selSet, fragments, fields);
  for (const field of fields) {
    const name = field.name.value;
    const key = field.alias ? field.alias.value : name;
    if (name === "__typename") { obj[key] = ctx.typename || "Mock"; continue; }
    if (!field.selectionSet) { obj[key] = LIST_FIELDS.has(name) ? [leaf(name, ctx), leaf(name, { ...ctx, index: (ctx.index || 0) + 1 })] : leaf(name, ctx); continue; }
    if (isConnection(field)) {
      obj[key] = resolveConnection(field, fragments, name);
    } else if (name === "node") {
      obj[key] = resolveObject(field.selectionSet, fragments, ctx);
    } else if (name === "pageInfo") {
      obj[key] = resolveObject(field.selectionSet, fragments, { typename: "PageInfo", index: 0 });
    } else if (LIST_FIELDS.has(name)) {
      const arr = [];
      for (let i = 0; i < 6; i++) arr.push(resolveObject(field.selectionSet, fragments, { typename: cap(singular(name)), index: i }));
      obj[key] = arr;
    } else {
      obj[key] = resolveObject(field.selectionSet, fragments, { typename: nodeTypename(name), index: ctx.index || 0 });
    }
  }
  // sensible pageInfo defaults if asked for as object
  if (ctx.typename === "PageInfo") {
    if ("hasNextPage" in obj) obj.hasNextPage = false;
    if ("hasPreviousPage" in obj) obj.hasPreviousPage = false;
    if ("startCursor" in obj) obj.startCursor = "cursor-0";
    if ("endCursor" in obj) obj.endCursor = "cursor-n";
  }
  return obj;
}

function resolveConnection(field: any, fragments: AnyObj, fieldName: string): AnyObj {
  const n = edgeCount(field);
  const tn = nodeTypename(fieldName);
  const conn: AnyObj = { __typename: cap(fieldName) + "Connection" };
  const edgesField = field.selectionSet.selections.find((s: any) => s.kind === "Field" && s.name.value === "edges");
  const edges: any[] = [];
  if (edgesField && edgesField.selectionSet) {
    const nodeField = edgesField.selectionSet.selections.find((s: any) => s.kind === "Field" && s.name.value === "node");
    for (let i = 0; i < n; i++) {
      const edge: AnyObj = { __typename: tn + "Edge", cursor: "cursor-" + i };
      if (nodeField && nodeField.selectionSet)
        edge.node = resolveObject(nodeField.selectionSet, fragments, { typename: tn, index: i });
      // resolve any extra scalar fields on the edge
      for (const s of edgesField.selectionSet.selections) {
        if (s.kind === "Field" && s.name.value !== "node") {
          const kk = s.alias ? s.alias.value : s.name.value;
          edge[kk] = s.selectionSet ? resolveObject(s.selectionSet, fragments, { typename: tn, index: i }) : leaf(s.name.value, { typename: tn, index: i });
        }
      }
      edges.push(edge);
    }
  }
  // resolve the connection's own non-edge fields (totalCount, pageInfo, extras)
  for (const s of field.selectionSet.selections) {
    if (s.kind !== "Field") continue;
    const nm = s.name.value;
    const kk = s.alias ? s.alias.value : nm;
    if (nm === "edges") { conn.edges = edges; }
    else if (nm === "pageInfo") { conn.pageInfo = resolveObject(s.selectionSet, fragments, { typename: "PageInfo", index: 0 }); }
    else if (nm === "totalCount") { conn.totalCount = n === 1 ? 1 : 3; }
    else if (!s.selectionSet) { conn[kk] = leaf(nm, { typename: cap(fieldName) + "Connection", index: 0 }); }
    else { conn[kk] = resolveObject(s.selectionSet, fragments, { typename: nodeTypename(nm), index: 0 }); }
  }
  if (!("edges" in conn)) conn.edges = edges;
  return conn;
}

export function mockResolve(query: any, _variables: AnyObj): AnyObj {
  const fragments: AnyObj = {};
  let opDef: any = null;
  for (const def of query.definitions) {
    if (def.kind === "FragmentDefinition") fragments[def.name.value] = def;
    else if (def.kind === "OperationDefinition") opDef = def;
  }
  if (!opDef) return {};
  const data: AnyObj = {};
  const roots: any[] = [];
  collectFields(opDef.selectionSet, fragments, roots);
  for (const field of roots) {
    const name = field.name.value;
    const key = field.alias ? field.alias.value : name;
    if (name === "__typename") { data[key] = "Query"; continue; }
    if (!field.selectionSet) { data[key] = LIST_FIELDS.has(name) ? [leaf(name, { typename: "Query", index: 0 })] : leaf(name, { typename: "Query", index: 0 }); continue; }
    if (isConnection(field)) data[key] = resolveConnection(field, fragments, name);
    else if (LIST_FIELDS.has(name)) {
      const arr = [];
      for (let i = 0; i < 6; i++) arr.push(resolveObject(field.selectionSet, fragments, { typename: cap(singular(name)), index: i }));
      data[key] = arr;
    } else data[key] = resolveObject(field.selectionSet, fragments, { typename: nodeTypename(name), index: 0 });
  }
  return data;
}

export function makeMockClient(): ApolloClient<any> {
  const link = new ApolloLink((operation) => {
    return new Observable((observer) => {
      const t = setTimeout(() => {
        let data: AnyObj = {};
        try { data = mockResolve(operation.query, operation.variables || {}); }
        catch (e) { /* keep data empty on resolver error rather than surfacing a network error */ }
        observer.next({ data });
        observer.complete();
      }, 0);
      return () => clearTimeout(t);
    });
  });
  return new ApolloClient({
    link,
    cache: new InMemoryCache(),
    defaultOptions: {
      watchQuery: { fetchPolicy: "no-cache", errorPolicy: "all" },
      query: { fetchPolicy: "no-cache", errorPolicy: "all" },
      mutate: { fetchPolicy: "no-cache", errorPolicy: "all" },
    },
    connectToDevTools: false,
  });
}
