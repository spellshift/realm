import { PageWrapper } from "../../components/page-wrapper";
import { TablePagination, TableWrapper } from "../../components/tavern-base-ui/table";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import QuestHeader from "./components/QuestHeader";
import { QuestTable } from "./components/QuestTable";
import { useQuests } from "./useQuests";
import { FilterControls, FilterPageType } from "../../context/FilterContext/index";
import { SortingControls } from "../../context/SortContext/index";

const Quests = () => {
    const {
        data,
        loading,
        error,
        page,
        setPage,
        updateQuestList
    } = useQuests(true);

    return (
        <PageWrapper currNavItem={PageNavItem.quests}>
            <QuestHeader />
            <TableWrapper
                title="Quests"
                totalItems={data?.quests?.totalCount}
                loading={loading}
                error={error}
                filterControls={<FilterControls type={FilterPageType.QUEST} />}
                sortingControls={<SortingControls type={PageNavItem.quests} />}
                table={<QuestTable quests={data?.quests?.edges || []} />}
                pagination={
                    <TablePagination
                        totalCount={data?.quests?.totalCount || 0}
                        pageInfo={data?.quests?.pageInfo || { hasNextPage: false, hasPreviousPage: false, startCursor: null, endCursor: null }}
                        refetchTable={updateQuestList}
                        page={page}
                        setPage={setPage}
                        rowLimit={TableRowLimit.QuestRowLimit}
                    />
                }
            />
        </PageWrapper>
    );
}
export default Quests;
