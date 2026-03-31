import { Bug, XCircleIcon } from "lucide-react";
import { Link } from "react-router-dom";

const HostByTagCard = () => {
    return (
        <div className="bg-white rounded-lg border border-gray-200 p-4 flex flex-col gap-1">
            <div className="flex flex-col gap-1">
                <h1 className="text-lg font-semibold text-gray-900">Group 1</h1>
                <div className="flex flex-row justify-between items-center ">
                    <h2 className="text-sm"> 68 active beacons</h2>
                    <h2 className="text-sm"> 20 hosts </h2>
                </div>
            </div>
            <div className="flex flex-row flex-wrap gap-1">
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center font-semibold ">
                    {/* Each div represents a beacon on a host*/}
                    <Link
                        to={`/hosts/test}`}
                        className="text-purple-800 text-sm underline cursor-pointer pr-2 text-wrap"
                    >
                        coder-hulto-rea
                    </Link>
                    <Bug fill="currentColor" className="text-purple-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-purple-600">3</div>
                    <Bug fill="currentColor" className="text-red-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-red-600">-1</div>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center font-semibold ">
                    {/* Each div represents a beacon on a host*/}
                    <Link
                        to={`/hosts/test}`}
                        className="text-purple-800 text-sm underline cursor-pointer pr-2 text-wrap"
                    >
                        mustang.team8.wildwest.remote
                    </Link>
                    <Bug fill="currentColor" className="text-purple-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-purple-600">10</div>
                    <Bug fill="currentColor" className="text-red-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-red-600">-1</div>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center font-semibold ">
                    {/* Each div represents a beacon on a host*/}
                    <Link
                        to={`/hosts/test}`}
                        className="text-purple-800 text-sm underline cursor-pointer pr-2 text-wrap"
                    >
                        coder-hulto-alt
                    </Link>
                    <Bug fill="currentColor" className="text-purple-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-purple-600">2</div>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center font-semibold ">
                    {/* Each div represents a beacon on a host*/}
                    <Link
                        to={`/hosts/test}`}
                        className="text-purple-800 text-sm underline cursor-pointer pr-2 text-wrap"
                    >
                        mustang.team8.wildwest.local
                    </Link>
                    <Bug fill="currentColor" className="text-purple-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-purple-600">8</div>
                    <Bug fill="currentColor" className="text-red-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-red-600">-2</div>
                </div>
                <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center font-semibold ">
                    {/* Each div represents a beacon on a host*/}
                    <Link
                        to={`/hosts/test}`}
                        className="text-purple-800 text-sm underline cursor-pointer pr-2 text-wrap"
                    >
                        mustang.team8.wildwest.prod
                    </Link>
                    <XCircleIcon className="text-red-600 w-4 h-4 rounded-full"/>
                    <div className="text-sm text-red-600">Lost</div>
                </div>
            </div>
        </div>
    );
}
export default HostByTagCard;