import { TablePagination, TableWrapper } from "../../components/tavern-base-ui/table";
import { TableRowLimit } from "../../utils/enums";
import QuestHeader from "./components/QuestHeader";
import { QuestTable } from "./components/QuestTable";
import { useQuests } from "./useQuests";

const Quests = () => {
    const {
        data,
        loading,
        initialLoading,
        error,
        page,
        setPage,
        updateQuestList
    } = useQuests(true);

    return (
        <>
            <QuestHeader />
            <TableWrapper
                title="Quests"
                totalItems={data?.quests?.totalCount}
                loading={initialLoading}
                error={error}
                table={<QuestTable quests={data?.quests?.edges || []} />}
                pagination={
                    <TablePagination
                        totalCount={data?.quests?.totalCount || 0}
                        pageInfo={data?.quests?.pageInfo || { hasNextPage: false, hasPreviousPage: false, startCursor: null, endCursor: null }}
                        refetchTable={updateQuestList}
                        page={page}
                        setPage={setPage}
                        rowLimit={TableRowLimit.QuestRowLimit}
                        loading={loading}
                    />
                }
            />
        </>
    );
}
export default Quests;
