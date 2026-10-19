export const NO_INSTALL_WARNING = "No persistence tome is bundled. Without one, the install command won't persist the agent.";

export type Platform = "LINUX" | "MACOS" | "WINDOWS";
export type Format = "BIN" | "CDYLIB" | "WINDOWS_SERVICE";

export type CompileTarget = {
    id: string;
    platform: Platform;
    platformLabel: string;
    format: Format;
    formatLabel: string;
    description: string;
};

// One entry per concrete platform+format combination a BuildTask can
// actually target (target_os x target_format), grouped by platform for
// display rather than offered as two independent dropdowns/radio groups.
// Mirrors builder.SupportedFormats (tavern/internal/builder/build_config.go):
// Linux and macOS builds are BIN-only today; only Windows has multiple
// output formats (BIN, CDYLIB, WINDOWS_SERVICE).
export const COMPILE_TARGETS: CompileTarget[] = [
    { id: "linux-bin", platform: "LINUX", platformLabel: "Linux", format: "BIN", formatLabel: "Executable", description: "ELF binary, glibc/musl" },
    { id: "macos-bin", platform: "MACOS", platformLabel: "macOS", format: "BIN", formatLabel: "Executable", description: "Mach-O binary, arm64/x86_64" },
    { id: "windows-bin", platform: "WINDOWS", platformLabel: "Windows", format: "BIN", formatLabel: "Executable", description: "PE binary" },
    { id: "windows-cdylib", platform: "WINDOWS", platformLabel: "Windows", format: "CDYLIB", formatLabel: "Shared library", description: "Loadable .dll" },
    { id: "windows-service", platform: "WINDOWS", platformLabel: "Windows", format: "WINDOWS_SERVICE", formatLabel: "Windows service", description: "Installs as a service" },
];

// Mirrors builderpb.BuildProfileTransport (URI, Interval, Type) -- a
// profile holds a priority-ordered list of these, not a flat string list.
export type TransportType = { label: string; value: "GRPC" | "HTTP1" | "DNS" | "ICMP" };
export const TRANSPORT_TYPES: TransportType[] = [
    { label: "gRPC", value: "GRPC" },
    { label: "HTTP", value: "HTTP1" },
    { label: "DNS", value: "DNS" },
    { label: "ICMP", value: "ICMP" },
];

export type TransportConfig = {
    id: string;
    type: TransportType["value"];
    uri: string;
    interval: number;
    extra?: string;
};

// Mirrors builder.DefaultTransports (tavern/internal/builder/build_config.go).
export const DEFAULT_TRANSPORT: TransportConfig = {
    id: "t1",
    type: "GRPC",
    uri: "http://127.0.0.1:8000",
    interval: 5,
};

// Mirrors builderpb.BuildProfileTome (TomeID, Params) -- a reference to a
// tome plus its configured params, the same shape used by quest creation's
// tome-selection step.
export type TomeTactic = keyof typeof import("../../utils/enums").TomeTactic;
export type TomeRef = {
    id: string;
    tomeId: string;
    tomeName: string;
    paramSummary: string;
    params: string;
    tactic: TomeTactic;
};

export type BuildProfile = {
    id: string;
    name: string;
    description: string;
    transports: TransportConfig[];
    buildImage: string;
    tomes: TomeRef[];
    tomeCount: number;
    hasPersistenceTome: boolean;
    buildScript: string;
    artifactPath: string;
    prebuildscript: string;
    setupscript: string;
    postbuildscript: string;
    unique?: string;
};

// Mirrors the Builder ent schema (tavern/internal/ent/schema/builder.go):
// identifier, supported_targets, upstream, poll_interval, last_seen_at.
export type Builder = {
    id: string;
    identifier: string;
    supportedTargets: Platform[];
    upstream: string;
    pollInterval: number;
    lastSeenAt: string | null;
    // Raw ISO timestamp lastSeenAt is derived from, kept alongside the
    // human-readable string above so health can be recomputed without a
    // round trip, and whether that check currently passes.
    lastSeenAtRaw: string | null;
    online: boolean;
};

// Mirrors BuilderHealthy (tavern/internal/builder/config.go): a builder is
// healthy if it has checked in within three of its own polling intervals.
// Build creation (CreateBuildTask resolver) only considers healthy builders,
// so the UI uses the same rule to decide whether a target is actually
// buildable right now, not just nominally supported by some builder.
export function BuilderHealthy(lastSeenAt: string | null, pollInterval: number, now: number = Date.now()): boolean {
    if (!lastSeenAt || !Number.isFinite(pollInterval) || pollInterval <= 0) return false;
    const lastSeen = new Date(lastSeenAt).getTime();
    if (Number.isNaN(lastSeen)) return false;
    return lastSeen >= now - 3 * pollInterval * 1000;
}

export type BuildTaskStatus = "Queued" | "Claimed" | "Building" | "Awaiting artifact" | "Built" | "Failed";

// One row per BuildTask shown in the "Built agents" tab's table -- queued,
// claimed, and building tasks are included alongside finished ones so the
// table itself reflects live build progress. Only `Built` rows have
// artifact fields populated.
export type BuildTaskRow = {
    id: string;
    status: BuildTaskStatus;
    profileName: string;
    platform: Platform;
    format: Format;
    error: string | null;
    timestamp: string;
    assetId: string | null;
    assetName: string | null;
    artifactName: string | null;
    artifactSize: string | null;
};


export const DEFAULT_SETUP_SCRIPT = "cd /home/vscode && git clone https://github.com/spellshift/realm.git realm && cd realm/implants/imix && rm -rf install_scripts/* && cp -r /mnt/tomes/* install_scripts/ 2>/dev/null || true";

export type Tome = { id: string; name: string; description: string; tactic: TomeTactic; paramDefs?: string | null };
