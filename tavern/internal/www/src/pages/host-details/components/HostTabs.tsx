import { Tabs } from "@chakra-ui/react"
import { ArrowsUpDownIcon, ClipboardDocumentCheckIcon } from "@heroicons/react/20/solid"
import { useHost } from "../../../context/HostContext";
import { KeyIcon } from "@heroicons/react/24/outline";
import { getOfflineOnlineStatus } from "../../../utils/utils";
import { useQuery } from "@apollo/client";
import { useParams } from "react-router-dom";
import { GET_HOST_TASK_COUNT } from "../../../utils/queries";


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
        <Tabs.List className="flex flex-row space-x-4 border-gray-200 w-full bg-gray-100 ">
            <Tabs.Trigger value="beacons" className="p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg data-[state=active]:border-t-purple-600 data-[state=active]:bg-white data-[state=active]:text-purple-800 hover:bg-gray-100 border-transparent hover:bg-white hover:border-t-purple-600">
                <ArrowsUpDownIcon className="w-4 h-4" />
                <div>
                    Beacons
                </div>
                <div>
                    {(host?.beacons?.edges?.length !== undefined && host?.beacons?.edges?.length !== null) && `(${online}/${host?.beacons?.edges?.length})`}
                </div>
            </Tabs.Trigger>
            <Tabs.Trigger value="tasks" className="p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg data-[state=active]:border-t-purple-600 data-[state=active]:bg-white data-[state=active]:text-purple-800 hover:bg-gray-100 border-transparent hover:bg-white hover:border-t-purple-600">
                <ClipboardDocumentCheckIcon className="w-4 h-4" />
                <div>
                    Tasks
                </div>
                <div>
                    {hostTaskData?.tasks?.totalCount && `(${hostTaskData.tasks.totalCount})`}
                </div>
            </Tabs.Trigger>
            <Tabs.Trigger value="credentials" className="p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg data-[state=active]:border-t-purple-600 data-[state=active]:bg-white data-[state=active]:text-purple-800 hover:bg-gray-100 border-transparent hover:bg-white hover:border-t-purple-600">
                <KeyIcon className="w-4 h-4" />
                <div>
                    Credentials
                </div>
                <div>
                    {host?.credentials?.edges?.length && `(${host?.credentials?.edges?.length || 0})`}
                </div>
            </Tabs.Trigger>
        </Tabs.List>
    )
}
export default HostTabs;
