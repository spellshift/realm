import { useCreateQuestModal } from "../../context/CreateQuestModalContext";
import { QuestActionDropdown } from "../quest-action-dropdown/QuestActionDropdown";

export const CreateQuestDropdown = () => {
    const { openModal } = useCreateQuestModal();

    return (
        <QuestActionDropdown
            label="Create quest"
            primaryAction={() => openModal({ navigateToQuest: true })}
            actions={[
                { label: "Create scheduled quest", onClick: () => openModal({ mode: "scheduled" }) },
            ]}
        />
    );
};
