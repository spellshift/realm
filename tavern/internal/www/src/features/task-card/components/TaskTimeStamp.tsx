import { FC, ReactElement } from "react";
import { Task } from "../../../utils/consts";
import { ClockIcon } from "@heroicons/react/24/outline";

interface TaskTimeStampType extends Pick<Task, 'createdAt' | 'execStartedAt' | 'execFinishedAt'> { };

const TaskTimeStamp: FC<TaskTimeStampType> = ({
    createdAt,
    execStartedAt,
    execFinishedAt
}): ReactElement => {
    const createdTime = new Date(createdAt || "");
    const startTime = new Date(execStartedAt || "");
    const finishTime = new Date(execFinishedAt || "");

    return (
        <div className="flex flex-row gap-4">
            <ClockIcon className="h-5 w-5 mt-1" />
            <div className="flex flex-col gap-1 ">
                <div className="text-gray-600">Status</div>
                {createdAt && <span className="text-xs">{`Created at ${createdTime.toLocaleTimeString()} on ${createdTime.toDateString()}`}</span>}
                {execStartedAt && <span className="text-xs">{`Started at ${startTime.toLocaleTimeString()} on ${startTime.toDateString()}`}</span>}
                {execFinishedAt && <span className="text-xs">{`Finished at ${finishTime.toLocaleTimeString()} on ${finishTime.toDateString()}`}</span>}
            </div>
        </div>
    );
}
export default TaskTimeStamp;
