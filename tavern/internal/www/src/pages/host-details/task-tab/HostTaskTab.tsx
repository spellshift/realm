import { useCallback } from "react";
import { useParams } from "react-router-dom";
import { VirtualizedCardList, VirtualizedCardListWrapper } from "../../../components/tavern-base-ui/virtualized-card-list";
import { useHostTaskIds } from "./useHostTaskIds";
import { TaskCardVirtualized } from "../../tasks/TaskCardVirtualized";

const HostTaskTab = () => {
    const { hostId } = useParams();
    const {
        taskIds,
        totalCount,
        initialLoading,
        error,
        hasMore,
        loadMore,
    } = useHostTaskIds(hostId);

    const renderCard = useCallback(({ itemId, isVisible }: { itemId: string; isVisible: boolean }) => {
        return (
            <TaskCardVirtualized
                key={itemId}
                itemId={itemId}
                isVisible={isVisible}
            />
        );
    }, []);

    return (
        <div className="mt-2">
        <VirtualizedCardListWrapper
            title="Tasks"
            totalItems={totalCount}
            loading={initialLoading}
            error={error}
            showSorting={true}
            showFiltering={true}
            cardList={
                <VirtualizedCardList
                    items={taskIds}
                    renderCard={renderCard}
                    hasMore={hasMore}
                    onLoadMore={loadMore}
                    estimateCardSize={360}
                    gap={16}
                    padding="1rem 0"
                />
            }
        />
        </div>
    );
};

export default HostTaskTab;
