import { FC } from "react";
import { RefreshCw, Terminal, FileTerminal } from "lucide-react";
import { Tooltip } from "@chakra-ui/react";
import Button from "../../../tavern-base-ui/button/Button";
import { TaskNode } from "../../../../utils/interfacesQuery";
import { checkIfBeaconOffline } from "../../../../utils/utils";
import { useRerunTask } from "./useRerunTask";
import { useCreateNewQuest } from "./useCreateNewQuest";
import { useOpenShell } from "./useOpenShell";

interface TaskMenuFullProps {
    task: TaskNode;
}

const TaskMenuFull: FC<TaskMenuFullProps> = ({ task }) => {
    const beaconOffline = checkIfBeaconOffline(task.beacon);
    const { handleRerunTask } = useRerunTask(task);
    const { handleCreateNewQuest } = useCreateNewQuest(task);
    const { handleOpenShell, loading: shellLoading } = useOpenShell(task.beacon.id);

    if (beaconOffline) {
        return null;
    }

    return (
        <div className="flex flex-row gap-1">
            <Tooltip label="Rerun quest for task" bg="white" color="black" hasArrow>
                <span>
                    <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "gray", size: "sm" }}
                        leftIcon={<RefreshCw className="w-4 h-4" />}
                        onClick={handleRerunTask}
                        aria-label="Rerun quest for task"
                    />
                </span>
            </Tooltip>
            <Tooltip label="Create new quest" bg="white" color="black" hasArrow>
                <span>
                    <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "gray", size: "sm" }}
                        leftIcon={<FileTerminal className="w-4 h-4" />}
                        onClick={handleCreateNewQuest}
                        aria-label="Create new quest"
                    />
                </span>
            </Tooltip>
            <Tooltip label="Open shell" bg="white" color="black" hasArrow>
                <span>
                    <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "gray", size: "sm" }}
                        leftIcon={<Terminal className="w-4 h-4" />}
                        onClick={handleOpenShell}
                        isLoading={shellLoading}
                        aria-label="Open shell"
                    />
                </span>
            </Tooltip>
        </div>
    );
};

export default TaskMenuFull;
