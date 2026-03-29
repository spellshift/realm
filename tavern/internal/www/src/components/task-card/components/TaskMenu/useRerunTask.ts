import { useCallback } from "react";
import { constructTomeParams } from "../../../../utils/utils";
import { useCreateQuestModal } from "../../../../context/CreateQuestModalContext";
import { TaskNode } from "../../../../utils/interfacesQuery";
import { useQuestModalOptions } from "./useQuestModalOptions";

export function useRerunTask(task: TaskNode) {
    const { openModal } = useCreateQuestModal();
    const questModalOptions = useQuestModalOptions();

    const handleRerunTask = useCallback(() => {
        const params = constructTomeParams(task.quest.parameters, task.quest.tome?.paramDefs);

        openModal({
            initialFormData: {
                name: task.quest.name,
                tomeId: task.quest.tome?.id,
                params,
                beacons: [task.beacon.id],
            },
            ...questModalOptions,
        });
    }, [task, openModal, questModalOptions]);

    return { handleRerunTask };
}
