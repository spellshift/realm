import { Task } from "../../utils/consts";
import { FC } from "react";
import TaskTimeStamp from "./components/TaskTimeStamp";
import TaskHostBeacon from "./components/TaskHostBeacon";
import TaskParameters from "./components/TaskParameters";
import UserImageAndName from "../../components/UserImageAndName";
import TaskStatusBadge from "../../components/TaskStatusBadge";
import TaskShells from "./components/TaskShells";
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/react";
import { CommandLineIcon, DocumentTextIcon, NoSymbolIcon } from "@heroicons/react/24/outline";
import TaskResults from "./components/TaskResults";

interface TaskCardType {
    task: Task
}

const TaskCard: FC<TaskCardType> = (
    { task }
) => {

    // If Task has an error, start with error tab open
    const defaultOpenTabIndex = task?.error ? 1 : 0;

    return (
        <div className="rounded-lg shadow border-gray-200 border-2">
            <div className="flex flex-row gap-4 items-center p-4 bg-gray-100">
                <h3 className="text-lg font-semibold">{task.quest?.name}</h3>
                <TaskStatusBadge task={task} />
            </div>
            <div className="grid grid-cols-1 lg:grid-cols-2">
                <div className="border-t-4 border-gray-100 flex flex-col gap-4 px-4 py-2 w-full">
                    <TaskHostBeacon beaconData={task.beacon} />
                    <TaskTimeStamp {...task} />
                    <TaskParameters quest={task?.quest} />
                    <UserImageAndName userData={task?.quest?.creator} />
                </div>
                <TabGroup className="border-t-4 border-l-4 border-gray-100 rounded flex flex-col gap-1 -mt-10" defaultIndex={defaultOpenTabIndex}>
                    <TabList className="flex flex-row gap-2 text-gray-600 bg-gray-100">
                        <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}><DocumentTextIcon className="w-4" /> Output</Tab>
                        {task?.error && <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}><NoSymbolIcon className="w-4" /> Error</Tab>}
                        {task?.shells?.length > 0 && <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}><CommandLineIcon className="w-4" /> Shells</Tab>}
                    </TabList>
                    <TabPanels className="px-4">
                        <TabPanel>
                            <TaskResults result={task?.output} />
                        </TabPanel>
                        {task?.error &&
                            <TabPanel>
                                <TaskResults result={task?.error} />
                            </TabPanel>}
                        {task?.shells?.length > 0 && <TabPanel>
                            <TaskShells shells={task?.shells} />
                        </TabPanel>}
                    </TabPanels>
                </TabGroup>
            </div>
        </div>
    );
};
export default TaskCard;
