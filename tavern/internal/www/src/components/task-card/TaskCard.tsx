import { FC } from "react";
import TaskTimeStamp from "./components/TaskTimeStamp";
import UserImageAndName from "../UserImageAndName";
import TaskStatusBadge from "../TaskStatusBadge";
import TaskShells from "./components/TaskShells";
import TaskProcesses from "./components/TaskProcesses";
import TaskFiles from "./components/TaskFiles";
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/react";
import { BookOpenIcon, CommandLineIcon, DocumentTextIcon, NoSymbolIcon } from "@heroicons/react/24/outline";
import TaskResults from "./components/TaskResults";
import { TaskNode } from "../../utils/interfacesQuery";
import BeaconTile from "../BeaconTile";
import TomeAccordion from "../TomeAccordion";
import { constructTomeParams } from "../../utils/utils";
import { FileText, ListVideo } from "lucide-react";

interface TaskCardType {
    task: TaskNode
}

const TaskCard: FC<TaskCardType> = (
    { task }
) => {

    // If Task has an error, start with error tab open
    const defaultOpenTabIndex = task?.error ? 1 : 0;
    const tome = task?.quest?.tome;
    const params = constructTomeParams(task?.quest?.parameters, tome?.paramDefs);

    return (
        <div className="rounded-lg shadow border-gray-200 border-2">
            <div className="flex flex-row gap-4 items-center p-4 bg-gray-100">
                <h3 className="text-lg font-semibold">{task.quest?.name}</h3>
                <TaskStatusBadge task={task} />
            </div>
            <div className="grid grid-cols-1 lg:grid-cols-5">
                <div className="border-t-4 border-gray-100 flex flex-col gap-4 px-4 py-2 w-full lg:col-span-2">
                    <BeaconTile beacon={task.beacon} isBeaconIconVisible />
                    <TaskTimeStamp {...task} />
                    <UserImageAndName userData={task?.quest?.creator} />
                    {tome && <div className="-mx-4"><TomeAccordion tome={tome} params={params} leftContent={<BookOpenIcon className="h-5 w-5 mt-1 shrink-0 self-start" />} /></div>}
                </div>
                <TabGroup className="border-t-4 border-l-4 border-gray-100 rounded flex flex-col gap-1 -mt-10 lg:col-span-3" defaultIndex={defaultOpenTabIndex}>
                    <TabList className="flex flex-row gap-2 text-gray-600 bg-gray-100">
                        <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}><DocumentTextIcon className="w-4" /> Output</Tab>
                        {task?.error && <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}><NoSymbolIcon className="w-4" /> Error</Tab>}
                        {task?.shells?.edges.length > 0 && <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}><CommandLineIcon className="w-4" /> Shells</Tab>}
                        {task?.reportedProcesses?.totalCount > 0 && <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}><ListVideo className="w-4" /> Processes ({task?.reportedProcesses?.totalCount})</Tab>}
                        {task?.reportedFiles?.totalCount > 0 && <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}><FileText className="w-4" /> Files ({task?.reportedFiles?.totalCount})</Tab>}
                    </TabList>
                    <TabPanels className="px-4">
                        <TabPanel aria-label="output panel">
                            <TaskResults result={task?.output} />
                        </TabPanel>
                        {task?.error &&
                            <TabPanel aria-label="error panel">
                                <TaskResults result={task?.error} />
                            </TabPanel>}
                        {task?.shells?.edges.length > 0 && <TabPanel aria-label="shells panel">
                            <TaskShells shells={task?.shells?.edges} />
                        </TabPanel>}
                        {task?.reportedProcesses?.totalCount > 0 && (
                            <TabPanel aria-label="process panel">
                                <TaskProcesses taskId={task.id} hostId={task.beacon.host?.id || ""} />
                            </TabPanel>
                        )}
                        {task?.reportedFiles?.totalCount > 0 && (
                            <TabPanel aria-label="files panel">
                                <TaskFiles taskId={task.id} hostId={task.beacon.host?.id || ""} />
                            </TabPanel>
                        )}
                    </TabPanels>
                </TabGroup>
            </div>
        </div>
    );
};
export default TaskCard;
