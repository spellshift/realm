import Breadcrumbs from "../../components/Breadcrumbs";
import { CreateQuestDropdown } from "../../components/create-quest-modal/CreateQuestDropdown";
import { QuestSummaryCard } from "./QuestSummaryCard";
import { BeaconSummaryCard } from "./BeaconSummaryCard";
import { TagSummaryCard } from "./TagSummaryCard";
import { ErrorBoundary } from "../../components/ErrorBoundary";

export const Dashboard = () => {
    return (
        <>
            <div className="flex flex-row justify-between w-full items-center">
                <Breadcrumbs pages={[{
                    label: "Dashboard",
                    link: "/dashboard"
                }]} />
                <div>
                    <CreateQuestDropdown />
                </div>
            </div>
            <div className="flex flex-col gap-2">
                <ErrorBoundary><QuestSummaryCard /></ErrorBoundary>
                <ErrorBoundary><BeaconSummaryCard /></ErrorBoundary>
                <ErrorBoundary><TagSummaryCard /></ErrorBoundary>
            </div>
        </>
    );
};
