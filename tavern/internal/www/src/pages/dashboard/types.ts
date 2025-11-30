import { HostEdge, HostQueryTopLevel, TaskQueryTopLevel } from "../../utils/interfacesQuery";

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
    hosts: HostEdge[];
    loading: boolean;
}

export interface DashboardHostData {
    hostActivity: HostActivityByKind;
    onlineHostCount: number;
    offlineHostCount: number;
    loading: boolean;
}

export interface DashboardRawData {
    tasks: TaskQueryTopLevel | undefined;
    hosts: HostQueryTopLevel | undefined;
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
    hasHostData: boolean;
}
