import { useCallback, useEffect, useState } from "react";
import { QuestProps, Task, UserType } from "../../../utils/consts";

type QuestTableRow = {
    id: string;
    name: string;
    tome: string;
    creator: UserType;
    finished: number;
    inprogress: number;
    queued: number;
    outputCount: number;
    lastUpdated: null | string,
    errorCount: number
}

export const useFormatQuests = (data: Array<QuestProps>) => {
    const [loading, setLoading] = useState<boolean>(false);
    const [formattedData, setFormattedData] = useState<Array<QuestTableRow>>([]);

    const formatQuestsTableData = useCallback((data: any) => {
        setLoading(true);

        const fData = data?.map((questNode: {node: QuestProps}) => {
            const taskDetails = questNode?.node.tasks.reduce((map: any, task: Task) => {
                const modMap = { ...map };

                if (task.execFinishedAt) {
                    modMap.finished += 1;
                }
                else if (task.execStartedAt) {
                    modMap.inprogress += 1;
                }
                else {
                    modMap.queued += 1;
                }

                if (new Date(task.lastModifiedAt) > new Date(modMap.lastUpdated)) {
                    modMap.lastUpdated = task.lastModifiedAt;
                }

                if (task.outputSize && task.outputSize > 0) {
                    modMap.outputCount += 1;
                }

                if (task.error.length > 0 ) {
                    modMap.errorCount += 1;
                }

                return modMap
            },
                {
                    finished: 0,
                    inprogress: 0,
                    queued: 0,
                    outputCount: 0,
                    lastUpdated: null,
                    errorCount: 0
                }
            );

            return {
                id: questNode?.node.id,
                name: questNode?.node.name,
                tome: questNode?.node?.tome.name,
                creator: questNode?.node?.creator,
                ...taskDetails
            }
        });
        setLoading(false);
        setFormattedData(fData)
    },[]);

    useEffect(()=> {
        formatQuestsTableData(data);
    },[data, formatQuestsTableData]);

    return {
        data: formattedData,
        loading,
    }
}
