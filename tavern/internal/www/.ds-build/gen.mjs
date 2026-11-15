// Generate per-component artifact files for the 67 NEW design-system
// components: _preview/<Name>.js (story), components/<Name>/{preview.html,
// README.md}, components/general/<Name>/{<Name>.jsx,<Name>.d.ts}.
// Stories are classic scripts using React.createElement + window.Realm.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const www = path.resolve(here, "..");
const outProject = path.join(here, "out", "project");

// ---- shared sample data injected into every story -------------------------
const SAMPLES = `
  var noop = function(){};
  var ago = function(m){ return new Date(Date.now()-m*60000).toISOString(); };
  var ahead = function(m){ return new Date(Date.now()+m*60000).toISOString(); };
  var S = {};
  S.user = { id:"u-1", name:"alice", photoURL:"", isActivated:true, isAdmin:true };
  S.user2 = { id:"u-2", name:"bob", photoURL:"", isActivated:true, isAdmin:false };
  S.asset = {
    id:"a-1", name:"imix_linux_amd64", size:5242880,
    hash:"3b1f9e2c7a4d5068f1b2c3d4e5f60718293a4b5c6d7e8f9012a3b4c5d6e7f809",
    createdAt:ago(1440), lastModifiedAt:ago(45), creator:S.user,
    tomes:{ totalCount:2, edges:[ {node:{id:"t-1",name:"Enumerate Users"}}, {node:{id:"t-2",name:"Portscan Subnet"}} ] },
    links:{ totalCount:2, edges:[
      {node:{id:"l-1",path:"/l/ab12cd34",expiresAt:ahead(10080),downloadLimit:25,downloads:3,creator:S.user}},
      {node:{id:"l-2",path:"/l/ef56gh78",expiresAt:null,downloadLimit:0,downloads:11,creator:S.user2}}
    ] }
  };
  S.credentials = [
    { id:"c-1", createdAt:ago(900), lastModifiedAt:ago(30), principal:"administrator", kind:"password", secret:"S3cr3t-P@ss!" },
    { id:"c-2", createdAt:ago(800), lastModifiedAt:ago(120), principal:"svc_sql", kind:"hash", secret:"aad3b435b51404eeaad3b435b51404ee" },
    { id:"c-3", createdAt:ago(400), lastModifiedAt:ago(10), principal:"root", kind:"ssh_key", secret:"-----BEGIN OPENSSH PRIVATE KEY-----" }
  ];
  S.repositories = [
    { node:{ id:"r-1", url:"https://github.com/acme/red-tomes", repoType:"FIRST_PARTY", lastModifiedAt:ago(2880), owner:S.user, publicKey:"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAA",
      tomes:[ {id:"t-1",name:"Enumerate Users",paramDefs:"[]"}, {id:"t-2",name:"Portscan Subnet",paramDefs:"[]"} ] } },
    { node:{ id:"r-2", url:"https://github.com/acme/persistence", repoType:"THIRD_PARTY", lastModifiedAt:ago(10080), owner:S.user2, publicKey:"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5BBBB",
      tomes:[ {id:"t-3",name:"Install Persistence",paramDefs:"[]"} ] } }
  ];
  S.profiles = [
    { id:"p-1", name:"Linux implant", description:"glibc ELF beacon over HTTPS", transports:[], buildImage:"realm/builder:latest", tomes:[], tomeCount:3, hasPersistenceTome:true, buildScript:"", artifactPath:"/out/imix", prebuildscript:"", setupscript:"", postbuildscript:"" },
    { id:"p-2", name:"Windows service", description:"PE service persistence", transports:[], buildImage:"realm/builder:latest", tomes:[], tomeCount:5, hasPersistenceTome:true, buildScript:"", artifactPath:"/out/imix.exe", prebuildscript:"", setupscript:"", postbuildscript:"" }
  ];
  S.targets = [
    { id:"linux-bin", platform:"LINUX", platformLabel:"Linux", format:"BIN", formatLabel:"Binary", description:"ELF binary, glibc/musl" },
    { id:"windows-bin", platform:"WINDOWS", platformLabel:"Windows", format:"BIN", formatLabel:"Binary", description:"PE binary" },
    { id:"windows-cdylib", platform:"WINDOWS", platformLabel:"Windows", format:"CDYLIB", formatLabel:"DLL", description:"Loadable .dll" }
  ];
  S.catalog = [
    { id:"t-1", name:"Enumerate Users", description:"List local user accounts", tactic:"DISCOVERY", paramDefs:"[]" },
    { id:"t-2", name:"Portscan Subnet", description:"TCP connect scan of a CIDR", tactic:"DISCOVERY", paramDefs:"[]" }
  ];
  S.quest = { id:"q-1", name:"Initial Access", creator:S.user, createdAt:ago(300), tome:{id:"t-1",name:"Enumerate Users"}, tasks:{totalCount:5}, par:"[]", parameters:"{}" };
  S.formik = {
    values:{ name:"Recon sweep", tomeId:null, params:[], beacons:[] }, errors:{}, touched:{}, status:{}, isSubmitting:false, isValid:true, dirty:false,
    setFieldValue:noop, setFieldTouched:noop, setFieldError:noop, setValues:noop, setErrors:noop, setStatus:noop,
    handleChange:noop, handleBlur:noop, handleSubmit:noop, handleReset:noop, submitForm:function(){return Promise.resolve();},
    validateForm:function(){return Promise.resolve({});}, resetForm:noop, registerField:noop, unregisterField:noop,
    getFieldProps:function(n){ var name = (n&&n.name)||n||""; return { name:name, value:"", onChange:noop, onBlur:noop }; },
    getFieldMeta:function(){ return { value:"", touched:false, error:undefined, initialValue:"" }; },
    getFieldHelpers:function(){ return { setValue:noop, setTouched:noop, setError:noop }; }
  };
  S.fileItem = { id:"f-1", file:{ name:"imix_linux_amd64", size:5242880, type:"application/octet-stream" }, name:"imix_linux_amd64", status:"success", progress:100 };
  S.historyGroup = {
    key:"g-1", count:3, credCount:1, fileCount:0, processCount:2,
    primaryTask:{ id:"task-1", eldritch:"process.list()", input:"process.list()", cleanedInput:"process.list()", output:"root\\nadministrator\\nsvc_sql\\n", error:"", status:"finished", createdAt:ago(20), shellId:"s-1", tome:{id:"t-x",name:"List Processes"}, quest:{id:"q-1",name:"Recon"} },
    occurrences:[]
  };
`;

// expr helpers used in SPEC stories: h (createElement), DS (window.Realm), S, noop
const box = (w, child) => `h('div',{style:{maxWidth:${w},padding:8}}, ${child})`;

// ---- per-component spec ----------------------------------------------------
// group defaults "general", mode defaults "single".
const SPEC = {
  // ---------- assets (priority) ----------
  AssetsTable: { desc: "Virtualized table of uploaded assets with per-row detail, download, create-link and (admin) delete actions.",
    stories: { Default: `h(DS.AssetsTable,{assetIds:["a-1","a-2","a-3","a-4"],hasMore:false,onCreateLink:noop,onAssetUpdate:noop})` } },
  AssetAccordion: { desc: "Expandable panel listing the download links generated for a single asset.",
    stories: { Default: box(760, `h(DS.AssetAccordion,{asset:S.asset,onUpdate:noop})`) } },
  AssetsHeader: { desc: "Header bar for the Assets page: upload and generate-agent actions.",
    stories: { Default: `h(DS.AssetsHeader,{setOpen:noop,onGenerateAgent:noop})` } },
  DownloadButton: { desc: "Icon button that downloads a named asset.", mode: "grid",
    stories: { Default: `h(DS.DownloadButton,{assetName:"imix_linux_amd64"})` } },
  CreateLinkModal: { desc: "Modal for minting a shareable, expiring download link for an asset.",
    stories: { Open: `h(DS.CreateLinkModal,{isOpen:true,setOpen:noop,assetId:"a-1",assetName:"imix_linux_amd64",onSuccess:noop})` } },
  UploadAssetModal: { desc: "Drag-and-drop modal for uploading one or more asset files.",
    stories: { Open: `h(DS.UploadAssetModal,{isOpen:true,setOpen:noop,onUploadSuccess:noop})` } },
  FileCard: { desc: "Row in the upload modal showing one staged file, its progress and a remove control.",
    stories: { Default: box(520, `h(DS.FileCard,{item:S.fileItem,index:0,formik:S.formik,onRemove:noop})`) } },

  // ---------- host-details tables ----------
  BeaconsTable: { desc: "Virtualized table of a host's beacons with a create-quest action.",
    stories: { Default: `h(DS.BeaconsTable,{beaconIds:["b-1","b-2","b-3"]})` } },
  CredentialTable: { desc: "Table of credentials harvested from a host.",
    stories: { Default: `h(DS.CredentialTable,{data:S.credentials,showLastModified:true})` } },
  FilesTable: { desc: "Virtualized table of files pulled from a host.",
    stories: { Default: `h(DS.FilesTable,{hostId:"h-sample",fileIds:["f-1","f-2","f-3"]})` } },
  PortalsTable: { desc: "Virtualized table of a host's portal (pivot) sessions.",
    stories: { Default: `h(DS.PortalsTable,{portalIds:["p-1","p-2"]})` } },
  ProcessesTable: { desc: "Virtualized table of a host's running processes.",
    stories: { Default: `h(DS.ProcessesTable,{hostId:"h-sample",processIds:["pr-1","pr-2","pr-3"]})` } },
  ShellsTable: { desc: "Virtualized table of interactive shell sessions on a host.",
    stories: { Default: `h(DS.ShellsTable,{shellIds:["s-1","s-2"]})` } },

  // ---------- page tables ----------
  HostsTable: { desc: "Virtualized table of hosts across the engagement.",
    stories: { Default: `h(DS.HostsTable,{hostIds:["h-1","h-2","h-3"]})` } },
  QuestsTable: { desc: "Virtualized table of quests with filtering.",
    stories: { Default: `h(DS.QuestsTable,{questIds:["q-1","q-2"]})` } },
  UserTable: { desc: "Admin table of user accounts with activate/admin controls.",
    stories: { Default: `h(DS.UserTable,{userIds:["u-1","u-2"],currentUser:S.user})` } },
  RepositoryTable: { desc: "Table of tome repositories with expandable per-repo tome lists.",
    stories: { Default: `h(DS.RepositoryTable,{repositories:S.repositories})` } },

  // ---------- dashboard ----------
  MetricCard: { desc: "Compact KPI tile: a labelled count with an optional trend indicator.", mode: "grid",
    stories: {
      Plain: `h(DS.MetricCard,{label:"Active beacons",count:128,timeframe:"last 24h"})`,
      Trending: `h(DS.MetricCard,{label:"Quests",count:42,timeframe:"last 24h",trend:"up",trendValue:12})`,
    } },
  BeaconSummaryCard: { desc: "Dashboard card charting beacon activity over a selectable time range.",
    stories: { Default: box(760, `h(DS.BeaconSummaryCard)`) } },
  QuestSummaryCard: { desc: "Dashboard card charting quest activity and active tactics over time.",
    stories: { Default: box(760, `h(DS.QuestSummaryCard)`) } },
  TagSummaryCard: { desc: "Dashboard card summarising host access grouped by tag, with admin auto-tag controls.",
    stories: { Default: box(760, `h(DS.TagSummaryCard)`) } },

  // ---------- tasks / shell ----------
  TaskCardVirtualized: { desc: "Self-fetching task card for virtualized task lists.",
    stories: { Default: box(760, `h(DS.TaskCardVirtualized,{itemId:"task-1",isVisible:true})`) } },
  HistoryTaskCard: { desc: "Aggregated shell-history entry with run and load-into-composer actions.",
    stories: { Default: box(760, `h(DS.HistoryTaskCard,{group:S.historyGroup,onRunCode:noop,onLoadIntoComposer:noop,onSelectShell:noop})`) } },
  Credential: { desc: "Masked secret display with reveal/copy affordances.", mode: "grid",
    stories: { Default: `h(DS.Credential,{secret:"S3cr3t-P@ssw0rd!"})` } },
  RerunQuestDropdown: { desc: "Dropdown that re-runs a quest, pre-filling the create-quest modal.",
    stories: { Default: `h(DS.RerunQuestDropdown,{quest:S.quest})` } },
  EditablePageHeader: { desc: "Quest page header with inline-editable title and re-run action.",
    stories: { Default: `h(DS.EditablePageHeader)` } },

  // ---------- empty states ----------
  EmptyStateNoBeacon: { desc: "Empty state shown when no beacons have called back yet.",
    stories: { Default: box(640, `h(DS.EmptyStateNoBeacon)`) } },
  EmptyStateNoQuests: { desc: "Empty state shown when no quests exist, with a create-quest action.",
    stories: { Default: box(640, `h(DS.EmptyStateNoQuests)`) } },

  // ---------- general building blocks ----------
  Breadcrumbs: { desc: "Breadcrumb trail of labelled links.",
    stories: { Default: `h(DS.Breadcrumbs,{pages:[{label:"Hosts",link:"/hosts"},{label:"DESKTOP-7F3A2C",link:"/hosts/h-sample"}]})` } },
  ButtonDialogPopover: { desc: "Button that opens a popover dialog anchored to it.",
    stories: { Default: `h(DS.ButtonDialogPopover,{label:"Filters",leftIcon:h('span',{style:{fontSize:14}},"\\u2699"),children:h('div',{style:{padding:12,minWidth:180}},"Popover content")})` } },
  UserImageAndName: { desc: "Avatar + display name for a user.", mode: "grid",
    stories: { Default: `h(DS.UserImageAndName,{userData:S.user})`, Unknown: `h(DS.UserImageAndName,{userData:null})` } },
  NotificationBell: { desc: "Header bell showing unread notifications in a popover.",
    stories: { Default: box(360, `h(DS.NotificationBell,{})`) } },

  // ---------- forms ----------
  FormTextArea: { desc: "Formik-bound multi-line text field.",
    stories: { Default: box(480, `h(DS.FormTextArea,{formik:S.formik,name:"notes",label:"Notes",placeholder:"Add a note"})`) } },
  DebouncedFreeTextSearch: { desc: "Free-text search input that debounces before firing its callback.",
    stories: { Default: box(420, `h(DS.DebouncedFreeTextSearch,{setSearchInput:noop,placeholder:"Search"})`) } },

  // ---------- quest buttons / host actions ----------
  CreateShellButton: { desc: "Opens an interactive shell quest against a host or beacon.",
    stories: { Default: `h(DS.CreateShellButton,{hostId:"h-sample",beaconId:"b-1"})` } },
  HostSubscriptionButton: { desc: "Toggles the current user's subscription to a host's activity.",
    stories: { Default: `h(DS.HostSubscriptionButton,{hostId:"h-sample"})` } },
  CreatePortalQuestButton: { desc: "Starts a portal (pivot) quest from the host portal tab.", mode: "grid",
    stories: { Solid: `h(DS.CreatePortalQuestButton,{variant:"solid",size:"md"})`, Ghost: `h(DS.CreatePortalQuestButton,{variant:"ghost",size:"sm"})` } },
  CreateProcessQuestButton: { desc: "Fetches a host's process list via a new quest.",
    stories: { Default: `h(DS.CreateProcessQuestButton)` } },
  RefetchProcessQuestButton: { desc: "Re-runs the process-list quest for a host.",
    stories: { Default: `h(DS.RefetchProcessQuestButton)` } },

  // ---------- host-details sections / tags ----------
  EditableTag: { desc: "Inline-editable group/service tag chip for a host.", mode: "grid",
    stories: { Group: `h(DS.EditableTag,{kind:"group",hostId:"h-sample",tagSaved:{id:"tg-1",name:"production",kind:"group"}})`, New: `h(DS.EditableTag,{kind:"service",hostId:"h-sample"})` } },
  HostDetailsSection: { desc: "Summary panel of a host's IPs, platform and tags (reads HostContext).",
    stories: { Default: box(760, `h(DS.HostDetailsSection)`) } },
  HostHeader: { desc: "Host-details page header with breadcrumbs, subscribe and create-quest (reads HostContext).",
    stories: { Default: `h(DS.HostHeader)` } },
  HostTabs: { desc: "Tab bar for the host-details page with per-tab counts (reads HostContext).",
    stories: { Default: `h(DS.__Tab.Group,null,h(DS.HostTabs))` } },

  // ---------- page headers / tabs ----------
  QuestHeader: { desc: "Quests page header with a create-quest action.",
    stories: { Default: `h(DS.QuestHeader)` } },
  TomesHeader: { desc: "Tomes page header with an import-repository action.",
    stories: { Default: `h(DS.TomesHeader,{setOpen:noop})` } },
  AdminTabs: { desc: "Admin portal tab bar (users / profiles / builders) with counts.",
    stories: { Default: `h(DS.__Tab.Group,null,h(DS.AdminTabs,{userCount:12,profileCount:4,builderCount:2}))` } },
  AgentBuilderTabs: { desc: "Agent-builder tab bar (agents / profiles / builders) with counts.",
    stories: { Default: `h(DS.__Tab.Group,null,h(DS.AgentBuilderTabs,{agentCount:8,profileCount:4,builderCount:2}))` } },

  // ---------- agent builder ----------
  AgentBuilderWizard: { desc: "Multi-step wizard for compiling and deploying an agent.",
    stories: { Default: box(960, `h(DS.AgentBuilderWizard,{showHeader:true,onComplete:noop,onCancel:noop})`) } },
  ProfileSelectionTable: { desc: "Selectable table of saved build profiles.",
    stories: { Default: `h(DS.ProfileSelectionTable,{profiles:S.profiles,selectedProfileId:"p-1",onSelectProfile:noop})` } },
  TargetSelectionTable: { desc: "Selectable table of compile targets (platform × format).",
    stories: { Default: `h(DS.TargetSelectionTable,{targets:S.targets,selectedTargetId:"linux-bin",onSelectTarget:noop})` } },
  NewBuildProfileModal: { desc: "Modal for creating or editing a build profile.",
    stories: { Open: `h(DS.NewBuildProfileModal,{isOpen:true,setOpen:noop,catalog:S.catalog,onSave:noop})` } },

  // ---------- tomes import ----------
  ImportRepositoryModal: { desc: "Modal wizard for importing a tome repository.",
    stories: { Open: `h(DS.ImportRepositoryModal,{isOpen:true,setOpen:noop})` } },
  StepCreateRepository: { desc: "Import step: enter a repository URL to create it.",
    stories: { Default: box(640, `h(DS.StepCreateRepository,{setCurrStep:noop,setNewRepository:noop})`) } },
  StepAddDeploymentKey: { desc: "Import step: add the generated deploy key to the repository.",
    stories: { Default: box(640, `h(DS.StepAddDeploymentKey,{setCurrStep:noop,setOpen:noop,newRepository:S.repositories[0].node})`) } },

  // ---------- create-quest-modal internals (parent-coupled; best effort) ----------
  BeaconSelectionStep: { desc: "Create-quest step: choose which beacons to task.",
    stories: { Default: box(960, `h(DS.BeaconSelectionStep,{setCurrStep:noop,setOpen:noop,formik:S.formik,initialFilters:undefined})`) } },
  TomeSelectionStep: { desc: "Create-quest step: choose a tome to run.",
    stories: { Default: box(960, `h(DS.TomeSelectionStep,{setCurrStep:noop,formik:S.formik,initialFilters:undefined})`) } },
  FinalizeSelection: { desc: "Create-quest final step: review and launch the quest.",
    stories: { Default: box(760, `h(DS.FinalizeSelection,{setCurrStep:noop,formik:S.formik,loading:false})`) } },
  ConfigureParams: { desc: "Create-quest step: fill in a tome's parameters.",
    stories: { Default: box(640, `h(DS.ConfigureParams,{formik:S.formik})`) } },
  TomeTableWrapper: { desc: "Filterable wrapper around the tome-selection table.",
    stories: { Default: box(760, `h(DS.TomeTableWrapper,{formik:S.formik})`) } },
  TomeTable: { desc: "Selectable table of tomes used inside the create-quest modal.",
    stories: { Default: box(760, `h(DS.TomeTable,{tomeIds:["t-1","t-2","t-3"],selectable:true,selectedTomeId:"t-1",onSelectTome:noop})`) } },
  BeaconTable: { desc: "Selectable table of beacons used inside the create-quest modal.",
    stories: { Default: box(760, `h(DS.BeaconTable,{beaconIds:["b-1","b-2","b-3"],selectable:true,selectedBeaconIds:["b-1"],onToggle:noop})`) } },
  BeaconOption: { desc: "Checkbox row rendering one beacon (a BeaconTile) for the beacon picker list.",
    stories: { Default: box(420, `h(DS.BeaconOption,{index:0,style:{},toggleCheck:noop,beaconsSelected:{"b-1":true},beaconsToDisplay:[{id:"b-1",name:"strong-eagle",principal:"SYSTEM",lastSeenAt:ago(3),host:{id:"h-1",name:"DESKTOP-7F3A2C",platform:"Windows"},interval:60}]})`) } },

  // ---------- virtualized primitives ----------
  VirtualizedTable: { desc: "Generic virtualized, self-fetching data table driven by a GraphQL query per row.",
    stories: { Default: `h(DS.VirtualizedTable,{items:["h-1","h-2","h-3"],columns:[{key:"name",label:"Name",width:"minmax(200px,2fr)",render:function(d){return d&&d.name}},{key:"platform",label:"Platform",width:"minmax(120px,1fr)",render:function(d){return d&&d.platform}}],query:DS.__gql("query DemoRow($id:ID!){ hosts(where:{id:$id}){ edges{ node{ id name platform } } } }"),getVariables:function(id){return {id:id}},extractData:function(r){return r&&r.hosts&&r.hosts.edges[0]&&r.hosts.edges[0].node}})` } },
  VirtualizedTableWrapper: { desc: "VirtualizedTable plus a filter/search toolbar (reads FilterContext).",
    stories: { Default: `h(DS.VirtualizedTableWrapper,{items:["h-1","h-2","h-3"],columns:[{key:"name",label:"Name",width:"minmax(200px,2fr)",render:function(d){return d&&d.name}}],query:DS.__gql("query DemoRow2($id:ID!){ hosts(where:{id:$id}){ edges{ node{ id name } } } }"),getVariables:function(id){return {id:id}},extractData:function(r){return r&&r.hosts&&r.hosts.edges[0]&&r.hosts.edges[0].node}})` } },
  VirtualizedCardItem: { desc: "Generic virtualized, self-fetching card for card lists.",
    stories: { Default: box(520, `h(DS.VirtualizedCardItem,{itemId:"h-1",isVisible:true,query:DS.__gql("query DemoCard($id:ID!){ hosts(where:{id:$id}){ edges{ node{ id name platform } } } }"),getVariables:function(id){return {id:id}},extractData:function(r){return r&&r.hosts&&r.hosts.edges[0]&&r.hosts.edges[0].node},renderSkeleton:function(){return h("div",{style:{padding:12}},"Loading...")},renderCard:function(d){return h("div",{style:{padding:12,border:"1px solid #e5e7eb",borderRadius:8}}, d?d.name:"...")}})`) } },
  VirtualizedCardList: { desc: "Virtualized list of self-fetching cards.",
    stories: { Default: box(520, `h(DS.VirtualizedCardList,{items:["h-1","h-2","h-3"],query:DS.__gql("query DemoCardL($id:ID!){ hosts(where:{id:$id}){ edges{ node{ id name platform } } } }"),getVariables:function(id){return {id:id}},extractData:function(r){return r&&r.hosts&&r.hosts.edges[0]&&r.hosts.edges[0].node},renderCard:function(d){return h("div",{style:{padding:12,border:"1px solid #e5e7eb",borderRadius:8,marginBottom:8}}, d?d.name:"...")}})`) } },
  VirtualizedCardListWrapper: { desc: "VirtualizedCardList plus a filter/search toolbar (reads FilterContext).",
    stories: { Default: box(520, `h(DS.VirtualizedCardListWrapper,{items:["h-1","h-2"],query:DS.__gql("query DemoCardLW($id:ID!){ hosts(where:{id:$id}){ edges{ node{ id name } } } }"),getVariables:function(id){return {id:id}},extractData:function(r){return r&&r.hosts&&r.hosts.edges[0]&&r.hosts.edges[0].node},renderCard:function(d){return h("div",{style:{padding:12}}, d?d.name:"...")}})`) } },
};

// ---- accurate props extraction for .d.ts + README -------------------------
function resolveFile(rel) {
  const p = path.join(www, "src", rel.slice(2));
  for (const c of [p + ".tsx", p + ".ts", p + "/index.tsx", p + "/index.ts"])
    if (fs.existsSync(c)) return c;
  return null;
}
function extractProps(name, src) {
  const propsName = name + "Props";
  const re = new RegExp("(export\\s+)?(interface|type)\\s+" + propsName + "\\b");
  const idx = src.search(re);
  if (idx < 0) return null;
  let seg = src.slice(idx);
  const isType = /^(export\s+)?type/.test(seg);
  let end;
  if (!isType) {
    let depth = 0, i = seg.indexOf("{");
    for (let j = i; j < seg.length; j++) { if (seg[j] === "{") depth++; else if (seg[j] === "}") { depth--; if (depth === 0) { end = j + 1; break; } } }
  } else {
    const eq = seg.indexOf("=");
    let k = eq + 1; while (k < seg.length && /\s/.test(seg[k])) k++;
    if (seg[k] === "{") {
      let depth = 0;
      for (let j = k; j < seg.length; j++) { if (seg[j] === "{") depth++; else if (seg[j] === "}") { depth--; if (depth === 0) { end = seg.indexOf(";", j); end = end < 0 ? j + 1 : end + 1; break; } } }
    } else {
      end = seg.indexOf(";"); end = end < 0 ? seg.indexOf("\n") : end + 1;
    }
  }
  return seg.slice(0, end).replace(/^export\s+/, "").trim();
}

// ---- parse design-system.ts Added block -----------------------------------
const ds = fs.readFileSync(path.join(www, "src/design-system.ts"), "utf8");
const added = ds.split("--- Added")[1];
const comps = [];
const re = /export \{\s*([A-Za-z0-9_]+)\s*\} from "(\.\/[^"]+)"/g;
let m;
const seen = new Set();
while ((m = re.exec(added))) { if (!seen.has(m[1])) { seen.add(m[1]); comps.push([m[1], m[2]]); } }

// ---- templates -------------------------------------------------------------
function storyFile(name, spec) {
  const stories = spec.stories || { Default: `h(DS.${name})` };
  const body = Object.entries(stories)
    .map(([k, expr]) => `    ${k}: function () { return ${expr}; }`)
    .join(",\n");
  return `"use strict";
var __dsPreview = (function () {
  var h = React.createElement, DS = window.Realm;
${SAMPLES}
  return {
${body}
  };
})();
`;
}
function previewHtml(name, spec) {
  const group = spec.group || "general";
  const mode = spec.mode || "single";
  const primary = spec.primary || "";
  return `<!-- @dsCard group="${group}" -->
<!doctype html>
<html><head><meta charset="utf-8">
  <style>
    body{margin:0;padding:24px;background:#fff}
    .ds-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:20px;align-items:start}
    .ds-grid.ds-col{grid-template-columns:1fr}
    .ds-cell{border:1px solid #e5e7eb;border-radius:8px;padding:12px;min-width:0;overflow:hidden;transform:translateZ(0)}
    .ds-cell>h4{margin:0 0 8px;font:600 12px system-ui;color:#6b7280;text-transform:uppercase;letter-spacing:.04em}
    .ds-single{transform:translateZ(0)}
  </style>
</head><body>
  <div class="ds-grid" id="g"></div>
  <script src="../../_preview/${name}.js"></script>
  <script>
    var h=React.createElement, g=document.getElementById('g');
    var E=[]; for (var k in (window.__dsPreview||{})) {
      if (typeof window.__dsPreview[k]==='function' && /^[A-Z]/.test(k)) E.push(k);
    }
    window.__dsCells=E.slice();
    var q=null; try{q=new URLSearchParams(location.search).get('story')}catch(e){}
    var MODE="${mode}";
    window.__dsMode=MODE;
    var PRIMARY="${primary}";
    if(MODE==='column'){
      g.className+=' ds-col';
      var cpi=PRIMARY?E.indexOf(PRIMARY):-1;
      if(cpi>0){E.splice(cpi,1);E.unshift(PRIMARY)}
    }
    function mount(id,key){try{ReactDOM.createRoot(document.getElementById(id)).render(h(window.Realm.PreviewShell,{},h(window.__dsPreview[key])))}catch(e){document.getElementById(id).textContent='\\u26A0 '+(e&&e.message||e)}}
    var pick=null;
    if(q){for(var j=0;j<E.length;j++){if(E[j]===q||E[j].toLowerCase()===q.toLowerCase()){pick=E[j];break}}}
    else if(MODE==='single'&&E.length){pick=E.indexOf(PRIMARY)>=0?PRIMARY:E[0]}
    if(q&&!pick){g.textContent='\\u26A0 no export named '+q}
    else if(pick){
      var s=document.createElement('div'); s.className='ds-single'; s.id='r0';
      if(!q)document.body.style.padding='0';
      g.parentNode.replaceChild(s,g); mount('r0',pick);
    } else {
      for(var i=0;i<E.length;i++){
        var cell=document.createElement('section'); cell.className='ds-cell';
        cell.innerHTML='<h4>'+E[i]+'</h4><div id="r'+i+'"></div>'; g.appendChild(cell);
        mount('r'+i,E[i]);
      }
      if(E.length===0){g.textContent='\\u26A0 no PascalCase exports in _preview/${name}.js'}
    }
  </script>
</body></html>
`;
}
function readme(name, spec, propsBlock) {
  const stories = Object.keys(spec.stories || { Default: 1 });
  return `${name} — ${spec.desc} Rendered via \`window.Realm.${name}\` inside \`<PreviewShell>\` (the full app provider stack — Chakra, router, Authorization/Filter/Sort/UserPreferences/CreateQuestModal/Host/Polling/Tag contexts — over a mocked GraphQL API).

## Props

\`\`\`ts
${propsBlock || "// No props."}
\`\`\`

## Preview

Stories: ${stories.map((s) => "`" + s + "`").join(", ")}. The preview mounts sample data through the mocked API; in the real app this component is wired to live Tavern GraphQL.
`;
}
function dts(name, propsBlock) {
  let block = propsBlock;
  if (!block) block = `export interface ${name}Props {\n  [key: string]: unknown;\n}`;
  else if (/^(interface|type)/.test(block)) block = "export " + block;
  return `import * as React from 'react';

/** ${name} — from Realm (tavern/internal/www). */
${block}

export declare const ${name}: React.ComponentType<${name}Props>;
`;
}
function jsxStub(name) {
  return `// Re-export of Realm ${name}. Implementation is in the root bundle (window.Realm).
Object.assign(window, { ${name}: window.Realm.${name} });
`;
}

// ---- emit ------------------------------------------------------------------
fs.rmSync(outProject, { recursive: true, force: true });
const report = [];
for (const [name, rel] of comps) {
  const spec = SPEC[name] || { desc: `${name} component from Realm.`, stories: { Default: `h(DS.${name})` } };
  const file = resolveFile(rel);
  const src = file ? fs.readFileSync(file, "utf8") : "";
  const propsBlock = file ? extractProps(name, src) : null;

  const prevDir = path.join(outProject, "_preview");
  const compDir = path.join(outProject, "components", name);
  const genDir = path.join(outProject, "components", "general", name);
  fs.mkdirSync(prevDir, { recursive: true });
  fs.mkdirSync(compDir, { recursive: true });
  fs.mkdirSync(genDir, { recursive: true });

  fs.writeFileSync(path.join(prevDir, name + ".js"), storyFile(name, spec));
  fs.writeFileSync(path.join(compDir, "preview.html"), previewHtml(name, spec));
  fs.writeFileSync(path.join(compDir, "README.md"), readme(name, spec, propsBlock));
  fs.writeFileSync(path.join(genDir, name + ".d.ts"), dts(name, propsBlock));
  fs.writeFileSync(path.join(genDir, name + ".jsx"), jsxStub(name));
  report.push({ name, hasProps: !!propsBlock, stories: Object.keys(spec.stories || {}), custom: !!SPEC[name] });
}
fs.writeFileSync(path.join(here, "out", "gen-report.json"), JSON.stringify(report, null, 1));
console.log("generated", comps.length, "components;",
  report.filter((r) => !r.custom).length, "without a custom spec;",
  report.filter((r) => !r.hasProps).length, "without extracted props");
