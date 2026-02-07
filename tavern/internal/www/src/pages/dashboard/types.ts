import { HostEdge, HostQueryTopLevel, TaskQueryTopLevel } from "../../utils/interfacesQuery";

export interface DashboardHostMetric {
    tag: string;
    tagID: string;
    online: number;
    total: number;
    hostsOnline: number;
    hostsTotal: number;
    lastSeenAt?: string;
}

export interface DashboardHostMetrics {
    group: DashboardHostMetric[];
    service: DashboardHostMetric[];
    platform: DashboardHostMetric[];
    onlineHostCount: number;
    offlineHostCount: number;
    totalHostCount: number;
}

export interface DashboardQuestMetric {
    name: string;
    tasksError: number;
    tasksNoError: number;
    id: string;
}

export interface DashboardTacticCount {
    tactic: string;
    count: number;
}

export interface DashboardTimelineItem {
    label: string;
    timestamp: string;
    taskCreated: number;
    tactics: DashboardTacticCount[];
}

export interface DashboardQuestMetrics {
    tomeUsage: DashboardQuestMetric[];
    taskTimeline: DashboardTimelineItem[];
    taskTactics: string[];
    groupUsage: DashboardQuestMetric[];
    serviceUsage: DashboardQuestMetric[];
    totalQuests: number;
    totalOutput: number;
    totalTasks: number;
    totalErrors: number;
}

export interface DashboardQueryResponse {
    dashboard: {
        hostMetrics: DashboardHostMetrics;
        questMetrics: DashboardQuestMetrics;
    }
}

// UI Types
export interface TaskTimelineItem {
    label: string;
    timestamp: Date;
    taskCreated: number;
    [tactic: string]: number | string | Date;
}

export interface TagUsageItem {
    name: string;
    taskError: number;
    taskNoError: number;
    id: string;
}

export interface TomeUsageItem {
    name: string;
    taskError: number;
    taskNoError: number;
    id: string;
}

export interface QuestFormattedData {
    tomeUsage: TomeUsageItem[];
    taskTimeline: TaskTimelineItem[];
    taskTactics: string[];
    groupUsage: TagUsageItem[];
    serviceUsage: TagUsageItem[];
    totalQuests: number;
    totalOutput: number;
    totalTasks: number;
    totalErrors: number;
}

export interface HostActivityItem {
    tagId: string;
    tag: string;
    online: number;
    total: number;
    lastSeenAt: string | undefined | null;
    hostsOnline: number;
    hostsTotal: number;
}

export interface HostActivityByKind {
    group: HostActivityItem[];
    service: HostActivityItem[];
    platform: HostActivityItem[];
}

export interface DashboardQuestData {
    formattedData: QuestFormattedData;
    hosts: HostEdge[]; // Kept for compatibility if used, otherwise empty
    loading: boolean;
}

export interface DashboardHostData {
    hostActivity: HostActivityByKind;
    onlineHostCount: number;
    offlineHostCount: number;
    loading: boolean;
}

export interface DashboardRawData {
    tasks: TaskQueryTopLevel | undefined; // Deprecated/Unused?
    hosts: HostQueryTopLevel | undefined; // Deprecated/Unused?
}

export interface DashboardData {
    questData: DashboardQuestData;
    hostData: DashboardHostData;
    raw: DashboardRawData;
}

export interface UseDashboardDataReturn {
    loading: boolean;
    error: any;
    data: DashboardData;
    hasTaskData: boolean;
    hasHostData: boolean; // Maybe rename to hasData?
}
