import { useState, useEffect } from "react";
import { PlugIcon, DownloadIcon } from "lucide-react";
import { VirtualizedTableWrapper } from "../../../components/tavern-base-ui/virtualized-table";
import { PortalsTable } from "./PortalsTable";
import { usePortalIds } from "./usePortalIds";
import { useParams } from "react-router-dom";
import Button from "../../../components/tavern-base-ui/button/Button";

const PortalsHeader = () => {
    const [downloadAsset, setDownloadAsset] = useState<string>("linux/socks5");

    useEffect(() => {
        const ua = navigator.userAgent.toLowerCase();
        if (ua.includes("win")) {
            setDownloadAsset("windows/socks5.exe");
        } else if (ua.includes("mac")) {
            setDownloadAsset("macos/socks5");
        } else if (ua.includes("linux")) {
            setDownloadAsset("linux/socks5");
        } else if (ua.includes("bsd")) {
            setDownloadAsset("bsd/socks5");
        }
    }, []);

    return (
        <div className="flex flex-col flex-1 bg-white">
            <div className="px-4 py-3 bg-gray-50 flex justify-between items-center shadow-sm border-b border-gray-200">
                <div className="flex items-center text-sm text-gray-600 gap-2">
                    <PlugIcon className="w-5 h-5 text-gray-400" />
                    <span>Download the SOCKS5 proxy client for your platform</span>
                </div>
                <a href={`/assets/download/${downloadAsset}`} download onClick={(e) => e.stopPropagation()}>
                    <Button buttonStyle={{ color: "purple", size: 'sm' }} buttonVariant="solid">
                        <DownloadIcon className="w-4 h-4 mr-2" />
                        Download socks5
                    </Button>
                </a>
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
