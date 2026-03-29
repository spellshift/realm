import Button from "../../../../components/tavern-base-ui/button/Button";
import { useCreateQuestModal } from "../../../../context/CreateQuestModalContext";
import { useProcessQuestFormData } from "./useProcessQuestFormData";

export const CreateProcessQuestButton = () => {
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
        <Button
            onClick={handleClick}
            buttonVariant="solid"
            buttonStyle={{ color: "purple", size: "md" }}
            disabled={loading}
        >
            {loading ? "Loading..." : "Get process list"}
        </Button>
    );
};
