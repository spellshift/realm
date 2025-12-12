import React, { useContext } from "react"
import { Tab } from "@headlessui/react"
import { ArrowsUpDownIcon, ClipboardDocumentCheckIcon } from "@heroicons/react/20/solid"
import { HostContext } from "../../../context/HostContext";
import { KeyIcon } from "@heroicons/react/24/outline";
import { getOfflineOnlineStatus } from "../../../utils/utils";
import { useQuery } from "@apollo/client";
import { useParams } from "react-router-dom";
import { GET_HOST_TASK_COUNT } from "../../../utils/queries";


const HostTabs = () => {
    const { hostId } = useParams();
    const { data: host } = useContext(HostContext);
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

    const { online } = getOfflineOnlineStatus(host?.beacons || []);

    return (
        <Tab.List className="flex flex-row space-x-4 border-gray-200 w-full bg-gray-100 ">
            <Tab className={({ selected }) => `p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg ${selected ? 'border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100' : 'border-transparent hover:bg-white hover:border-t-purple-600'}`}>
                <ArrowsUpDownIcon className="w-4 h-4" />
                <div>
                    Beacons
                </div>
                <div>
                    {(host?.beacons?.length !== undefined && host?.beacons?.length !== null) && `(${online}/${host?.beacons?.length})`}
                </div>
            </Tab>
            <Tab className={({ selected }) => `p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg ${selected ? 'border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100' : 'border-transparent hover:bg-white hover:border-t-purple-600'}`}>
                <ClipboardDocumentCheckIcon className="w-4 h-4" />
                <div>
                    Tasks
                </div>
                <div>
                    {hostTaskData?.tasks?.totalCount && `(${hostTaskData.tasks.totalCount})`}
                </div>
            </Tab>
            <Tab className={({ selected }) => `p-4 flex flex-row gap-1 items-center border-t-2 border-l-2 border-r-2 rounded-t-lg ${selected ? 'border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100' : 'border-transparent hover:bg-white hover:border-t-purple-600'}`}>
                <KeyIcon className="w-4 h-4" />
                <div>
                    Credentials
                </div>
                <div>
                    {host?.credentials?.length && `(${host?.credentials?.length || 0})`}
                </div>
            </Tab>
        </Tab.List>
    )
}
export default HostTabs;
