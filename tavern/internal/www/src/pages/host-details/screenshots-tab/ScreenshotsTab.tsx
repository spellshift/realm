import { useCallback } from "react";
import { useParams } from "react-router-dom";
import { VirtualizedCardList, VirtualizedCardListWrapper } from "../../../components/tavern-base-ui/virtualized-card-list";
import { useHostScreenshotIds } from "./useScreenshotIds";
import { ScreenshotCardVirtualized } from "./ScreenshotCardVirtualized";
import { PageNavItem } from "../../../utils/enums";

export const ScreenshotsTab = () => {
    const { hostId } = useParams();
    const {
        screenshotIds,
        totalCount,
        initialLoading,
        error,
        hasMore,
        loadMore,
    } = useHostScreenshotIds(hostId);

    const renderCard = useCallback(({ itemId, isVisible }: { itemId: string; isVisible: boolean }) => {
        return (
            <ScreenshotCardVirtualized
                key={itemId}
                itemId={itemId}
                isVisible={isVisible}
            />
        );
    }, []);

    return (
        <div className="mt-2">
            <VirtualizedCardListWrapper
                title="Screenshots"
                totalItems={totalCount}
                loading={initialLoading}
                error={error}
                sortType={undefined}
                showFiltering={false}
                cardList={
                    <VirtualizedCardList
                        items={screenshotIds}
                        renderCard={renderCard}
                        hasMore={hasMore}
                        onLoadMore={loadMore}
                        estimateCardSize={400}
                        gap={16}
                        padding="1rem 0"
                    />
                }
            />
        </div>
    );
};
