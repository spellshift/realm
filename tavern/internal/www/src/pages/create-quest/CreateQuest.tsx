import React, { useContext } from "react";
import { PageWrapper } from "../../components/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import { TagContext } from "../../context/TagContext";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { QuestForm } from "./quest-form";

export const CreateQuest = () => {
    const {data, isLoading, error } = useContext(TagContext);

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
                <EmptyState type={EmptyStateType.noData} label="No beacons found" details="Get started by deploying an imix agent on your target system.">
                    <button
                        type="button"
                        className="inline-flex items-center rounded-md bg-purple-700 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-purple-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-700"
                        onClick={() => window.open("https://docs.realm.pub/user-guide/getting-started#start-the-agent", '_blank')}  
                    >
                        See imix docs
                    </button>
                </EmptyState>
            )}
        </PageWrapper>
    );
}