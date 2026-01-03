import { useQuery } from "@apollo/client";
import { useMemo } from "react";
import { GET_TASK_QUERY, GET_HOST_QUERY } from "../../../utils/queries";
import { useQuestData } from "./useQuestData";
import { useHostData } from "./useHostData";
import { UseDashboardDataReturn } from "../types";
import { TaskQueryTopLevel, HostQueryTopLevel } from "../../../utils/interfacesQuery";

export const useDashboardData = (): UseDashboardDataReturn => {
    const {
        loading: taskLoading,
        error: taskError,
        data: taskData
    } = useQuery<TaskQueryTopLevel>(GET_TASK_QUERY, {
        variables: {
            orderBy: [{
                direction: "ASC",
                field: "CREATED_AT"
            }],
        },
        notifyOnNetworkStatusChange: true,
    });

    const {
        loading: hostLoading,
        data: hostData,
        error: hostError
    } = useQuery<HostQueryTopLevel>(GET_HOST_QUERY, {
        notifyOnNetworkStatusChange: true,
    });

    const { loading: questFormatLoading, formattedData } = useQuestData(taskData);
    const {
        loading: hostFormatLoading,
        hostActivity,
        onlineHostCount,
        offlineHostCount
    } = useHostData(hostData);

    const loading = taskLoading || hostLoading || questFormatLoading || hostFormatLoading;
    const error = taskError || hostError;

    const dashboardData = useMemo(() => ({
        questData: {
            formattedData,
            hosts: hostData?.hosts?.edges || [],
            loading: taskLoading,
        },
        hostData: {
            hostActivity,
            onlineHostCount,
            offlineHostCount,
            loading: hostFormatLoading,
        },
        raw: {
            tasks: taskData,
            hosts: hostData,
        }
    }), [
        formattedData,
        hostData,
        taskLoading,
        hostActivity,
        onlineHostCount,
        offlineHostCount,
        hostFormatLoading,
        taskData
    ]);

    return {
        loading,
        error,
        data: dashboardData,
        hasTaskData: !!(taskData?.tasks?.edges?.length),
        hasHostData: !!(hostData?.hosts?.edges?.length),
    };
};
