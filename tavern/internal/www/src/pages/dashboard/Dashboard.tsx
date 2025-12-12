import { useQuery } from "@apollo/client";
import React, { useEffect } from "react";

import EmptyStateNoQuests from "../../components/empty-states/EmptyStateNoQuests";
import { PageWrapper } from "../../features/page-wrapper"
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { PageNavItem } from "../../utils/enums";
import { GET_HOST_QUERY, GET_TASK_QUERY } from "../../utils/queries";

import { useOverviewData } from "./hook/useOverviewData";
import QuestCard from "./components/QuestCard";
import AccessCard from "./components/AccessCard";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";
import { HostType } from "../../utils/consts";


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

    const { loading: hostLoading, data: unFormattedHosts, error: hostError } = useQuery(GET_HOST_QUERY);
    const hosts = unFormattedHosts?.hosts?.edges?.map((edge: { node: HostType }) => edge?.node);

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
                <QuestCard formattedData={formattedData} hosts={hosts || []} loading={loading} />
                <AccessCard hosts={hosts || []} />
            </div>
        )
    }

    return (
        <PageWrapper currNavItem={PageNavItem.dashboard}>
            <Breadcrumbs pages={[{
                label: "Dashboard",
                link: "/dashboard"
            }]} />
            <PageHeader title="Dashboard" />
            {getOverviewWrapper()}
        </PageWrapper>
    );
}
