import React from "react";
import EmptyStateNoQuests from "../../components/empty-states/EmptyStateNoQuests";
import { PageWrapper } from "../../features/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import TablePagination from "../../components/tavern-base-ui/TablePagination";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import QuestHeader from "./components/QuestHeader";
import { QuestTable } from "./components/QuestTable";
import { useQuests } from "./useQuests";
import FilterControls, { FilterPageType } from "../../components/filter-controls";

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
            <div className="bg-gray-50 rounded-md">
                <div className="flex flex-row justify-between items-end px-4 py-2 border-b border-gray-200 pb-5">
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">Quests</h3>
                    <div className="flex flex-row justify-end">
                        {/* Sorting not added yet */}
                        {/* <Button leftIcon={<Bars3BottomLeftIcon className="w-4" />} buttonVariant="ghost" buttonStyle={{ color: 'gray', size: "md" }} onClick={() => console.log("hi")}>Sort</Button> */}
                        <FilterControls type={FilterPageType.QUEST} />
                    </div>
                </div>
                {(loading) ?
                    <EmptyState type={EmptyStateType.loading} label="Loading quests..." />
                    : error ?
                        <EmptyState type={EmptyStateType.error} label="Error loading quests" />
                        : data?.quests?.edges.length > 0 ?
                            <div className="flex flex-col gap-1 w-full">
                                <QuestTable quests={data?.quests?.edges} />
                                <TablePagination totalCount={data?.quests?.totalCount} pageInfo={data?.quests?.pageInfo} refetchTable={updateQuestList} page={page} setPage={setPage} rowLimit={TableRowLimit.QuestRowLimit} />
                            </div>
                            :
                            <EmptyStateNoQuests />
                }
            </div>
        </PageWrapper>
    );
}
export default Quests;
