import { gql } from "@apollo/client";
import { UserNode } from "./interfacesQuery";

// Build-task status derivation shared by the Built agents tab and the Assets table.

export type BuildTaskStatus = "Queued" | "Claimed" | "Building" | "Awaiting artifact" | "Built" | "Failed";

export const GET_BUILD_TASKS = gql`
    query AgentBuildTasks {
        buildTasks(orderBy: [{ field: CREATED_AT, direction: DESC }]) {
            edges { node {
                id targetOs targetFormat createdAt claimedAt startedAt finishedAt exitCode error
                profile { id name }
                profileAtCreation { name }
                artifact { id name size hash status creator { id name photoURL } }
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
    profile: { id: string; name: string } | null;
    profileAtCreation: { name: string } | null;
    artifact: { id: string; name: string; size: number; hash: string; status: "FINISHED" | "IN_PROGRESS" | "ERROR"; creator?: UserNode | null } | null;
};

// Error text DockerExecutor.Build returns, with exit code 0, when artifact extraction fails.
const EXTRACTION_FAILURE_TEXT = "artifact extraction failed";

// How long a finished build may wait for its artifact upload before it counts as failed.
const AWAITING_ARTIFACT_TIMEOUT_MS = 5 * 60 * 1000;

// CreateBuildTask attaches a pending placeholder asset; only a FINISHED one has been uploaded.
export function uploadedArtifact(task: BuildTask): BuildTask["artifact"] {
    return task.artifact?.status === "FINISHED" ? task.artifact : null;
}

export function taskStatus(task: BuildTask, now: number = Date.now()): BuildTaskStatus {
    if (task.finishedAt) {
        // `error` is stderr and can be non-empty on success, so exitCode is the source of truth.
        if (task.exitCode !== 0) return "Failed";
        if (uploadedArtifact(task)) return "Built";
        if (task.artifact?.status === "ERROR") return "Failed";
        if (task.error?.includes(EXTRACTION_FAILURE_TEXT)) return "Failed";
        const elapsed = now - new Date(task.finishedAt).getTime();
        return elapsed > AWAITING_ARTIFACT_TIMEOUT_MS ? "Failed" : "Awaiting artifact";
    }
    if (task.startedAt) return "Building";
    return task.claimedAt ? "Claimed" : "Queued";
}
