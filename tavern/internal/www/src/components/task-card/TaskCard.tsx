import { FC } from "react";
import TaskTimeStamp from "./components/TaskTimeStamp";
import UserImageAndName from "../UserImageAndName";
import TaskStatusBadge from "../TaskStatusBadge";
import TaskShells from "./components/TaskShells";
import { Tabs } from "@chakra-ui/react";
import { BookOpenIcon, CommandLineIcon, DocumentTextIcon, NoSymbolIcon } from "@heroicons/react/24/outline";
import TaskResults from "./components/TaskResults";
import { TaskNode } from "../../utils/interfacesQuery";
import BeaconTile from "../BeaconTile";
import TomeAccordion from "../TomeAccordion";
import { constructTomeParams } from "../../utils/utils";

interface TaskCardType {
    task: TaskNode
}

const TaskCard: FC<TaskCardType> = (
    { task }
) => {

    // If Task has an error, start with error tab open
    const defaultOpenTabValue = task?.error ? "error" : "output";
    const tome = task?.quest?.tome;
    const params = constructTomeParams(task?.quest?.parameters, tome?.paramDefs);

    return (
        <div className="rounded-lg shadow border-gray-200 border-2">
            <div className="flex flex-row gap-4 items-center p-4 bg-gray-100">
                <h3 className="text-lg font-semibold">{task.quest?.name}</h3>
                <TaskStatusBadge task={task} />
            </div>
            <div className="grid grid-cols-1 lg:grid-cols-2">
                <div className="border-t-4 border-gray-100 flex flex-col gap-4 px-4 py-2 w-full">
                    <BeaconTile beacon={task.beacon} isBeaconIconVisible />
                    <TaskTimeStamp {...task} />
                    <UserImageAndName userData={task?.quest?.creator} />
                    {tome && <div className="-mx-4"><TomeAccordion tome={tome} params={params} leftContent={<BookOpenIcon className="h-5 w-5 mt-1 shrink-0 self-start" />} /></div>}
                </div>
                <Tabs.Root className="border-t-4 border-l-4 border-gray-100 rounded flex flex-col gap-1 -mt-10" defaultValue={defaultOpenTabValue}>
                    <Tabs.List className="flex flex-row gap-2 text-gray-600 bg-gray-100">
                        <Tabs.Trigger value="output" className="flex flex-row gap-1 items-center py-2 px-4 data-[state=active]:bg-white data-[state=active]:rounded data-[state=active]:border-t-2 data-[state=active]:border-gray-200"><DocumentTextIcon className="w-4" /> Output</Tabs.Trigger>
                        {task?.error && <Tabs.Trigger value="error" className="flex flex-row gap-1 items-center py-2 px-4 data-[state=active]:bg-white data-[state=active]:rounded data-[state=active]:border-t-2 data-[state=active]:border-gray-200"><NoSymbolIcon className="w-4" /> Error</Tabs.Trigger>}
                        {task?.shells?.edges.length > 0 && <Tabs.Trigger value="shells" className="flex flex-row gap-1 items-center py-2 px-4 data-[state=active]:bg-white data-[state=active]:rounded data-[state=active]:border-t-2 data-[state=active]:border-gray-200"><CommandLineIcon className="w-4" /> Shells</Tabs.Trigger>}
                    </Tabs.List>
                    <div className="px-4">
                        <Tabs.Content value="output" aria-label="output panel">
                            <TaskResults result={task?.output} />
                        </Tabs.Content>
                        {task?.error &&
                            <Tabs.Content value="error" aria-label="error panel">
                                <TaskResults result={task?.error} />
                            </Tabs.Content>}
                        {task?.shells?.edges.length > 0 && <Tabs.Content value="shells" aria-label="shells panel">
                            <TaskShells shells={task?.shells?.edges} />
                        </Tabs.Content>}
                    </div>
                </Tabs.Root>
            </div>
        </div>
    );
};
export default TaskCard;
