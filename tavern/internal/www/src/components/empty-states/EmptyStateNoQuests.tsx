import { EmptyState, EmptyStateType } from "../tavern-base-ui/EmptyState";
import Button from "../tavern-base-ui/button/Button";
import { useCreateQuestModal } from "../../context/CreateQuestModalContext";
import { FileTerminal } from "lucide-react";

const EmptyStateNoQuests = () => {
    const { openModal } = useCreateQuestModal();

    return (
        <EmptyState label="No quests found" type={EmptyStateType.noData} details="Get started by creating a new quest." >
            <Button
                leftIcon={<FileTerminal className="w-5 h-5" />}
                buttonStyle={{ color: "purple", size: "md" }}
                onClick={() => openModal({navigateToQuest: true})}
            >
                Create new quest
            </Button>
        </EmptyState>
    )
}
export default EmptyStateNoQuests;
