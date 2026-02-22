import { Tab, TabList } from "@headlessui/react"
import { useHost } from "../../../context/HostContext";
import { getOfflineOnlineStatus } from "../../../utils/utils";
import { useQuery } from "@apollo/client";
import { useParams } from "react-router-dom";
import { GET_HOST_TASK_COUNT } from "../../../utils/queries";
import { ArrowUpDownIcon, FileCheckIcon, KeyRoundIcon, ListVideo } from "lucide-react";


const HostTabs = () => {
    const { hostId } = useParams();
    const { data: host } = useHost();
    const { data: hostTaskData } = useQuery(GET_HOST_TASK_COUNT, {
        variables: {
            "where":
            {
                "hasBeaconWith": {
                    "hasHostWith": {
                        "id": hostId
                    }
                }
            }
        }
    });

    const { online } = getOfflineOnlineStatus(host?.beacons?.edges || []);

    return (
        <TabList className="flex flex-row space-x-4 border-gray-200 w-full bg-gray-100 ">
            <Tab className={({ selected }) => `p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg ${selected ? 'border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100' : 'border-transparent hover:bg-white hover:border-t-purple-600'}`}>
                <ArrowUpDownIcon className="w-4 h-4" />
                <div>
                    Beacons
                </div>
                <div>
                    {host?.beacons?.totalCount !== undefined && `(${online}/${host.beacons.totalCount})`}
                </div>
            </Tab>
            <Tab className={({ selected }) => `p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg ${selected ? 'border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100' : 'border-transparent hover:bg-white hover:border-t-purple-600'}`}>
                <FileCheckIcon className="w-4 h-4" />
                <div>
                    Tasks
                </div>
                <div>
                    {hostTaskData?.tasks?.totalCount !== undefined && `(${hostTaskData.tasks.totalCount})`}
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
                    Credentials
                </div>
                <div>
                    {host?.credentials?.totalCount !== undefined && `(${host.credentials.totalCount})`}
                </div>
            </Tab>
        </TabList>
    )
}
export default HostTabs;
