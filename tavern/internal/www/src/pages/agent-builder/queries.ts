import { gql, useQuery } from "@apollo/client";
import { BuildTask, GET_BUILD_TASKS, taskStatus, uploadedArtifact } from "../../utils/buildTasks";
import { formatDistanceToNow } from "date-fns";
import { UserNode } from "../../utils/interfacesQuery";
import { Builder, BuilderHealthy, BuildProfile, BuildTaskRow, Tome, TransportConfig, Platform, Format } from "./types";

export const PROFILE_FIELDS = gql`
    fragment BuildProfileFields on BuildProfile {
        id name description buildImage buildScript artifactPath prebuildscript setupscript postbuildscript unique
        transports { uri interval type extra jitter }
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
        builders { edges { node { id identifier hostname supportedTargets upstream pollInterval lastSeenAt } } }
    }
`;
export const GET_ENROLLED_BUILDER = gql`
    query EnrolledAgentBuilder($identifier: String!) {
        builders(where: { identifier: $identifier }) { edges { node { id identifier hostname supportedTargets upstream pollInterval lastSeenAt } } }
    }
`;
export const GET_ONLINE_BUILDER_PLATFORMS = gql`
    query OnlineAgentBuilderPlatforms {
        onlineBuilderPlatforms
    }
`;
export const CREATE_BUILD_PROFILE = gql`
    ${PROFILE_FIELDS}
    mutation CreateAgentBuildProfile($input: CreateBuildProfileInput!) {
        createBuildProfile(input: $input) { ...BuildProfileFields }
    }
`;
export const UPDATE_BUILD_PROFILE = gql`
    ${PROFILE_FIELDS}
    mutation UpdateAgentBuildProfile($profileID: ID!, $input: UpdateBuildProfileInput!) {
        updateBuildProfile(profileID: $profileID, input: $input) { ...BuildProfileFields }
    }
`;
export const DELETE_BUILD_PROFILE = gql`
    mutation DeleteAgentBuildProfile($profileID: ID!) {
        deleteBuildProfile(profileID: $profileID)
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

export type ProfileRecord = Omit<BuildProfile, "transports" | "tomes" | "tomeCount" | "hasPersistenceTome"> & {
    transports: Array<Omit<TransportConfig, "id" | "type"> & { type: string }>;
    tomes: Array<{ tomeID: number; params: string }> | null;
};
export { GET_BUILD_TASKS, taskStatus };
export type { BuildTask };
export type BuilderRecord = Omit<Builder, "supportedTargets" | "lastSeenAtRaw" | "online"> & { supportedTargets: string[] };

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
        transports: (profile.transports ?? []).map((transport, index) => ({
            ...transport,
            id: `transport-${index}`,
            type: transport.type as TransportConfig["type"],
        })),
        tomes,
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
        transports: profile.transports.map(({ type, uri, interval, extra, jitter }) => ({ type, uri, interval, extra, jitter })),
        tomes: profile.tomes.map(({ tomeId, params }) => ({ tomeID: tomeId, params })),
    };
}

// Every BuildTask becomes a row -- queued/claimed/building tasks render
// with an in-progress status and no artifact fields, failed tasks carry
// their error, and only `Built` rows have a downloadable asset attached.
export function buildTaskRows(tasks: BuildTask[]): BuildTaskRow[] {
    const now = Date.now();
    return tasks.map((task) => {
        const status = taskStatus(task, now);
        const artifact = uploadedArtifact(task);
        const timestamp = formatDistanceToNow(new Date(task.finishedAt ?? task.createdAt), { addSuffix: true });
        return {
            id: task.id,
            status,
            profileName: task.profileAtCreation?.name ?? task.profile?.name ?? "Deleted profile",
            platform: task.targetOs as Platform,
            format: task.targetFormat as Format,
            error: task.error ?? (status === "Failed" && task.exitCode === 0 ? "The build finished but no artifact was produced." : null),
            exitCode: task.exitCode,
            creator: (artifact?.creator as UserNode) ?? null,
            timestamp,
            assetId: artifact?.id ?? null,
            assetName: artifact?.name ?? null,
            artifactName: artifact ? artifact.name.split("/").pop()! : null,
            artifactSize: artifact ? `${(artifact.size / 1024 / 1024).toFixed(2)} MB` : null,
            artifactHash: artifact ? artifact.hash : null,
        };
    });
}

export function useBuildProfiles() {
    const result = useQuery<{ buildProfiles: Connection<ProfileRecord>; tomes: Connection<Tome> }>(GET_BUILD_PROFILES, { fetchPolicy: "cache-and-network" });
    const tomes = nodes(result.data?.tomes);
    return { ...result, tomes, profiles: nodes(result.data?.buildProfiles).map((profile) => mapProfile(profile, tomes)) };
}

export function useBuilders(options: { skip?: boolean } = {}) {
    const { skip = false } = options;
    const result = useQuery<{ builders: Connection<BuilderRecord> }>(GET_BUILDERS, {
        skip,
        fetchPolicy: "cache-and-network",
        pollInterval: skip ? undefined : 30000,
    });
    const now = Date.now();
    const builders = nodes(result.data?.builders).map((builder) => mapBuilder(builder, now));
    return { ...result, builders };
}

function mapBuilder(builder: BuilderRecord, now: number = Date.now()): Builder {
    return {
        ...builder,
        supportedTargets: builder.supportedTargets.map((target) => target as Platform),
        lastSeenAtRaw: builder.lastSeenAt,
        online: BuilderHealthy(builder.lastSeenAt, builder.pollInterval, now),
        lastSeenAt: builder.lastSeenAt ? formatDistanceToNow(new Date(builder.lastSeenAt), { addSuffix: true }) : null,
    };
}

export function useOnlineBuilderPlatforms() {
    const result = useQuery<{ onlineBuilderPlatforms: Platform[] }>(GET_ONLINE_BUILDER_PLATFORMS, {
        fetchPolicy: "network-only",
    });
    return { ...result, platforms: result.loading || result.error ? [] : result.data?.onlineBuilderPlatforms ?? [] };
}

export function useEnrolledBuilder(identifier: string | null, skip: boolean) {
    const result = useQuery<{ builders: Connection<BuilderRecord> }>(
        GET_ENROLLED_BUILDER,
        { variables: { identifier: identifier ?? "" }, skip: skip || !identifier, fetchPolicy: "network-only" }
    );
    const enrolled = nodes(result.data?.builders).find((builder) => builder.identifier === identifier);
    const builder = !result.loading && !result.error && enrolled ? mapBuilder(enrolled) : null;
    return { ...result, builder, online: !!builder?.online };
}

export function useBuildTasks(options: { skip?: boolean } = {}) {
    const { skip = false } = options;
    const result = useQuery<{ buildTasks: Connection<BuildTask> }>(GET_BUILD_TASKS, {
        skip,
        fetchPolicy: "cache-and-network",
        pollInterval: 5000,
    });
    const tasks = nodes(result.data?.buildTasks);
    const rows = buildTaskRows(tasks);
    return { ...result, tasks, rows };
}

export function downloadBuilderConfig(config: string, identifier: string) {
    const url = URL.createObjectURL(new Blob([config], { type: "application/yaml" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = `builder-${identifier}.yaml`;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 0);
}
