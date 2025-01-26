import { useQuery } from "@apollo/client";
import React, { useEffect } from "react";

import EmptyStateNoQuests from "../../components/empty-states/EmptyStateNoQuests";
import { PageWrapper } from "../../components/page-wrapper"
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { PageNavItem } from "../../utils/enums";
import { GET_HOST_QUERY, GET_TASK_QUERY } from "../../utils/queries";

import { useOverviewData } from "./hook/useOverviewData";
import QuestCard from "./components/QuestCard";
import AccessCard from "./components/AccessCard";


export const Dashboard = () => {
    const { loading, error, data, refetch } = useQuery(GET_TASK_QUERY, {
        variables: {
            "orderBy": [{
                "direction": "ASC",
                "field": "CREATED_AT"
            }]
        },
        notifyOnNetworkStatusChange: true
    });

    const { loading: hostLoading, data: hosts, error: hostError } = useQuery(GET_HOST_QUERY);

    const { loading: formatLoading, formattedData } = useOverviewData(data);

    useEffect(() => {
        refetch();
    }, [refetch]);


    function getOverviewWrapper() {
        if (loading || hostLoading || formatLoading) {
            return <EmptyState type={EmptyStateType.loading} label="Loading dashboard data..." />
        }

        if (error || hostError) {
            return <EmptyState type={EmptyStateType.error} label="Error loading dashboard data..." />
        }

        if (!data || !data.tasks || !data.tasks?.edges) {
            return <EmptyStateNoQuests />
        }

        return (
            <div className="my-4 flex flex-col gap-4">
                <QuestCard formattedData={formattedData} hosts={hosts?.hosts || []} loading={loading} />
                <AccessCard hosts={hosts?.hosts || []} />
            </div>
        )
    }

    return (
        <PageWrapper currNavItem={PageNavItem.dashboard}>
            <div className="border-b border-gray-200 pb-6 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Dashboard</h3>
            </div>
            {getOverviewWrapper()}
        </PageWrapper>
    );
}
