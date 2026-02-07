import { useCallback } from "react";
import { useParams } from "react-router-dom";
import { VirtualizedCardList, VirtualizedCardListWrapper } from "../../components/tavern-base-ui/virtualized-card-list";
import { useTaskIds } from "./useTaskIds";
import { TaskCardVirtualized } from "./TaskCardVirtualized";
import { EditablePageHeader } from "./EditablePageHeader";

const Tasks = () => {
    const { questId } = useParams();
    const {
        taskIds,
        totalCount,
        initialLoading,
        error,
        hasMore,
        loadMore,
    } = useTaskIds(questId);

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
        <>
            <EditablePageHeader />
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
        </>
    );
};

export default Tasks;
