import { FC, Fragment } from "react";
import { Menu, Transition } from "@headlessui/react";
import { Ellipsis, RefreshCw, Terminal, FileTerminal } from "lucide-react";
import Button from "../../../tavern-base-ui/button/Button";
import { TaskNode } from "../../../../utils/interfacesQuery";
import { checkIfBeaconOffline } from "../../../../utils/utils";
import { useRerunTask } from "./useRerunTask";
import { useCreateNewQuest } from "./useCreateNewQuest";
import { useOpenShell } from "./useOpenShell";

interface TaskMenuProps {
    task: TaskNode;
}

const TaskMenu: FC<TaskMenuProps> = ({ task }) => {
    const beaconOffline = checkIfBeaconOffline(task.beacon);
    const { handleRerunTask } = useRerunTask(task);
    const { handleCreateNewQuest } = useCreateNewQuest(task);
    const { handleOpenShell, loading: shellLoading } = useOpenShell(task.beacon.id);

    if (beaconOffline) {
        return null;
    }

    return (
        <Menu as="div" className="relative">
            <Menu.Button
                as={Button}
                buttonVariant="ghost"
                buttonStyle={{ color: "gray", size: "sm" }}
                aria-label="Task menu"
            >
                <Ellipsis className="w-4 h-4" />
            </Menu.Button>
            <Transition
                as={Fragment}
                enter="transition ease-out duration-100"
                enterFrom="transform opacity-0 scale-95"
                enterTo="transform opacity-100 scale-100"
                leave="transition ease-in duration-75"
                leaveFrom="transform opacity-100 scale-100"
                leaveTo="transform opacity-0 scale-95"
            >
                <Menu.Items className="absolute right-0 mt-2 w-48 origin-top-right divide-y divide-gray-100 rounded-md bg-white shadow-lg ring-1 ring-black/5 focus:outline-none z-10">
                    <div className="px-1 py-1">
                        <Menu.Item>
                            {() => (
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "sm" }}
                                    className="w-full justify-start"
                                    leftIcon={<RefreshCw className="w-4 h-4" />}
                                    onClick={handleRerunTask}
                                >
                                    Rerun quest for task
                                </Button>
                            )}
                        </Menu.Item>
                        <Menu.Item>
                            {() => (
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "sm" }}
                                    className="w-full justify-start"
                                    leftIcon={<FileTerminal className="w-4 h-4" />}
                                    onClick={handleCreateNewQuest}
                                >
                                    Create new quest
                                </Button>
                            )}
                        </Menu.Item>
                        <Menu.Item>
                            {() => (
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "sm" }}
                                    className="w-full justify-start"
                                    leftIcon={<Terminal className="w-4 h-4" />}
                                    onClick={handleOpenShell}
                                    isLoading={shellLoading}
                                >
                                    Open shell
                                </Button>
                            )}
                        </Menu.Item>
                    </div>
                </Menu.Items>
            </Transition>
        </Menu>
    );
};

export default TaskMenu;
