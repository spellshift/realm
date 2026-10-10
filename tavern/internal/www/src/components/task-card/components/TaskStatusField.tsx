import { FC } from "react";
import { CircleCheck, CirclePause, CircleX, Clock } from "lucide-react";
import { TaskNode } from "../../../utils/interfacesQuery";

interface TaskStatusFieldProps {
    task: TaskNode;
}

const TaskStatusField: FC<TaskStatusFieldProps> = ({ task }) => {
    const iconClass = "w-6 h-6 mt-1";

    if (task.error) {
        return <CircleX className={`${iconClass} text-red-600`} />;
    }
    if (task.execFinishedAt) {
        return <CircleCheck className={`${iconClass} text-green-600`} />;
    }
    if (task.execStartedAt) {
        return <Clock className={`${iconClass} text-gray-700`} />;
    }
    return <CirclePause className={`${iconClass} text-gray-700`} />;
};

export default TaskStatusField;
