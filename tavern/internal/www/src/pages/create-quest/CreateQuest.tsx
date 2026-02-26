import { useTags } from "../../context/TagContext";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import QuestForm from "./components/QuestForm";
import EmptyStateNoBeacon from "../../components/empty-states/EmptyStateNoBeacon";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";

export const CreateQuest = () => {
    const { data, isLoading, error } = useTags();

    const isDataLoading = isLoading || !data?.beacons;
    const hasBeacons = data.beacons && data.beacons.length > 0;

    return (
        <>
            <Breadcrumbs pages={[{
                label: "Create new quest",
                link: "/createQuest"
            }]} />
            <PageHeader title="Create new quest" />
            {error ? (
                <EmptyState type={EmptyStateType.error} label="Error loading beacon info" />
            ) : isDataLoading ? (
                <EmptyState type={EmptyStateType.loading} label="loading beacon info..." />
            ) : hasBeacons ? (
                <QuestForm />
            ) : (
                <EmptyStateNoBeacon />
            )}
        </>
    );
}
