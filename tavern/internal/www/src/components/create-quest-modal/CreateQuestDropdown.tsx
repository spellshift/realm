import { useCreateQuestModal } from "../../context/CreateQuestModalContext";
import { OpenCreateQuestModalOptions } from "../../context/CreateQuestModalContext/types";
import { QuestActionDropdown } from "../quest-action-dropdown/QuestActionDropdown";

interface CreateQuestDropdownProps {
    primaryOpenOptions?: OpenCreateQuestModalOptions;
}

export const CreateQuestDropdown = ({ primaryOpenOptions }: CreateQuestDropdownProps) => {
    const { openModal } = useCreateQuestModal();

    return (
        <QuestActionDropdown
            label="Create quest"
            actions={[
                { label: "Create quest", onClick: () => openModal(primaryOpenOptions ?? { navigateToQuest: true }) },
                { label: "Schedule quest", onClick: () => openModal({ mode: "scheduled" }) },
            ]}
        />
    );
};
