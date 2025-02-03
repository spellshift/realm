import React, { useContext } from "react";
import { PageWrapper } from "../../features/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import { TagContext } from "../../context/TagContext";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import QuestForm from "./components/QuestForm";
import EmptyStateNoBeacon from "../../components/empty-states/EmptyStateNoBeacon";

export const CreateQuest = () => {
    const { data, isLoading, error } = useContext(TagContext);

    return (
        <PageWrapper currNavItem={PageNavItem.createQuest}>
            <div className="border-b border-gray-200 pb-6 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Create new quest</h3>
            </div>
            {isLoading ? (
                <EmptyState type={EmptyStateType.loading} label="loading beacon info..." />
            ) : error ? (
                <EmptyState type={EmptyStateType.error} label="Error loading beacon info" />
            ) : data?.beacons && data?.beacons?.length > 0 ? (
                <QuestForm />
            ) : (
                <EmptyStateNoBeacon />
            )}
        </PageWrapper>
    );
}
