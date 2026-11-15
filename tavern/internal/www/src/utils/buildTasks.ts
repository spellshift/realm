import { gql } from "@apollo/client";
import { UserNode } from "./interfacesQuery";

// Build-task status derivation shared by the Built agents tab and the Assets
// table, so both pages always agree on a build's state.

export type BuildTaskStatus = "Queued" | "Claimed" | "Building" | "Awaiting artifact" | "Built" | "Failed";

export const GET_BUILD_TASKS = gql`
    query AgentBuildTasks {
        buildTasks(orderBy: [{ field: CREATED_AT, direction: DESC }]) {
            edges { node {
                id targetOs targetFormat createdAt claimedAt startedAt finishedAt exitCode error
                profile { id name }
                profileAtCreation { name }
                artifact { id name size hash creator { id name photoURL } }
            } }
        }
    }
`;

export type BuildTask = {
    id: string;
    targetOs: string;
    targetFormat: string;
    createdAt: string;
    claimedAt: string | null;
    startedAt: string | null;
    finishedAt: string | null;
    exitCode: number | null;
    error: string | null;
    profile: { id: string; name: string };
    profileAtCreation: { name: string } | null;
    artifact: { id: string; name: string; size: number; hash: string; creator?: UserNode | null } | null;
};

// DockerExecutor.Build (tavern/internal/builder/executor/docker.go) reports a
// failed artifact extraction by returning an error containing this text
// alongside a successful (0) exit code, specifically so a successful build
// with a missing artifact isn't mistaken for one still in progress.
const EXTRACTION_FAILURE_TEXT = "artifact extraction failed";

// A finished task with exitCode 0 and no artifact yet is normally just the
// brief gap before the builder's separate UploadBuildArtifact call lands --
// but if extraction failed, or the upload itself silently never arrives
// (the builder only logs that failure locally), no artifact is ever coming.
// Treat either case, or a gap that's stretched well past that window, as
// terminal so the row doesn't spin forever.
const AWAITING_ARTIFACT_TIMEOUT_MS = 5 * 60 * 1000;

// CreateBuildTask eagerly attaches an empty placeholder asset so the build is
// visible in the Assets list while it runs; UploadBuildArtifact fills it in
// (and rejects empty uploads), so a zero-size artifact is not yet uploaded.
export function uploadedArtifact(task: BuildTask): BuildTask["artifact"] {
    return task.artifact && task.artifact.size > 0 ? task.artifact : null;
}

export function taskStatus(task: BuildTask, now: number = Date.now()): BuildTaskStatus {
    if (task.finishedAt) {
        // `error` holds whatever the build container wrote to stderr, which
        // for a normal cargo build includes non-fatal progress/warnings --
        // it isn't reliable evidence of failure on its own. exitCode (and
        // whether an artifact was produced) is the source of truth; a
        // successful build can still have populated `error` text.
        if (task.exitCode !== 0) return "Failed";
        if (uploadedArtifact(task)) return "Built";
        if (task.error?.includes(EXTRACTION_FAILURE_TEXT)) return "Failed";
        const elapsed = now - new Date(task.finishedAt).getTime();
        return elapsed > AWAITING_ARTIFACT_TIMEOUT_MS ? "Failed" : "Awaiting artifact";
    }
    if (task.startedAt) return "Building";
    return task.claimedAt ? "Claimed" : "Queued";
}
