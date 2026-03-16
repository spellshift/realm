import Button from "../../../../components/tavern-base-ui/button/Button";
import { useCreateQuestModal } from "../../../../context/CreateQuestModalContext";
import { useProcessQuestFormData } from "./useProcessQuestFormData";
import { RefreshCw } from "lucide-react";
import Tooltip from "../../../../components/tavern-base-ui/Tooltip";

export const RefetchProcessQuestButton = () => {
    const { openModal } = useCreateQuestModal();
    const { fetchFormData, loading } = useProcessQuestFormData();

    const handleClick = async () => {
        const initialFormData = await fetchFormData();
        openModal({
            initialFormData,
            refetchQueries: ["GetHostContext", "GetProcessIds"],
        });
    };

    return (
        <Tooltip label="Create a new quest to update the process list">
            <Button
                leftIcon={<RefreshCw className="w-4 h-4" />}
                onClick={handleClick}
                buttonVariant="ghost"
                buttonStyle={{ color: "gray", size: "sm" }}
                disabled={loading}
            >
                {loading ? "Loading..." : "Refetch"}
            </Button>
        </Tooltip>
    );
};
