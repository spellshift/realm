import {
    ClipboardDocumentCheckIcon,
    ArrowsUpDownIcon,
    BugAntIcon,
    CheckCircleIcon
} from '@heroicons/react/24/outline'
import HostTile from '../../../components/HostTile';
import { HostType } from '../../../utils/consts';

const HostStatistics = (
    { host }:
        { host: HostType }
) => {
    return (
        <div className="grid grid-cols-1 md:grid-cols-3 xl:grid-cols-5  gap-2">
            <div
                className="flex xl:col-span-2 md:col-span-3 col-span-1 flex-row gap-4 rounded-lg bg-white shadow items-center"
            >
                <div className="rounded-md bg-purple-900 p-6">
                    <BugAntIcon className="text-white w-8 h-8" />
                </div>
                <div className='flex flex-col gap-2 px-6'>
                    <HostTile data={host} />
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
                    <p className="text-2xl font-semibold text-gray-900">2/3</p>
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
                    <p className="text-2xl font-semibold text-gray-900">8/10</p>
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
                    <p className="text-2xl font-semibold text-gray-900">9</p>
                </div>
            </div>
        </div>
    );
}
export default HostStatistics;
