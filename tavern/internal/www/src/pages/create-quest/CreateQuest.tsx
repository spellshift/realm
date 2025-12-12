import React, { useContext } from "react";
import { PageWrapper } from "../../features/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import { TagContext } from "../../context/TagContext";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import QuestForm from "./components/QuestForm";
import EmptyStateNoBeacon from "../../components/empty-states/EmptyStateNoBeacon";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";

export const CreateQuest = () => {
    const { data, isLoading, error } = useContext(TagContext);

    return (
        <PageWrapper currNavItem={PageNavItem.createQuest}>
            <Breadcrumbs pages={[{
                label: "Create new quest",
                link: "/createQuest"
            }]} />
            <PageHeader title="Create new quest" />
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
