import { useQuery } from "@apollo/client";
import { useEffect } from "react";
import EmptyStateNoQuests from "../../components/empty-states/EmptyStateNoQuests";
import { PageWrapper } from "../../components/page-wrapper"
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { PageNavItem } from "../../utils/enums";
import { GET_HOST_QUERY, GET_TASK_QUERY } from "../../utils/queries";
import OverviewChartWrapper from "./components/OverviewChartWrapper";


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

    useEffect(() => {
        refetch();
    }, []);


    function getOverviewWrapper() {
        if (loading || hostLoading) {
            return <EmptyState type={EmptyStateType.loading} label="Loading dashboard data..." />
        }

        if (error || hostError) {
            return <EmptyState type={EmptyStateType.error} label="Error loading dashboard data..." />
        }

        if (data?.tasks?.totalCount === 0) {
            return <EmptyStateNoQuests />
        }

        return (
            <OverviewChartWrapper data={data?.tasks?.edges} hosts={hosts.hosts || []} />
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
