import { ArrowDownToLine, ChevronLeft, ChevronRight } from "lucide-react";
import { useCallback, useMemo } from "react";
import { useParams } from "react-router-dom";
import Button from "../../../components/tavern-base-ui/button/Button";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import Tooltip from "../../../components/tavern-base-ui/Tooltip";
import { VirtualizedCardList } from "../../../components/tavern-base-ui/virtualized-card-list";
import { SortingControls } from "../../../context/SortContext";
import { PageNavItem } from "../../../utils/enums";
import { formatBytes } from "../../../utils/utils";
import { useScreenshots } from "./useScreenshots";
import { capturedAt, downloadAllURL, imageURL, sourceLabel, thumbnailURL } from "./utils";

const ScreenshotsTab = () => {
    const { hostId } = useParams();
    const {
        screenshots,
        screenshotIds,
        totalCount,
        selected,
        selectedIndex,
        setSelectedId,
        selectPrevious,
        selectNext,
        canSelectPrevious,
        canSelectNext,
        hasMore,
        loadMore,
        initialLoading,
        error,
    } = useScreenshots(hostId || "");

    const screenshotsById = useMemo(
        () => new Map(screenshots.map(screenshot => [screenshot.id, screenshot])),
        [screenshots]
    );

    const renderThumbnail = useCallback(({ itemId }: { itemId: string }) => {
        const screenshot = screenshotsById.get(itemId);
        if (!screenshot) {
            return null;
        }
        return (
            <div className={`flex h-[61px] items-center gap-2 px-2 py-2 border-b border-gray-100 ${screenshot.id === selected?.id ? "bg-purple-50 border-l-2 border-l-purple-600" : "hover:bg-gray-50"}`}>
                <button
                    type="button"
                    onClick={() => setSelectedId(screenshot.id)}
                    aria-label={`Preview ${screenshot.name}`}
                    aria-current={screenshot.id === selected?.id ? "true" : undefined}
                    className="flex items-center gap-2 min-w-0 flex-1 text-left"
                >
                    <img src={thumbnailURL(screenshot.id)} alt="" loading="lazy" className="w-16 h-11 object-contain bg-gray-100 rounded-sm shrink-0" />
                    <span className="min-w-0 flex-1">
                        <span className="block truncate text-xs text-gray-900" title={screenshot.name}>{screenshot.name}</span>
                        <span className="block truncate text-xs text-gray-600">{capturedAt(screenshot.createdAt)} · {formatBytes(screenshot.size)}</span>
                    </span>
                </button>
                <Tooltip label={`Download ${screenshot.name}`}>
                    <a href={imageURL(screenshot.id)} download={screenshot.name} aria-label={`Download ${screenshot.name}`} className="p-1 rounded text-gray-600 hover:bg-gray-100 hover:text-purple-700">
                        <ArrowDownToLine className="w-4 h-4" />
                    </a>
                </Tooltip>
            </div>
        );
    }, [screenshotsById, selected?.id, setSelectedId]);

    return (
        <div className="flex flex-col w-full gap-4 mt-2">
            <div className="flex flex-wrap justify-between items-center gap-3 border-b border-gray-200 bg-white py-2">
                <div className="flex items-center gap-2">
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">Screenshots</h3>
                    {totalCount !== undefined && <span className="text-gray-600">({totalCount})</span>}
                </div>
                <div className="flex flex-wrap items-center justify-end gap-2">
                    <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "gray", size: "sm" }}
                        leftIcon={<ArrowDownToLine className="w-4 h-4" />}
                        disabled={!hostId}
                        onClick={() => window.location.assign(downloadAllURL(hostId || ""))}
                    >
                        Download all
                    </Button>
                    <SortingControls sortType={PageNavItem.screenshots} />
                </div>
            </div>

            {(error || !hostId) && <EmptyState type={EmptyStateType.error} label="Error loading data" details={error?.message || "Host ID not found"} />}
            {!error && hostId && initialLoading && <EmptyState type={EmptyStateType.loading} label="Loading screenshots..." />}
            {!error && hostId && !initialLoading && totalCount === 0 && (
                <EmptyState type={EmptyStateType.noData} label="No screenshots reported" />
            )}

            {!error && screenshots.length > 0 && selected && (
                <div className="grid grid-cols-1 lg:grid-cols-[minmax(260px,300px)_minmax(0,1fr)] gap-4 lg:min-h-[60vh] [--screenshot-list-height:min(25dvh,220px)] lg:[--screenshot-list-height:60vh] [--screenshot-list-min-height:0px] lg:[--screenshot-list-min-height:320px]">
                    <div className="order-2 lg:order-1 border border-gray-200 rounded-md overflow-hidden flex flex-col min-h-0">
                        <VirtualizedCardList
                            items={screenshotIds}
                            renderCard={renderThumbnail}
                            hasMore={hasMore}
                            onLoadMore={loadMore}
                            estimateCardSize={61}
                            dynamicSizing={false}
                            gap={0}
                            padding="0"
                            height="var(--screenshot-list-height)"
                            minHeight="var(--screenshot-list-min-height)"
                        />
                    </div>

                    <div className="order-1 lg:order-2 border border-gray-200 rounded-md overflow-hidden flex flex-col min-w-0 h-[30dvh] min-h-[220px] lg:h-[60vh] lg:min-h-[320px]">
                        <div className="flex items-center justify-between gap-2 px-4 py-2 border-b border-gray-200 text-sm">
                            <span className="truncate font-medium text-gray-900" title={selected.name}>{selected.name}</span>
                            <div className="flex items-center gap-1 shrink-0">
                                <span className="text-xs text-gray-600 mr-2">{selectedIndex + 1} of {totalCount}</span>
                                <Button buttonVariant="ghost" buttonStyle={{ color: "gray", size: "square_sm" }} leftIcon={<ChevronLeft className="w-4 h-4" />} aria-label="Previous screenshot" disabled={!canSelectPrevious} onClick={selectPrevious} />
                                <Button buttonVariant="ghost" buttonStyle={{ color: "gray", size: "square_sm" }} leftIcon={<ChevronRight className="w-4 h-4" />} aria-label="Next screenshot" disabled={!canSelectNext} onClick={selectNext} />
                                <Tooltip label={`Download ${selected.name}`}>
                                    <a href={imageURL(selected.id)} download={selected.name} aria-label={`Download ${selected.name}`} className="p-1 rounded text-gray-600 hover:bg-gray-100 hover:text-purple-700">
                                        <ArrowDownToLine className="w-4 h-4" />
                                    </a>
                                </Tooltip>
                            </div>
                        </div>
                        <div className="flex-1 min-h-0 overflow-hidden bg-gray-100 flex items-center justify-center p-4">
                            <img src={imageURL(selected.id)} alt={selected.name} className="max-w-full max-h-full object-contain shadow-sm" />
                        </div>
                        <div className="flex flex-wrap gap-x-6 gap-y-1 px-4 py-3 border-t border-gray-200 text-xs text-gray-600">
                            <span>Captured <b className="text-gray-900 font-medium">{capturedAt(selected.createdAt)}</b></span>
                            <span>Source <b className="text-gray-900 font-medium">{sourceLabel(selected)}</b></span>
                            <span>Size <b className="text-gray-900 font-medium">{formatBytes(selected.size)}</b></span>
                        </div>
                    </div>
                </div>
            )}
            {!error && !initialLoading && totalCount !== undefined && screenshots.length === 0 && totalCount > 0 && (
                <EmptyState type={EmptyStateType.noData} label="No screenshots loaded" details="Try refreshing the page." />
            )}
        </div>
    );
};

export default ScreenshotsTab;
