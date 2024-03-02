import React from "react";
import { PageWrapper } from "../../components/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { PageNavItem } from "../../utils/enums";
import QuestFormatWrapper from "./components/QuestFormatWrapper";
import QuestHeader from "./components/QuestHeader";
import { useQuests } from "./hooks/useQuests";

export const Quests = () => {
    const {
        data,
        loading,
        error,
        page,
        filtersSelected,
        setPage,
        setSearch,
        setFiltersSelected
    } = useQuests();

    return (
        <PageWrapper currNavItem={PageNavItem.quests}>
            <QuestHeader />
            <div className="p-4 bg-white rounded-lg shadow-lg mt-2">
                Filter goes here
            </div>
            {() => {
                if (loading) {
                    return <EmptyState type={EmptyStateType.loading} label="Loading quests..." />
                }
                if (error) {
                    return <EmptyState type={EmptyStateType.error} label="Error loading quests" />
                }
                return <QuestFormatWrapper data={data} />
            }}
        </PageWrapper>
    );
}
