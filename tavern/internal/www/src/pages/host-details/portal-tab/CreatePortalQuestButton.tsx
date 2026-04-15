import Button from "../../../components/tavern-base-ui/button/Button";
import { useCreateQuestModal } from "../../../context/CreateQuestModalContext";
import { usePortalQuestFormData } from "./usePortalQuestFormData";
import { PlusIcon } from "lucide-react";

interface CreatePortalQuestButtonProps {
    variant?: "solid" | "ghost";
    size?: "sm" | "md" | "lg" | "xs";
}

export const CreatePortalQuestButton = ({ variant = "solid", size = "md" }: CreatePortalQuestButtonProps) => {
    const { openModal } = useCreateQuestModal();
    const { fetchFormData, loading } = usePortalQuestFormData();

    const handleClick = async () => {
        const initialFormData = await fetchFormData();
        openModal({
            initialFormData,
            refetchQueries: ["GetHostContext", "GetPortalIds"],
        });
    };

    return (
        <Button
            leftIcon={<PlusIcon className="w-4 h-4" />}
            onClick={handleClick}
            buttonVariant={variant}
            buttonStyle={{ color: variant === "solid" ? "purple" : "gray", size }}
            disabled={loading}
        >
            {loading ? "Loading..." : "Create Portal"}
        </Button>
    );
};
