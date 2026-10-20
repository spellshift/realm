import { FC } from "react";
import Modal from "../../../../components/tavern-base-ui/Modal";
import AgentBuilderWizard from "../../../agent-builder/components/AgentBuilderWizard";
import { useToast } from "@chakra-ui/react";

type GenerateAgentModalProps = {
    isOpen: boolean;
    setOpen: (open: boolean) => void;
    onSuccess?: () => void;
};

export const GenerateAgentModal: FC<GenerateAgentModalProps> = ({ isOpen, setOpen, onSuccess }) => {
    const toast = useToast();

    const handleComplete = (taskId: string) => {
        toast({
            title: "Agent build started",
            description: `Build task #${taskId} queued. The generated artifact will appear here once built.`,
            status: "success",
            duration: 5000,
            isClosable: true,
        });
        setOpen(false);
        if (onSuccess) {
            onSuccess();
        }
    };

    return (
        <Modal setOpen={setOpen} isOpen={isOpen} size="lg">
            <div className="flex flex-col gap-4 pb-4" data-testid="generate-agent-modal">
                <AgentBuilderWizard
                    showHeader={false}
                    onComplete={handleComplete}
                    onCancel={() => setOpen(false)}
                />
            </div>
        </Modal>
    );
};

export default GenerateAgentModal;
