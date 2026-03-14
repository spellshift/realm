import EmptyStateNoQuests from "../../components/empty-states/EmptyStateNoQuests";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import QuestCard from "./components/QuestCard";
import AccessCard from "./components/AccessCard";
import Breadcrumbs from "../../components/Breadcrumbs";
import Button from "../../components/tavern-base-ui/button/Button";
import PageHeader from "../../components/tavern-base-ui/PageHeader";
import { useDashboardData } from "./hook/useDashboardData";
import { useCreateQuestModal } from "../../context/CreateQuestModalContext";
import { UseDashboardDataReturn } from "./types";
import { FileTerminal } from "lucide-react";

export const Dashboard = () => {
    const { loading, error, data, hasTaskData }: UseDashboardDataReturn = useDashboardData();
    const { openModal } = useCreateQuestModal();

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
        <>
            <div className="flex flex-row justify-between w-full items-center">
                <Breadcrumbs pages={[{
                    label: "Dashboard",
                    link: "/dashboard"
                }]} />
                <div>
                    <Button
                        leftIcon={<FileTerminal className="w-5 h-5" />}
                        buttonStyle={{ color: "purple", size: "md" }}
                        onClick={() => openModal({navigateToQuest: true})}
                    >
                        Create a quest
                    </Button>
                </div>
            </div>
            <PageHeader title="Dashboard" />
            {getOverviewWrapper()}
        </>
    );
}
