import React from "react";
import EmptyStateNoQuests from "../../components/empty-states/EmptyStateNoQuests";
import FilterBar from "../../components/FilterBar";
import { PageWrapper } from "../../components/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import TablePagination from "../../components/tavern-base-ui/TablePagination";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import QuestHeader from "./components/QuestHeader";
import { QuestTable } from "./components/QuestTable";
import { useQuests } from "../../hooks/useQuests";

const Quests = () => {
    const {
        data,
        loading,
        error,
        page,
        filtersSelected,
        setPage,
        setSearch,
        setFiltersSelected,
        updateQuestList
    } = useQuests(true);

    return (
        <PageWrapper currNavItem={PageNavItem.quests}>
            <QuestHeader />
            <div className="bg-white  mt-2">
                <FilterBar setSearch={setSearch} searchPlaceholder="Search by quest or tome name" filtersSelected={filtersSelected} setFiltersSelected={setFiltersSelected} />
            </div>
            {(loading) ?
                <EmptyState type={EmptyStateType.loading} label="Loading quests..." />
                : error ?
                    <EmptyState type={EmptyStateType.error} label="Error loading quests" />
                    : data?.quests?.edges.length > 0 ?
                        <div className="py-4 mt-2 flex flex-col gap-1 w-full">
                            <QuestTable quests={data?.quests?.edges} filtersSelected={filtersSelected} />
                            <TablePagination totalCount={data?.quests?.totalCount} pageInfo={data?.quests?.pageInfo} refetchTable={updateQuestList} page={page} setPage={setPage} rowLimit={TableRowLimit.QuestRowLimit} />
                        </div>
                        :
                        <EmptyStateNoQuests />
            }
        </PageWrapper>
    );
}
export default Quests;
