import { FileTerminal } from "lucide-react";
import { useCreateQuestModal } from "../../context/CreateQuestModalContext";
import { OpenCreateQuestModalOptions, ScheduledQuestHostScope } from "../../context/CreateQuestModalContext/types";
import { QuestActionDropdown } from "../quest-action-dropdown/QuestActionDropdown";

interface CreateQuestDropdownProps {
    primaryOpenOptions?: OpenCreateQuestModalOptions;
    scheduledHost?: ScheduledQuestHostScope;
}

export const CreateQuestDropdown = ({ primaryOpenOptions, scheduledHost }: CreateQuestDropdownProps) => {
    const { openModal } = useCreateQuestModal();

    return (
        <QuestActionDropdown
            label="Create quest"
            leftIcon={<FileTerminal className="w-5 h-5" />}
            actions={[
                { label: "Create quest", onClick: () => openModal(primaryOpenOptions ?? { navigateToQuest: true }) },
                {
                    label: "Schedule quest",
                    onClick: () => openModal(scheduledHost ? { mode: "scheduled", scheduledHost } : { mode: "scheduled" }),
                },
            ]}
        />
    );
};
