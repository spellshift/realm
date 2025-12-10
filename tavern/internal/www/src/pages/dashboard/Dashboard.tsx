import EmptyStateNoQuests from "../../components/empty-states/EmptyStateNoQuests";
import { PageWrapper } from "../../components/page-wrapper"
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { PageNavItem } from "../../utils/enums";
import QuestCard from "./components/QuestCard";
import AccessCard from "./components/AccessCard";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";
import { useDashboardData } from "./hook/useDashboardData";
import { UseDashboardDataReturn } from "./types";

export const Dashboard = () => {
    const { loading, error, data, hasTaskData }: UseDashboardDataReturn = useDashboardData();

    function getOverviewWrapper() {
        if (loading) {
            return <EmptyState type={EmptyStateType.loading} label="Loading dashboard data..." />
        }

        if (error) {
            return <EmptyState type={EmptyStateType.error} label="Error loading dashboard data..." />
        }

        if (!hasTaskData) {
            return <EmptyStateNoQuests />
        }

        return (
            <div className="my-4 flex flex-col gap-4">
                <QuestCard
                    formattedData={data.questData.formattedData}
                    hosts={data.questData.hosts}
                    loading={data.questData.loading}
                />
                <AccessCard
                    hostActivity={data.hostData.hostActivity}
                    onlineHostCount={data.hostData.onlineHostCount}
                    offlineHostCount={data.hostData.offlineHostCount}
                    loading={data.hostData.loading}
                />
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
