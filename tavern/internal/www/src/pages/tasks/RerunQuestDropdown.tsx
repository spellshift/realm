import { FileTerminal } from "lucide-react";
import { QuestActionDropdown } from "../../components/quest-action-dropdown/QuestActionDropdown";
import { useCreateQuestModal } from "../../context/CreateQuestModalContext";
import { useRerunQuestFormData } from "./useRerunQuestFormData";
import { QuestNode } from "../../utils/interfacesQuery";

interface RerunQuestDropdownProps {
    quest: QuestNode;
}

export const RerunQuestDropdown = ({ quest }: RerunQuestDropdownProps) => {
    const { openModal } = useCreateQuestModal();
    const { buildFormDataWithSameTome, buildFormDataWithNewTome, loading } =
        useRerunQuestFormData(quest);

    const handleRerunWithOnlineBeacons = async () => {
        const initialFormData = await buildFormDataWithSameTome();
        openModal({
            initialFormData,
            navigateToQuest: true,
        });
    };

    const handleRerunWithNewTome = async () => {
        const initialFormData = await buildFormDataWithNewTome();
        openModal({
            initialFormData,
            navigateToQuest: true,
        });
    };

    return (
        <QuestActionDropdown
            label="Re-run quest"
            leftIcon={<FileTerminal className="h-5 w-5" />}
            loading={loading}
            menuWidthClass="w-72"
            menuItemSize="md"
            actions={[
                { label: "Rerun with online beacons", onClick: handleRerunWithOnlineBeacons },
                { label: "Rerun with new tome", onClick: handleRerunWithNewTome },
            ]}
        />
    );
};
