import { useQuery } from "@apollo/client";
import { useParams } from "react-router-dom";
import { HostType } from "../../../utils/consts";
import { GET_HOST_QUERY, GET_HOST_TASK_SUMMARY } from "../../../utils/queries";
import EditableHostHeader from "./EditableHostHeader";
import HostStatistics from "./HostStatistics";


const HostContent = () => {
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
        <>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <EditableHostHeader hostId={hostId} loading={loading} error={error} hostData={host} />
            </div>
            <div className="my-2">
                <HostStatistics host={host} taskLoading={taskLoading} taskError={taskError} taskData={taskData} />
            </div>
        </>
    );
};
export default HostContent;
