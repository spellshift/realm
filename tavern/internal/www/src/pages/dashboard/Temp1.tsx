import { XCircleIcon } from "lucide-react";

const HostByTagCard = () => {
    return (
        <div className="bg-white rounded-lg border border-gray-200 p-4 flex flex-col gap-4">
            <div className="flex flex-col gap-1">
                <h1 className="text-lg font-semibold text-gray-900">Group 1</h1>
                <div className="flex flex-row justify-between items-center ">
                    <h2 className="text-sm"> 68 active beacons</h2>
                    <h2 className="text-sm"> 20 hosts </h2>
                </div>
            </div>
            <div className="flex flex-row flex-wrap gap-1">
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-red-600 w-4 h-4 rounded-full"/>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-red-600">-1</div>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-red-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-green-600">+4</div>
                    <div className="text-sm text-red-600">-1</div>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    <XCircleIcon className="w-4 h-4 text-red-600"/>
                    <div className="text-sm text-red-600">Host lost</div>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-red-600 w-4 h-4 rounded-full"/>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-red-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-green-600">+2</div>
                    <div className="text-sm text-red-600">-2</div>
                </div>
                    <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-red-600 w-4 h-4 rounded-full"/>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    <XCircleIcon className="w-4 h-4 text-red-600"/>
                    <div className="text-sm text-red-600">Host lost</div>
                </div>
                                            <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-red-600 w-4 h-4 rounded-full"/>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-red-600">-1</div>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-red-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-green-600">+4</div>
                    <div className="text-sm text-red-600">-1</div>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    <XCircleIcon className="w-4 h-4 text-red-600"/>
                    <div className="text-sm text-red-600">Host lost</div>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-red-600 w-4 h-4 rounded-full"/>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-red-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-green-600">+2</div>
                    <div className="text-sm text-red-600">-2</div>
                </div>
                    <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    {/* Each div represents a beacon on a host*/}
                    <div className="bg-green-600 w-4 h-4 rounded-full"/>
                    <div className="bg-red-600 w-4 h-4 rounded-full"/>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center">
                    <XCircleIcon className="w-4 h-4 text-red-600"/>
                    <div className="text-sm text-red-600">Host lost</div>
                </div>
            </div>
        </div>
)
}
export default HostByTagCard;