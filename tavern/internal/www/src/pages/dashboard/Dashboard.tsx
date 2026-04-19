import Breadcrumbs from "../../components/Breadcrumbs";
import Button from "../../components/tavern-base-ui/button/Button";
import { FileTerminal } from "lucide-react";
import { useCreateQuestModal } from "../../context/CreateQuestModalContext";
import { QuestSummaryCard } from "./QuestSummaryCard";
import { BeaconSummaryCard } from "./BeaconSummaryCard";

export const Dashboard = () => {
    const { openModal } = useCreateQuestModal();

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
                        onClick={() => openModal({ navigateToQuest: true })}
                    >
                        Create a quest
                    </Button>
                </div>
            </div>
            <div className="flex flex-col gap-2">
                <QuestSummaryCard />
                <BeaconSummaryCard />
            </div>
        </>
    );
}
