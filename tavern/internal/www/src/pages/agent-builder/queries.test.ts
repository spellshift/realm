import { describe, it, expect } from "vitest";
import { mapProfile, profileInput, taskStatus, buildTaskRows, BuildTask, ProfileRecord } from "./queries";
import { BuildProfile, Tome } from "./types";

const catalog: Tome[] = [
    { id: "101", name: "Exfil Recon", description: "", tactic: "RECON" },
    { id: "102", name: "Persist via Cron", description: "", tactic: "PERSISTENCE" },
];

describe("mapProfile", () => {
    const record: ProfileRecord = {
        id: "1",
        name: "Default Imix",
        description: "Standard build",
        buildImage: "spellshift/devcontainer:main",
        buildScript: "{{.BuildCommand}}",
        artifactPath: "{{.ArtifactPath}}",
        prebuildscript: "echo pre",
        setupscript: "echo setup",
        postbuildscript: "echo post",
        unique: undefined,
        transports: [{ type: "TRANSPORT_GRPC", uri: "http://127.0.0.1:8000", interval: 5, extra: undefined }],
        tomes: [{ tomeID: 102, params: "{}" }],
    };

    it("strips the TRANSPORT_ prefix and assigns stable ids to transports", () => {
        const mapped = mapProfile(record, catalog);
        expect(mapped.transports).toEqual([
            { type: "GRPC", uri: "http://127.0.0.1:8000", interval: 5, extra: undefined, id: "transport-0" },
        ]);
    });

    it("resolves bundled tome references against the tome catalog", () => {
        const mapped = mapProfile(record, catalog);
        expect(mapped.tomes).toHaveLength(1);
        expect(mapped.tomes[0].tomeName).toBe("Persist via Cron");
        expect(mapped.tomes[0].tactic).toBe("PERSISTENCE");
    });

    it("falls back to a generic name when a tome reference isn't in the catalog", () => {
        const mapped = mapProfile({ ...record, tomes: [{ tomeID: 999, params: "{}" }] }, catalog);
        expect(mapped.tomes[0].tomeName).toBe("Tome 999");
        expect(mapped.tomes[0].tactic).toBe("UNSPECIFIED");
    });

    it("flags hasPersistenceTome based on bundled tome tactics", () => {
        expect(mapProfile(record, catalog).hasPersistenceTome).toBe(true);
        expect(mapProfile({ ...record, tomes: [] }, catalog).hasPersistenceTome).toBe(false);
    });
});

describe("profileInput", () => {
    const profile: BuildProfile = {
        id: "1",
        name: "  Stealth DNS  ",
        description: "DNS-only transport",
        transports: [{ id: "t1", type: "DNS", uri: "dns://127.0.0.1:53", interval: 60 }],
        buildImage: "ghcr.io/realm/builder:stealth",
        tomes: [{ id: "tr1", tomeId: "101", tomeName: "Exfil Recon", paramSummary: "No params", params: "{}", tactic: "RECON" }],
        tomeCount: 1,
        hasPersistenceTome: false,
        buildScript: "cargo build --release",
        artifactPath: "target/release/imix.exe",
        prebuildscript: "echo pre",
        setupscript: "echo setup",
        postbuildscript: "echo post",
        unique: "",
    };

    it("trims the name and re-adds the TRANSPORT_ prefix", () => {
        const input = profileInput(profile);
        expect(input.name).toBe("Stealth DNS");
        expect(input.transports).toEqual([{ type: "TRANSPORT_DNS", uri: "dns://127.0.0.1:53", interval: 60, extra: undefined }]);
    });

    it("maps bundled tomes to {tomeID, params} pairs", () => {
        const input = profileInput(profile);
        expect(input.tomes).toEqual([{ tomeID: "101", params: "{}" }]);
    });

    it("omits an empty unique field", () => {
        expect(profileInput(profile).unique).toBeUndefined();
        expect(profileInput({ ...profile, unique: '[{"type":"env"}]' }).unique).toBe('[{"type":"env"}]');
    });
});

function buildTask(overrides: Partial<BuildTask>): BuildTask {
    return {
        id: "task-1",
        targetOs: "PLATFORM_LINUX",
        targetFormat: "TARGET_FORMAT_BIN",
        createdAt: "2024-01-01T00:00:00Z",
        claimedAt: null,
        startedAt: null,
        finishedAt: null,
        exitCode: null,
        error: null,
        profile: { id: "1", name: "Default Imix" },
        profileAtCreation: { name: "Default Imix" },
        artifact: null,
        ...overrides,
    };
}

describe("taskStatus", () => {
    it("reports Queued before a builder claims the task", () => {
        expect(taskStatus(buildTask({}))).toBe("Queued");
    });

    it("reports Claimed once a builder has claimed but not started it", () => {
        expect(taskStatus(buildTask({ claimedAt: "2024-01-01T00:01:00Z" }))).toBe("Claimed");
    });

    it("reports Building once execution has started", () => {
        expect(taskStatus(buildTask({ claimedAt: "t", startedAt: "2024-01-01T00:02:00Z" }))).toBe("Building");
    });

    it("reports Failed on a non-zero exit code", () => {
        expect(taskStatus(buildTask({ startedAt: "t", finishedAt: "t", exitCode: 1 }))).toBe("Failed");
    });

    it("reports Awaiting artifact when finished successfully without an artifact yet", () => {
        expect(taskStatus(buildTask({ startedAt: "t", finishedAt: "t", exitCode: 0 }))).toBe("Awaiting artifact");
    });

    it("reports Built once an artifact is attached", () => {
        expect(taskStatus(buildTask({ startedAt: "t", finishedAt: "t", exitCode: 0, artifact: { id: "a1", name: "build/linux/bin/imix", size: 1024 } }))).toBe("Built");
    });

    // Cargo writes its normal build progress/warnings to stderr, and Tavern
    // persists that as `error` regardless of outcome -- it isn't evidence
    // of failure on its own. exitCode/artifact are the source of truth.
    it("reports Built -- not Failed -- when exitCode is 0 and an artifact exists despite non-empty stderr diagnostics", () => {
        expect(taskStatus(buildTask({
            startedAt: "t",
            finishedAt: "t",
            exitCode: 0,
            error: "warning: unused import `foo`\n   Compiling imix v0.1.0\n    Finished release [optimized] target(s) in 42.17s",
            artifact: { id: "a1", name: "build/linux/bin/imix", size: 1024 },
        }))).toBe("Built");
    });
});

describe("buildTaskRows", () => {
    it("includes one row per task regardless of status, not just Built ones", () => {
        const tasks = [
            buildTask({ id: "queued" }),
            buildTask({ id: "claimed", claimedAt: "t" }),
            buildTask({ id: "building", claimedAt: "t", startedAt: "t" }),
            buildTask({ id: "failed", startedAt: "t", finishedAt: "2024-01-01T00:05:00Z", exitCode: 1, error: "boom" }),
            buildTask({
                id: "built",
                startedAt: "t",
                finishedAt: new Date().toISOString(),
                exitCode: 0,
                artifact: { id: "a1", name: "build/linux/bin/imix", size: 1024 },
            }),
        ];
        const rows = buildTaskRows(tasks);
        expect(rows.map((row) => row.id)).toEqual(["queued", "claimed", "building", "failed", "built"]);
        expect(rows.map((row) => row.status)).toEqual(["Queued", "Claimed", "Building", "Failed", "Built"]);
    });

    it("leaves artifact fields null for in-progress and failed rows", () => {
        const rows = buildTaskRows([buildTask({ id: "queued" }), buildTask({ id: "failed", startedAt: "t", finishedAt: "2024-01-01T00:05:00Z", exitCode: 1, error: "boom" })]);
        for (const row of rows) {
            expect(row.assetId).toBeNull();
            expect(row.assetName).toBeNull();
            expect(row.artifactName).toBeNull();
            expect(row.artifactSize).toBeNull();
        }
        expect(rows[1].error).toBe("boom");
    });

    it("derives download-ready artifact fields for Built rows", () => {
        const rows = buildTaskRows([
            buildTask({
                id: "t2",
                targetOs: "PLATFORM_MACOS",
                targetFormat: "TARGET_FORMAT_BIN",
                startedAt: "t",
                finishedAt: new Date().toISOString(),
                exitCode: 0,
                artifact: { id: "a1", name: "build/macos/bin/imix-calm-raven", size: 1024 * 1024 * 2 },
            }),
        ]);
        expect(rows).toHaveLength(1);
        expect(rows[0]).toMatchObject({
            id: "t2",
            status: "Built",
            assetId: "a1",
            assetName: "build/macos/bin/imix-calm-raven",
            artifactName: "imix-calm-raven",
            artifactSize: "2.00 MB",
            platform: "MACOS",
            format: "BIN",
        });
    });
});
