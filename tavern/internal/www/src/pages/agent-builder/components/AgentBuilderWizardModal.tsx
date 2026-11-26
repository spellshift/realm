import { FC } from "react";
import Modal from "../../../components/tavern-base-ui/Modal";
import AgentBuilderWizard from "./AgentBuilderWizard";

type AgentBuilderWizardModalProps = {
    isOpen: boolean;
    setOpen: (open: boolean) => void;
    initialProfileId?: string;
    onComplete?: (taskId: string) => void;
};

/**
 * Slide-out for generating an agent, matching the Create Quest slide-out.
 * Callers should only mount it while open so the wizard state resets.
 */
export const AgentBuilderWizardModal: FC<AgentBuilderWizardModalProps> = ({ isOpen, setOpen, initialProfileId, onComplete }) => (
    <Modal isOpen={isOpen} setOpen={setOpen} size="xl">
        <AgentBuilderWizard
            showHeader={false}
            initialProfileId={initialProfileId}
            onCancel={() => setOpen(false)}
            onComplete={(taskId) => {
                setOpen(false);
                onComplete?.(taskId);
            }}
        />
    </Modal>
);

export default AgentBuilderWizardModal;
