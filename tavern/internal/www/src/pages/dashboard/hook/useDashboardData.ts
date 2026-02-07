import { useQuery } from "@apollo/client";
import { useMemo } from "react";
import { GET_DASHBOARD_QUERY } from "../../../utils/queries";
import { UseDashboardDataReturn, DashboardQueryResponse, TaskTimelineItem, HostActivityItem } from "../types";
import { TaskChartKeys } from "../../../utils/enums";

export const useDashboardData = (): UseDashboardDataReturn => {
    const {
        loading,
        error,
        data
    } = useQuery<DashboardQueryResponse>(GET_DASHBOARD_QUERY, {
        notifyOnNetworkStatusChange: true,
    });

    const dashboardData = useMemo(() => {
        if (!data?.dashboard) {
            return {
                questData: {
                    formattedData: {
                        tomeUsage: [],
                        taskTimeline: [],
                        taskTactics: [],
                        groupUsage: [],
                        serviceUsage: [],
                        totalQuests: 0,
                        totalOutput: 0,
                        totalTasks: 0,
                        totalErrors: 0
                    },
                    hosts: [],
                    loading: loading,
                },
                hostData: {
                    hostActivity: { group: [], service: [], platform: [] },
                    onlineHostCount: 0,
                    offlineHostCount: 0,
                    loading: loading,
                },
                raw: {
                    tasks: undefined,
                    hosts: undefined,
                }
            };
        }

        const { hostMetrics, questMetrics } = data.dashboard;

        // Transform Host Metrics
        const mapHostMetrics = (metrics: any[]): HostActivityItem[] => metrics.map(m => ({
            tag: m.tag,
            tagId: m.tagID,
            online: m.online,
            total: m.total,
            hostsOnline: m.hostsOnline,
            hostsTotal: m.hostsTotal,
            lastSeenAt: m.lastSeenAt
        }));

        const hostActivity = {
            group: mapHostMetrics(hostMetrics.group),
            service: mapHostMetrics(hostMetrics.service),
            platform: mapHostMetrics(hostMetrics.platform),
        };

        // Transform Quest Metrics
        const mapQuestMetrics = (metrics: any[]) => metrics.map(m => ({
            name: m.name,
            [TaskChartKeys.taskError]: m.tasksError,
            [TaskChartKeys.taskNoError]: m.tasksNoError,
            id: m.id
        }));

        const taskTimeline: TaskTimelineItem[] = questMetrics.taskTimeline.map((item: any) => {
            const tacticsObj = item.tactics.reduce((acc: any, curr: any) => ({ ...acc, [curr.tactic]: curr.count }), {});
            return {
                label: item.label,
                timestamp: new Date(item.timestamp),
                [TaskChartKeys.taskCreated]: item.taskCreated,
                ...tacticsObj
            };
        });

        const formattedData: any = {
            tomeUsage: mapQuestMetrics(questMetrics.tomeUsage),
            taskTimeline,
            taskTactics: questMetrics.taskTactics,
            groupUsage: mapQuestMetrics(questMetrics.groupUsage),
            serviceUsage: mapQuestMetrics(questMetrics.serviceUsage),
            totalQuests: questMetrics.totalQuests,
            totalOutput: questMetrics.totalOutput,
            totalTasks: questMetrics.totalTasks,
            totalErrors: questMetrics.totalErrors,
        };

        return {
            questData: {
                formattedData,
                hosts: [],
                loading: loading,
            },
            hostData: {
                hostActivity,
                onlineHostCount: hostMetrics.onlineHostCount,
                offlineHostCount: hostMetrics.offlineHostCount,
                loading: loading,
            },
            raw: {
                tasks: undefined,
                hosts: undefined,
            }
        };

    }, [data, loading]);

    return {
        loading,
        error,
        data: dashboardData,
        hasTaskData: !!(data?.dashboard?.questMetrics?.totalTasks),
        hasHostData: !!(data?.dashboard?.hostMetrics?.totalHostCount),
    };
};
