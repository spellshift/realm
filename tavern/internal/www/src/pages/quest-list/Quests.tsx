import React from "react";
import EmptyStateNoQuests from "../../components/empty-states/EmptyStateNoQuests";
import { PageWrapper } from "../../components/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import FreeTextSearch from "../../components/tavern-base-ui/FreeTextSearch";
import { PageNavItem } from "../../utils/enums";
import QuestFormatWrapper from "./components/QuestFormatWrapper";
import QuestHeader from "./components/QuestHeader";
import { useQuests } from "./hooks/useQuests";

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
    } = useQuests();

    return (
        <PageWrapper currNavItem={PageNavItem.quests}>
            <QuestHeader />
            <div className="p-4 bg-white rounded-lg shadow-lg mt-2">
                <FreeTextSearch setSearch={setSearch} placeholder="Search by tome name, quest name, or output" />
            </div>
            {(loading) ?
                <EmptyState type={EmptyStateType.loading} label="Loading quests..." />
                : error ?
                    <EmptyState type={EmptyStateType.error} label="Error loading quests" />
                    : data?.quests?.edges.length > 0 ?
                        <QuestFormatWrapper
                            data={data?.quests?.edges}
                            totalCount={data?.quests?.totalCount}
                            pageInfo={data?.quests?.pageInfo}
                            page={page}
                            setPage={setPage}
                            updateQuestList={updateQuestList}
                        />
                        :
                        <EmptyStateNoQuests />
            }
        </PageWrapper>
    );
}
export default Quests;
