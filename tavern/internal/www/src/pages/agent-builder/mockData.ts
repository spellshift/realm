// Mock data for the approved agent builder landing page and wizard.
// Mirrors the Builder / BuildProfile / BuildTask ent schemas until the UI
// is connected to GraphQL.

export const NO_INSTALL_WARNING = "No persistence tome is bundled. Without one, the install command won't persist the agent.";

export type MockPlatform = "LINUX" | "MACOS" | "WINDOWS";
export type MockFormat = "BIN" | "CDYLIB" | "WINDOWS_SERVICE";

export type MockCompileTarget = {
    id: string;
    platform: MockPlatform;
    platformLabel: string;
    format: MockFormat;
    formatLabel: string;
    description: string;
};

// One entry per concrete platform+format combination a BuildTask can
// actually target (target_os x target_format), grouped by platform for
// display rather than offered as two independent dropdowns/radio groups.
// Mirrors builder.SupportedFormats (tavern/internal/builder/build_config.go):
// Linux and macOS builds are BIN-only today; only Windows has multiple
// output formats (BIN, CDYLIB, WINDOWS_SERVICE).
export const MOCK_COMPILE_TARGETS: MockCompileTarget[] = [
    { id: "linux-bin", platform: "LINUX", platformLabel: "Linux", format: "BIN", formatLabel: "Executable", description: "ELF binary, glibc/musl" },
    { id: "macos-bin", platform: "MACOS", platformLabel: "macOS", format: "BIN", formatLabel: "Executable", description: "Mach-O binary, arm64/x86_64" },
    { id: "windows-bin", platform: "WINDOWS", platformLabel: "Windows", format: "BIN", formatLabel: "Executable", description: "PE binary" },
    { id: "windows-cdylib", platform: "WINDOWS", platformLabel: "Windows", format: "CDYLIB", formatLabel: "Shared library", description: "Loadable .dll" },
    { id: "windows-service", platform: "WINDOWS", platformLabel: "Windows", format: "WINDOWS_SERVICE", formatLabel: "Windows service", description: "Installs as a service" },
];

// Mirrors builderpb.BuildProfileTransport (URI, Interval, Type) -- a
// profile holds a priority-ordered list of these, not a flat string list.
export type MockTransportType = { label: string; value: "GRPC" | "HTTP1" | "DNS" | "ICMP" };
export const MOCK_TRANSPORT_TYPES: MockTransportType[] = [
    { label: "gRPC", value: "GRPC" },
    { label: "HTTP", value: "HTTP1" },
    { label: "DNS", value: "DNS" },
    { label: "ICMP", value: "ICMP" },
];

export type MockTransportConfig = {
    id: string;
    type: MockTransportType["value"];
    uri: string;
    interval: number;
};

// Mirrors builder.DefaultTransports (tavern/internal/builder/build_config.go).
export const MOCK_DEFAULT_TRANSPORT: MockTransportConfig = {
    id: "t1",
    type: "GRPC",
    uri: "http://127.0.0.1:8000",
    interval: 5,
};

// Mirrors builderpb.BuildProfileTome (TomeID, Params) -- a reference to a
// tome plus its configured params, the same shape used by quest creation's
// tome-selection step.
export type MockTomeTactic = "RECON" | "PERSISTENCE" | "CREDENTIAL_ACCESS";
export type MockTomeRef = {
    id: string;
    tomeId: string;
    tomeName: string;
    paramSummary: string;
    tactic: MockTomeTactic;
};

// Small catalog of pickable tomes for the "+ Add tome" picker. `tactic`
// mirrors the real TomeTactic enum (utils/enums.ts) used for tactic badges
// elsewhere in the app.
export const MOCK_TOME_CATALOG: Array<{ id: string; name: string; description: string; tactic: MockTomeTactic }> = [
    { id: "101", name: "Exfil Recon", description: "Collects host/network recon data.", tactic: "RECON" },
    { id: "102", name: "Persist via Cron", description: "Installs a cron-based persistence mechanism.", tactic: "PERSISTENCE" },
    { id: "103", name: "Dump Credentials", description: "Harvests credentials from common stores.", tactic: "CREDENTIAL_ACCESS" },
];

export type MockBuildProfile = {
    id: string;
    name: string;
    description: string;
    transports: MockTransportConfig[];
    buildImage: string;
    platforms: MockPlatform[];
    // Preview-only compatibility metadata; the current BuildProfile schema
    // does not store supported compilation targets.
    supportedTargets: string[];
    tomes: MockTomeRef[];
    tomeCount: number;
    hasPersistenceTome: boolean;
    buildScript: string;
    artifactPath: string;
    prebuildscript: string;
    setupscript: string;
    postbuildscript: string;
};

export const MOCK_PROFILES: MockBuildProfile[] = [
    {
        id: "1",
        name: "Default Imix",
        description: "Standard imix build for day to day operations.",
        transports: [
            { id: "default-grpc", type: "GRPC", uri: "http://127.0.0.1:8000", interval: 5 },
            { id: "default-http", type: "HTTP1", uri: "http://127.0.0.1:8080", interval: 30 },
        ],
        buildImage: "ghcr.io/realm/builder:latest",
        platforms: ["LINUX", "MACOS", "WINDOWS"],
        supportedTargets: MOCK_COMPILE_TARGETS.map((target) => target.id),
        tomes: [
            { id: "default-recon", tomeId: "101", tomeName: "Exfil Recon", paramSummary: "No params", tactic: "RECON" },
            { id: "default-cron", tomeId: "102", tomeName: "Persist via Cron", paramSummary: "No params", tactic: "PERSISTENCE" },
            { id: "default-creds", tomeId: "103", tomeName: "Dump Credentials", paramSummary: "No params", tactic: "CREDENTIAL_ACCESS" },
        ],
        tomeCount: 3,
        hasPersistenceTome: true,
        buildScript: "{{.BuildCommand}}",
        artifactPath: "{{.ArtifactPath}}",
        prebuildscript: "echo 'no prebuild set'",
        setupscript: "cd /home/vscode && git clone https://github.com/KCarretto/realm realm",
        postbuildscript: "echo 'no postbuild set'",
    },
    {
        id: "2",
        name: "Stealth DNS",
        description: "DNS-only transport, slow jitter, signed binary.",
        transports: [{ id: "stealth-dns", type: "DNS", uri: "dns://127.0.0.1:53", interval: 60 }],
        buildImage: "ghcr.io/realm/builder:stealth",
        platforms: ["WINDOWS"],
        supportedTargets: ["windows-bin"],
        tomes: [{ id: "stealth-recon", tomeId: "101", tomeName: "Exfil Recon", paramSummary: "No params", tactic: "RECON" }],
        tomeCount: 1,
        hasPersistenceTome: false,
        buildScript: "cargo build --release --features dns_only",
        artifactPath: "target/release/imix.exe",
        prebuildscript: "echo 'no prebuild set'",
        setupscript: "cd /home/vscode && git clone https://github.com/KCarretto/realm realm",
        postbuildscript: "signtool sign /f cert.pfx target/release/imix.exe",
    },
    {
        id: "3",
        name: "macOS Service",
        description: "Builds imix as a persistent macOS launch agent.",
        transports: [{ id: "macos-http", type: "HTTP1", uri: "http://127.0.0.1:8000", interval: 30 }],
        buildImage: "ghcr.io/realm/builder:macos",
        platforms: ["MACOS"],
        supportedTargets: ["macos-bin"],
        tomes: [
            { id: "macos-cron", tomeId: "102", tomeName: "Persist via Cron", paramSummary: "No params", tactic: "PERSISTENCE" },
            { id: "macos-recon", tomeId: "101", tomeName: "Exfil Recon", paramSummary: "No params", tactic: "RECON" },
        ],
        tomeCount: 2,
        hasPersistenceTome: true,
        buildScript: "{{.BuildCommand}}",
        artifactPath: "{{.ArtifactPath}}",
        prebuildscript: "echo 'no prebuild set'",
        setupscript: "cd /home/vscode && git clone https://github.com/KCarretto/realm realm",
        postbuildscript: "echo 'no postbuild set'",
    },
];

// Mirrors the Builder ent schema (tavern/internal/ent/schema/builder.go):
// identifier, supported_targets, upstream, poll_interval, last_seen_at.
export type MockBuilder = {
    id: string;
    identifier: string;
    supportedTargets: MockPlatform[];
    upstream: string;
    pollInterval: number;
    lastSeenAt: string | null;
};

export const MOCK_BUILDERS: MockBuilder[] = [
    { id: "bd1", identifier: "builder-a1b2c3", supportedTargets: ["LINUX", "MACOS", "WINDOWS"], upstream: "http://127.0.0.1:8000", pollInterval: 5, lastSeenAt: "12s ago" },
    { id: "bd2", identifier: "builder-d4e5f6", supportedTargets: ["WINDOWS"], upstream: "http://127.0.0.1:8000", pollInterval: 5, lastSeenAt: "3m ago" },
    { id: "bd3", identifier: "builder-g7h8i9", supportedTargets: ["LINUX"], upstream: "http://127.0.0.1:8000", pollInterval: 30, lastSeenAt: null },
];

// "Built agents" = successful BuildTasks with a downloadable artifact.
export type MockBuiltAgent = {
    id: string;
    assetId: string;
    assetName: string;
    profileName: string;
    platform: MockPlatform;
    format: MockFormat;
    artifactName: string;
    artifactSize: string;
    builtAt: string;
};

export const MOCK_BUILT_AGENTS: MockBuiltAgent[] = [
    { id: "a1", assetId: "101", assetName: "build/linux/bin/imix-default-imix-agile-falcon", profileName: "Default Imix", platform: "LINUX", format: "BIN", artifactName: "imix", artifactSize: "6.1 MB", builtAt: "2h ago" },
    { id: "a2", assetId: "102", assetName: "build/macos/bin/imix-default-imix-calm-raven", profileName: "Default Imix", platform: "MACOS", format: "BIN", artifactName: "imix", artifactSize: "5.8 MB", builtAt: "2h ago" },
    { id: "a3", assetId: "103", assetName: "build/macos/bin/imix-macos-service-quiet-otter", profileName: "macOS Service", platform: "MACOS", format: "BIN", artifactName: "imix", artifactSize: "5.9 MB", builtAt: "1d ago" },
    { id: "a4", assetId: "104", assetName: "build/windows/cdylib/imix-stealth-dns-silver-fox", profileName: "Stealth DNS", platform: "WINDOWS", format: "CDYLIB", artifactName: "imix.dll", artifactSize: "4.2 MB", builtAt: "3d ago" },
];
