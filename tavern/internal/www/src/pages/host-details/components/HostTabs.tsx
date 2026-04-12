import { Tab, TabList } from "@headlessui/react"
import { useHost } from "../../../context/HostContext";
import { ArrowUpDownIcon, FileCheckIcon, KeyRoundIcon, ListVideo, TerminalIcon, PlugIcon } from "lucide-react";


const HostTabs = () => {
    const { data: host, taskCount, totalShellCount, activeShellCount, totalPortalCount, activePortalCount } = useHost();

    return (
        <TabList className="flex flex-row space-x-4 border-gray-200 w-full bg-gray-100 ">
            <Tab className={({ selected }) => `p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg ${selected ? 'border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100' : 'border-transparent hover:bg-white hover:border-t-purple-600'}`}>
                <ArrowUpDownIcon className="w-4 h-4" />
                <div>
                    Beacons
                </div>
                <div>
                    {host?.beacons?.totalCount !== undefined && `(${host.beacons.totalCount})`}
                </div>
            </Tab>
            <Tab className={({ selected }) => `p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg ${selected ? 'border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100' : 'border-transparent hover:bg-white hover:border-t-purple-600'}`}>
                <FileCheckIcon className="w-4 h-4" />
                <div>
                    Tasks
                </div>
                <div>
                    {taskCount !== undefined && `(${taskCount})`}
                </div>
            </Tab>
            <Tab className={({ selected }) => `p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg ${selected ? 'border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100' : 'border-transparent hover:bg-white hover:border-t-purple-600'}`}>
                <ListVideo className="w-4 h-4" />
                <div>
                    Processes 
                </div>
                <div>
                    {host?.processes?.totalCount !== undefined && `(${host.processes.totalCount})`}
                </div>
            </Tab>
            <Tab className={({ selected }) => `p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg ${selected ? 'border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100' : 'border-transparent hover:bg-white hover:border-t-purple-600'}`}>
                <KeyRoundIcon className="w-4 h-4" />
                <div>
                    Files
                </div>
                <div>
                    {host?.files?.totalCount !== undefined && `(${host.files.totalCount})`}
                </div>
            </Tab>
            <Tab className={({ selected }) => `p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg ${selected ? 'border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100' : 'border-transparent hover:bg-white hover:border-t-purple-600'}`}>
                <KeyRoundIcon className="w-4 h-4" />
                <div>
                    Credentials
                </div>
                <div>
                    {host?.credentials?.totalCount !== undefined && `(${host.credentials.totalCount})`}
                </div>
            </Tab>
            <Tab className={({ selected }) => `p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg ${selected ? 'border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100' : 'border-transparent hover:bg-white hover:border-t-purple-600'}`}>
                <TerminalIcon className="w-4 h-4" />
                <div>
                    Shells
                </div>
                <div>
                    {totalShellCount !== undefined && `(${activeShellCount ?? 0}/${totalShellCount})`}
                </div>
            </Tab>
            <Tab className={({ selected }) => `p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg ${selected ? 'border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100' : 'border-transparent hover:bg-white hover:border-t-purple-600'}`}>
                <PlugIcon className="w-4 h-4" />
                <div>
                    Portals
                </div>
                <div>
                    {totalPortalCount !== undefined && `(${activePortalCount ?? 0}/${totalPortalCount})`}
                </div>
            </Tab>
        </TabList>
    )
}
export default HostTabs;
