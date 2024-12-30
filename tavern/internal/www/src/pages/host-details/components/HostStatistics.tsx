import { ApolloError } from '@apollo/client';
import {
    ClipboardDocumentCheckIcon,
    ArrowsUpDownIcon,
    BugAntIcon,
    CheckCircleIcon,
    KeyIcon,
} from '@heroicons/react/24/outline'
import HostTile from '../../../components/HostTile';
import { HostType } from '../../../utils/consts';
import { getOfflineOnlineStatus } from '../../../utils/utils';

const HostStatistics = (
    {
        host,
        taskLoading,
        taskError,
        taskData
    }:
        {
            host: HostType | null;
            taskLoading: boolean;
            taskError: ApolloError | undefined;
            taskData: any;
        }
) => {

    const getTaskCounts = (edges: any) => {
        return edges.reduce(
            (accumulator: any, currentValue: any) => {
                const task = currentValue?.node;
                if (task.outputSize > 0) {
                    accumulator.hasOutputCount += 1;
                }
                if (task.execFinishedAt) {
                    accumulator.hasFinishedCount += 1;
                }
                return accumulator;
            },
            {
                hasFinishedCount: 0,
                hasOutputCount: 0
            },
        );
    };

    const beaconStatuses = getOfflineOnlineStatus(host?.beacons || []);
    const taskCounts = getTaskCounts(taskData?.tasks?.edges || []);

    return (
        <div className="grid grid-cols-1 md:grid-cols-3 xl:grid-cols-6 gap-2 py-4">
            <div
                className="flex xl:col-span-2 md:col-span-3 col-span-1 flex-row gap-4 rounded-lg bg-white shadow items-center"
            >
                <div className="rounded-md bg-purple-900 p-6">
                    <BugAntIcon className="text-white w-8 h-8" />
                </div>
                <div className='flex flex-col gap-2'>
                    {host ?
                        <HostTile data={host} />
                        : "-"
                    }
                </div>
            </div>
            <div
                className="flex flex-row gap-4 rounded-lg bg-white shadow items-center"
            >
                <div className="rounded-md bg-purple-900 p-6">
                    <ArrowsUpDownIcon className="text-white w-8 h-8" />
                </div>
                <div className='flex flex-col gap-2'>
                    <p className="truncate text-sm font-medium text-gray-500">Active beacons</p>
                    <p className="text-2xl font-semibold text-gray-900">
                        {!host ? "-" : `${beaconStatuses.online} / ${beaconStatuses.online + beaconStatuses.offline}`}
                    </p>
                </div>
            </div>
            <div
                className="flex flex-row gap-4 rounded-lg bg-white shadow items-center"
            >
                <div className="rounded-md bg-purple-900 p-6">
                    <CheckCircleIcon className="text-white w-8 h-8" />
                </div>
                <div className='flex flex-col gap-2'>
                    <p className="truncate text-sm font-medium text-gray-500">Tasks finished</p>
                    <p className="text-2xl font-semibold text-gray-900">
                        {taskLoading || taskError || !taskData?.tasks?.edges ? "-" : `${taskCounts.hasFinishedCount} / ${taskData?.tasks?.totalCount}`}
                    </p>
                </div>
            </div>
            <div
                className="flex flex-row gap-4 rounded-lg bg-white shadow items-center"
            >
                <div className="rounded-md bg-purple-900 p-6">
                    <ClipboardDocumentCheckIcon className="text-white w-8 h-8" />
                </div>
                <div className='flex flex-col gap-2'>
                    <p className="truncate text-sm font-medium text-gray-500">Tasks with output</p>
                    <p className="text-2xl font-semibold text-gray-900">
                        {taskLoading || taskError || !taskData?.tasks?.edges ? "-" : `${taskCounts.hasOutputCount} / ${taskData?.tasks?.totalCount}`}
                    </p>
                </div>
            </div>
            <div
                className="flex flex-row gap-4 rounded-lg bg-white shadow items-center"
            >
                <div className="rounded-md bg-purple-900 p-6">
                    <KeyIcon className="text-white w-8 h-8" />
                </div>
                <div className='flex flex-col gap-2'>
                    <p className="truncate text-sm font-medium text-gray-500">Credentials</p>
                    <p className="text-2xl font-semibold text-gray-900">
                        {!host ? "-" : `${host.credentials!.length || 0}`}
                    </p>
                </div>
            </div>
        </div>
    );
}
export default HostStatistics;
