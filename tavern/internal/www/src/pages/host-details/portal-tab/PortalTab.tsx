import { Fragment } from "react";
import { Menu, Transition } from "@headlessui/react";
import { ChevronDownIcon } from "@heroicons/react/20/solid";
import { PlugIcon, DownloadIcon } from "lucide-react";
import { VirtualizedTableWrapper } from "../../../components/tavern-base-ui/virtualized-table";
import { PortalsTable } from "./PortalsTable";
import { usePortalIds } from "./usePortalIds";
import { useParams } from "react-router-dom";
import Button from "../../../components/tavern-base-ui/button/Button";
import { ghostButton } from "../../../components/tavern-base-ui/button/ButtonStyles";
import { CreatePortalQuestButton } from "./CreatePortalQuestButton";

const socks5Downloads = [
    { platform: "Linux", asset: "linux/socks5" },
    { platform: "macOS", asset: "macos/socks5" },
    { platform: "Windows", asset: "windows/socks5.exe" },
    { platform: "FreeBSD", asset: "bsd/socks5" },
];

export const PortalsHeader = () => {
    return (
        <div className="flex flex-col flex-1 bg-white">
            <div className="px-4 py-3 bg-gray-50 flex justify-between items-center shadow-sm border-b border-gray-200">
                <div className="flex items-center text-sm text-gray-600 gap-2">
                    <PlugIcon className="w-5 h-5 text-gray-400" />
                    <span>Download the SOCKS5 proxy client for your platform</span>
                </div>
                <Menu as="div" className="relative">
                    <Menu.Button
                        as={Button}
                        buttonStyle={{ color: "purple", size: "sm" }}
                        rightIcon={<ChevronDownIcon className="h-5 w-5" aria-hidden="true" />}
                    >
                        <DownloadIcon className="h-4 w-4 mr-2" aria-hidden="true" />
                        Download socks5
                    </Menu.Button>
                    <Transition
                        as={Fragment}
                        enter="transition ease-out duration-100"
                        enterFrom="transform opacity-0 scale-95"
                        enterTo="transform opacity-100 scale-100"
                        leave="transition ease-in duration-75"
                        leaveFrom="transform opacity-100 scale-100"
                        leaveTo="transform opacity-0 scale-95"
                    >
                        <Menu.Items className="absolute right-0 mt-2 w-40 origin-top-right divide-y divide-gray-100 rounded-md bg-white shadow-lg ring-1 ring-black/5 focus:outline-none z-10">
                            <div className="px-1 py-1">
                                {socks5Downloads.map(({ platform, asset }) => (
                                    <Menu.Item
                                        key={platform}
                                        as="a"
                                        href={`/assets/download/${asset}`}
                                        download
                                        className={ghostButton({ color: "gray", size: "sm", className: "w-full data-[focus]:bg-gray-100" })}
                                    >
                                        {platform}
                                    </Menu.Item>
                                ))}
                            </div>
                        </Menu.Items>
                    </Transition>
                </Menu>
            </div>
        </div>
    );
};

const PortalTab = () => {
    const { hostId } = useParams();

    const {
        data,
        portalIds,
        initialLoading,
        error,
        hasMore,
        loadMore,
    } = usePortalIds(hostId || "");

    return (
        <div className="mt-2">
            <div className="mb-4">
                <PortalsHeader />
            </div>
            <VirtualizedTableWrapper
                title="Portals"
                totalItems={data?.portals?.totalCount}
                loading={initialLoading}
                error={error}
                showFiltering={false}
                actions={<CreatePortalQuestButton variant="ghost" size="sm" />}
                emptyStateAction={<CreatePortalQuestButton variant="solid" size="md" />}
                table={
                    <PortalsTable
                        portalIds={portalIds}
                        hasMore={hasMore}
                        onLoadMore={loadMore}
                    />
                }
            />
        </div>
    );
}

export default PortalTab;
