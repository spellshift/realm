import { useCallback } from "react";
import { useCreateQuestModal } from "../../../../context/CreateQuestModalContext";
import { TaskNode } from "../../../../utils/interfacesQuery";
import { useQuestModalOptions } from "./useQuestModalOptions";

export function useCreateNewQuest(task: TaskNode) {
    const { openModal } = useCreateQuestModal();
    const questModalOptions = useQuestModalOptions();

    const handleCreateNewQuest = useCallback(() => {
        openModal({
            initialFormData: {
                beacons: [task.beacon.id],
                initialStep: 1,
            },
            ...questModalOptions,
        });
    }, [task, openModal, questModalOptions]);

    return { handleCreateNewQuest };
}
