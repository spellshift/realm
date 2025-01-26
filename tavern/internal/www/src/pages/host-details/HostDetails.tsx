import { PageWrapper } from "../../components/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import HostTasks from "./components/HostTasks";
import { Tab } from '@headlessui/react'
import { useQuery } from "@apollo/client";
import { useParams } from "react-router-dom";
import { GET_HOST_QUERY, GET_HOST_TASK_SUMMARY } from "../../utils/queries";
import EditableHostHeader from "./components/EditableHostHeader";
import { HostType } from "../../utils/consts";
import HostStatistics from "./components/HostStatistics";
import BeaconTableWrapper from "./components/BeaconTableWrapper";

const HostDetails = () => {
    const { hostId } = useParams();
    const { loading, data, error } = useQuery(GET_HOST_QUERY, {
        variables: {
            "where": {
                "id": hostId
            }
        }
    });

    const { loading: taskLoading, data: taskData, error: taskError } = useQuery(GET_HOST_TASK_SUMMARY, {
        variables: {
            "where": {
                "hasBeaconWith": {
                    "hasHostWith": {
                        "id": hostId
                    }
                }
            }
        }
    });

    const host = data?.hosts && data?.hosts.length > 0 ? data.hosts[0] : null as HostType | null;

    return (
        <PageWrapper currNavItem={PageNavItem.hosts}>
            <EditableHostHeader hostId={hostId} loading={loading} error={error} hostData={host} />
            <HostStatistics host={host} taskLoading={taskLoading} taskError={taskError} taskData={taskData} />
            <Tab.Group>
                <Tab.List className="flex flex-row space-x-4 border-b border-gray-200 w-full">
                    <Tab className={({ selected }) => `border-b-2 py-2 px-4 text-sm font-semibold ${selected ? 'border-purple-700 text-purple-800' : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'}`}>Beacons</Tab>
                    <Tab className={({ selected }) => `border-b-2 py-2 px-4 text-sm font-semibold ${selected ? 'border-purple-700 text-purple-800' : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'}`}>Tasks</Tab>
                </Tab.List>
                <Tab.Panels>
                    <Tab.Panel>
                        <BeaconTableWrapper loading={loading} error={error} host={host} />
                    </Tab.Panel>
                    <Tab.Panel>
                        <HostTasks />
                    </Tab.Panel>
                </Tab.Panels>
            </Tab.Group>
        </PageWrapper>
    );
}
export default HostDetails;
