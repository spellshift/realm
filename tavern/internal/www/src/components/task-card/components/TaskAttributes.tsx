import { FC } from "react";
import { TaskNode } from "../../../utils/interfacesQuery";
import { constructTomeParams } from "../../../utils/utils";
import { CopyableKeyValues } from "../../copyable-key-values/CopyableKeyValues";

interface TaskAttributesProps {
    task: TaskNode;
}

const formatDateTime = (dateString: string | null): string | null => {
    if (!dateString) return null;
    try {
        const date = new Date(dateString);
        return date.toLocaleString();
    } catch {
        return null;
    }
};

const TaskAttributes: FC<TaskAttributesProps> = ({ task }) => {
    const questParams = constructTomeParams(
        task.quest.parameters,
        task.quest.tome.paramDefs
    );

    const timeFields = [
        { name: "createdAt", label: "Created At", value: formatDateTime(task.createdAt) },
        { name: "claimedAt", label: "Claimed At", value: formatDateTime(task.claimedAt) },
        { name: "execStartedAt", label: "Execution Started", value: formatDateTime(task.execStartedAt) },
        { name: "execFinishedAt", label: "Execution Finished", value: formatDateTime(task.execFinishedAt) },
        { name: "lastModifiedAt", label: "Last Modified", value: formatDateTime(task.lastModifiedAt) },
    ].filter(field => field.value !== null).map(field => ({
        ...field,
        type: "text",
        placeholder: "",
    }));

    return (
        <div className="grid grid-cols-2 gap-4 py-4">
            {timeFields.length > 0 && (
                <CopyableKeyValues params={timeFields} heading="Status details" />
            )}
            {questParams.length > 0 && (
                <CopyableKeyValues params={questParams} heading="Tome parameters" />
            )}
        </div>
    );
};

export default TaskAttributes;
