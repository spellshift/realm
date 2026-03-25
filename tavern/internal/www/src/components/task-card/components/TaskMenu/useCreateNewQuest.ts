import { useCallback } from "react";
import { useCreateQuestModal } from "../../../../context/CreateQuestModalContext";
import { TaskNode } from "../../../../utils/interfacesQuery";

export function useCreateNewQuest(task: TaskNode) {
    const { openModal } = useCreateQuestModal();

    const handleCreateNewQuest = useCallback(() => {
        openModal({
            initialFormData: {
                beacons: [task.beacon.id],
                initialStep: 1,
            },
            navigateToQuest: true,
        });
    }, [task, openModal]);

    return { handleCreateNewQuest };
}
