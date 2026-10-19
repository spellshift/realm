import { gql, useQuery } from "@apollo/client";
import { formatDistanceToNow } from "date-fns";
import { Builder, BuildProfile, BuildTaskRow, COMPILE_TARGETS, Tome, TransportConfig, Platform, Format } from "./types";

export const PROFILE_FIELDS = gql`
    fragment BuildProfileFields on BuildProfile {
        id name description buildImage buildScript artifactPath prebuildscript setupscript postbuildscript unique
        transports { uri interval type extra }
        tomes { tomeID params }
    }
`;
export const GET_BUILD_PROFILES = gql`
    ${PROFILE_FIELDS}
    query AgentBuildProfiles {
        buildProfiles { edges { node { ...BuildProfileFields } } }
        tomes { edges { node { id name description tactic paramDefs } } }
    }
`;
export const GET_BUILDERS = gql`
    query AgentBuilders {
        builders { edges { node { id identifier supportedTargets upstream pollInterval lastSeenAt } } }
    }
`;
export const GET_BUILD_TASKS = gql`
    query AgentBuildTasks {
        buildTasks(orderBy: [{ field: CREATED_AT, direction: DESC }]) {
            edges { node {
                id targetOs targetFormat createdAt claimedAt startedAt finishedAt exitCode error
                profile { id name }
                profileAtCreation { name }
                artifact { id name size }
            } }
        }
    }
`;
export const CREATE_BUILD_PROFILE = gql`
    ${PROFILE_FIELDS}
    mutation CreateAgentBuildProfile($input: CreateBuildProfileInput!) {
        createBuildProfile(input: $input) { ...BuildProfileFields }
    }
`;
export const CREATE_BUILD_TASK = gql`
    mutation CreateAgentBuildTask($input: CreateBuildTaskInput!) {
        createBuildTask(input: $input) { id }
    }
`;
export const REGISTER_BUILDER = gql`
    mutation EnrollAgentBuilder($input: CreateBuilderInput!) {
        registerBuilder(input: $input) { builder { id identifier } config }
    }
`;

type Connection<T> = { edges: Array<{ node: T | null } | null> | null };
export const nodes = <T,>(connection?: Connection<T>): T[] =>
    connection?.edges?.flatMap((edge) => edge?.node ? [edge.node] : []) ?? [];

export type ProfileRecord = Omit<BuildProfile, "transports" | "tomes" | "platforms" | "supportedTargets" | "tomeCount" | "hasPersistenceTome"> & {
    transports: Array<Omit<TransportConfig, "id" | "type"> & { type: string }>;
    tomes: Array<{ tomeID: number; params: string }> | null;
};
export type BuilderRecord = Omit<Builder, "supportedTargets"> & { supportedTargets: string[] };
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
    artifact: { id: string; name: string; size: number } | null;
};

export function mapProfile(profile: ProfileRecord, catalog: Tome[]): BuildProfile {
    const tomes = (profile.tomes ?? []).map((reference, index) => {
        const tome = catalog.find((item) => item.id === String(reference.tomeID));
        return {
            id: `${reference.tomeID}-${index}`,
            tomeId: String(reference.tomeID),
            tomeName: tome?.name ?? `Tome ${reference.tomeID}`,
            tactic: tome?.tactic ?? "UNSPECIFIED" as Tome["tactic"],
            params: reference.params,
            paramSummary: reference.params === "{}" ? "No params" : reference.params,
        };
    });
    return {
        ...profile,
        transports: profile.transports.map((transport, index) => ({
            ...transport,
            id: `transport-${index}`,
            type: transport.type.replace(/^TRANSPORT_/, "") as TransportConfig["type"],
        })),
        tomes,
        platforms: ["LINUX", "MACOS", "WINDOWS"],
        supportedTargets: COMPILE_TARGETS.map((target) => target.id),
        tomeCount: tomes.length,
        hasPersistenceTome: tomes.some((tome) => tome.tactic === "PERSISTENCE"),
    };
}

export function profileInput(profile: BuildProfile) {
    return {
        name: profile.name.trim(), description: profile.description, buildImage: profile.buildImage.trim(),
        buildScript: profile.buildScript, artifactPath: profile.artifactPath,
        prebuildscript: profile.prebuildscript, setupscript: profile.setupscript, postbuildscript: profile.postbuildscript,
        unique: profile.unique || undefined,
        transports: profile.transports.map(({ type, uri, interval, extra }) => ({ type: `TRANSPORT_${type}`, uri, interval, extra })),
        tomes: profile.tomes.map(({ tomeId, params }) => ({ tomeID: tomeId, params })),
    };
}

export function taskStatus(task: BuildTask): BuildTaskRow["status"] {
    if (task.finishedAt) {
        // `error` holds whatever the build container wrote to stderr, which
        // for a normal cargo build includes non-fatal progress/warnings --
        // it isn't reliable evidence of failure on its own. exitCode (and
        // whether an artifact was produced) is the source of truth; a
        // successful build can still have populated `error` text.
        if (task.exitCode !== 0) return "Failed";
        return task.artifact ? "Built" : "Awaiting artifact";
    }
    if (task.startedAt) return "Building";
    return task.claimedAt ? "Claimed" : "Queued";
}

// Every BuildTask becomes a row -- queued/claimed/building tasks render
// with an in-progress status and no artifact fields, failed tasks carry
// their error, and only `Built` rows have a downloadable asset attached.
export function buildTaskRows(tasks: BuildTask[]): BuildTaskRow[] {
    return tasks.map((task) => {
        const status = taskStatus(task);
        const timestamp = formatDistanceToNow(new Date(task.finishedAt ?? task.createdAt), { addSuffix: true });
        return {
            id: task.id,
            status,
            profileName: task.profileAtCreation?.name ?? task.profile.name,
            platform: task.targetOs.replace(/^PLATFORM_/, "") as Platform,
            format: task.targetFormat.replace(/^TARGET_FORMAT_/, "") as Format,
            error: task.error,
            timestamp,
            assetId: task.artifact?.id ?? null,
            assetName: task.artifact?.name ?? null,
            artifactName: task.artifact ? task.artifact.name.split("/").pop()! : null,
            artifactSize: task.artifact ? `${(task.artifact.size / 1024 / 1024).toFixed(2)} MB` : null,
        };
    });
}

export function useBuildProfiles() {
    const result = useQuery<{ buildProfiles: Connection<ProfileRecord>; tomes: Connection<Tome> }>(GET_BUILD_PROFILES, { fetchPolicy: "cache-and-network" });
    const tomes = nodes(result.data?.tomes);
    return { ...result, tomes, profiles: nodes(result.data?.buildProfiles).map((profile) => mapProfile(profile, tomes)) };
}

export function useBuilders(skip = false) {
    const result = useQuery<{ builders: Connection<BuilderRecord> }>(GET_BUILDERS, { skip, pollInterval: 5000, fetchPolicy: "cache-and-network" });
    const builders: Builder[] = nodes(result.data?.builders).map((builder) => ({
        ...builder,
        supportedTargets: builder.supportedTargets.map((target) => target.replace(/^PLATFORM_/, "") as Platform),
        lastSeenAt: builder.lastSeenAt ? formatDistanceToNow(new Date(builder.lastSeenAt), { addSuffix: true }) : null,
    }));
    return { ...result, builders };
}

export function useBuildTasks() {
    const result = useQuery<{ buildTasks: Connection<BuildTask> }>(GET_BUILD_TASKS, { pollInterval: 3000, fetchPolicy: "cache-and-network" });
    const tasks = nodes(result.data?.buildTasks);
    return { ...result, tasks, rows: buildTaskRows(tasks) };
}

export function downloadBuilderConfig(config: string, identifier: string) {
    const url = URL.createObjectURL(new Blob([config], { type: "application/yaml" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = `builder-${identifier}.yaml`;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 0);
}
