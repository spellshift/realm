import { VirtualizedTableWrapper } from "../../components/tavern-base-ui/virtualized-table";
import QuestHeader from "./QuestHeader";
import { QuestsTable } from "./QuestsTable";
import { useQuestIds } from "./useQuestIds";

const Quests = () => {
    const {
        data,
        questIds,
        initialLoading,
        error,
        hasMore,
        loadMore,
    } = useQuestIds();

    return (
        <>
            <QuestHeader />
            <VirtualizedTableWrapper
                title="Quests"
                totalItems={data?.quests?.totalCount}
                loading={initialLoading}
                error={error}
                table={
                    <QuestsTable
                        questIds={questIds}
                        hasMore={hasMore}
                        onLoadMore={loadMore}
                    />
                }
            />
        </>
    );
}

export default Quests;
