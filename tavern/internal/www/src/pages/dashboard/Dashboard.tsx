import Breadcrumbs from "../../components/Breadcrumbs";
import Button from "../../components/tavern-base-ui/button/Button";
import PageHeader from "../../components/tavern-base-ui/PageHeader";
import { FileTerminal } from "lucide-react";
import { useCreateQuestModal } from "../../context/CreateQuestModalContext";
import { QuestTimelineChart } from "./QuestTimelineChart";
import { HostByTagCard } from "./HostByTagCard/HostByTagCard";
import { useQuery } from "@apollo/client";
import { GET_TAGS_FOR_DASHBOARD } from "./HostByTagCard/queries";
import { GetTagsForDashboardResponse } from "./HostByTagCard/types";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";

// Can be changed to "service" to display service tags instead
const TAG_KIND_DISPLAY: "group" | "service" = "group";

export const Dashboard = () => {
    const { openModal } = useCreateQuestModal();

    const { data, loading, error } = useQuery<GetTagsForDashboardResponse>(
        GET_TAGS_FOR_DASHBOARD,
        {
            variables: { kind: TAG_KIND_DISPLAY },
            fetchPolicy: "cache-and-network",
        }
    );

    const tags = data?.tags?.edges.map(edge => edge.node) || [];

    return (
        <>
            <div className="flex flex-row justify-between w-full items-center">
                <Breadcrumbs pages={[{
                    label: "Dashboard",
                    link: "/dashboard"
                }]} />
                <div>
                    <Button
                        leftIcon={<FileTerminal className="w-5 h-5" />}
                        buttonStyle={{ color: "purple", size: "md" }}
                        onClick={() => openModal({navigateToQuest: true})}
                    >
                        Create a quest
                    </Button>
                </div>
            </div>
            <PageHeader title="Dashboard" />

            <div className="mt-6 flex flex-col gap-2">
                <QuestTimelineChart />
                {loading && tags.length === 0 ? (
                    <EmptyState type={EmptyStateType.loading} label="Loading tags..." />
                ) : error ? (
                    <EmptyState
                        type={EmptyStateType.error}
                        label="Failed to load tags"
                        details={error.message}
                    />
                ) : tags.length === 0 ? (
                    <EmptyState
                        type={EmptyStateType.noData}
                        label={`No ${TAG_KIND_DISPLAY} tags found`}
                    />
                ) : (
                    <div className="grid grid-cols-4 gap-4">
                        {tags.map((tag) => (
                            <HostByTagCard
                                key={tag.id}
                                tagName={tag.name}
                                tagKind={TAG_KIND_DISPLAY}
                            />
                        ))}
                    </div>
                )}
            </div>
        </>
    );
}
