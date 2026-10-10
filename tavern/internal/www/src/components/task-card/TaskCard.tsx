import { FC } from "react";
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/react";
import { CommandLineIcon, DocumentTextIcon, NoSymbolIcon } from "@heroicons/react/24/outline";
import { BookOpenIcon, FileText, ListVideo, FileCog } from "lucide-react";
import TaskResults from "./components/TaskResults";
import TaskShells from "./components/TaskShells";
import TaskProcesses from "./components/TaskProcesses";
import TaskFiles from "./components/TaskFiles";
import TaskTome from "./components/TaskTome";
import TaskAttributes from "./components/TaskAttributes";
import { TaskNode } from "../../utils/interfacesQuery";
import BeaconFields from "./components/BeaconFields";
import UserFields from "./components/UserFields";
import TomeFields from "./components/TomeFields";
import TaskStatusField from "./components/TaskStatusField";
import { TaskMenu } from "./components/TaskMenu";

interface TaskCardType {
    task: TaskNode
}

const TaskCard: FC<TaskCardType> = ({ task }) => {
    const defaultOpenTabIndex = task?.error ? 3 : 2;

    return (
        <div className="rounded-lg shadow border-gray-200 border-2 w-full text-gray-800">
            <div className="flex flex-row gap-4 px-4 py-4 bg-gray-50 w-full">
                <TaskStatusField task={task} />
                <div className="flex flex-col gap-2 w-full">
                    <div className="flex flex-row gap-4 justify-between items-center w-full">
                        <div className="flex flex-row gap-4 items-center">
                            <div className="text-lg font-bold">
                                {task.quest.name}
                            </div>
                            <UserFields user={task.quest.creator} />
                        </div>
                        <TaskMenu task={task} />
                    </div>
                    <TomeFields tome={task.quest.tome} />
                    <BeaconFields beacon={task.beacon} />
                </div>
            </div>
            <TabGroup className="border-t-4 border-l-4 border-gray-100 rounded flex flex-col gap-1 lg:col-span-3" defaultIndex={defaultOpenTabIndex}>
                <TabList className="flex flex-row gap-2 text-gray-600 bg-gray-100 text-sm">
                    <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}>
                        <FileCog className="w-4" /> Attributes
                    </Tab>
                    <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}>
                        <BookOpenIcon className="w-4" /> Tome
                    </Tab>
                    <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}>
                        <DocumentTextIcon className="w-4" /> Output
                    </Tab>
                    {task?.error && (
                        <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}>
                            <NoSymbolIcon className="w-4" /> Error
                        </Tab>
                    )}
                    {task?.shells?.edges.length > 0 && (
                        <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}>
                            <CommandLineIcon className="w-4" /> Shells
                        </Tab>
                    )}
                    {task?.reportedProcesses?.totalCount > 0 && (
                        <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}>
                            <ListVideo className="w-4" /> Processes ({task?.reportedProcesses?.totalCount})
                        </Tab>
                    )}
                    {task?.reportedFiles?.totalCount > 0 && (
                        <Tab className={({ selected }) => `flex flex-row gap-1 items-center py-2 px-4 ${selected && 'bg-white rounded border-t-2 border-gray-200'}`}>
                            <FileText className="w-4" /> Files ({task?.reportedFiles?.totalCount})
                        </Tab>
                    )}
                </TabList>
                <TabPanels className="px-4">
                     <TabPanel aria-label="attributes panel">
                        <TaskAttributes task={task} />
                    </TabPanel>
                    <TabPanel aria-label="tome panel">
                        <TaskTome tome={task.quest.tome} />
                    </TabPanel>
                    <TabPanel aria-label="output panel">
                        <TaskResults result={task?.output} />
                    </TabPanel>
                    {task?.error && (
                        <TabPanel aria-label="error panel">
                            <TaskResults result={task?.error} />
                        </TabPanel>
                    )}
                    {task?.shells?.edges.length > 0 && (
                        <TabPanel aria-label="shells panel">
                            <TaskShells shells={task?.shells?.edges} />
                        </TabPanel>
                    )}
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
    );
};
export default TaskCard;
