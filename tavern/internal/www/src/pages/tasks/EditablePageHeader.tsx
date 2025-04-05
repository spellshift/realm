import { ApolloError } from "@apollo/client";
import { FC } from "react";
import { CreateQuestDropdown } from "../../features/create-quest-dropdown";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";

type EditablePageHeaderProps = {
    questId?: string;
    data: any;
    error?: ApolloError | undefined;
    loading: boolean;
}
export const EditablePageHeader: FC<EditablePageHeaderProps> = ({ questId, data }) => {

    const questsName = data?.quests?.edges[0]?.node?.name || questId;

    const BreadcrumbsList = [
        {
            label: "Quests",
            link: "/quests"
        },
        {
            label: questsName,
            link: `/hosts/${questId}`
        }
    ]

    return (
        <div className="flex flex-col gap-4">
            <div className="flex flex-row justify-between w-full items-center">
                <Breadcrumbs pages={BreadcrumbsList} />
                {(questId && data?.quests?.edges && data.quests?.edges.length > 0) &&
                    <CreateQuestDropdown
                        showLabel={true}
                        name={data?.quests?.edges[0]?.node?.name}
                        originalParms={data?.quests?.edges[0]?.node?.parameters}
                        tome={data?.quests?.edges[0]?.node?.tome}
                        tasks={data?.quests?.edges[0]?.node?.tasksTotal}
                    />
                }
            </div>
            <PageHeader title={data?.quests?.edges[0]?.node?.name || questId} />
        </div>
    );
};
