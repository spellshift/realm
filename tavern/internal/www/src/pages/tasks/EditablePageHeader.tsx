import { FC } from "react";
import Breadcrumbs from "../../components/Breadcrumbs";
import { useParams } from "react-router-dom";
import { useQuery } from "@apollo/client";
import { CreateQuestDropdown } from "../../components/create-quest-dropdown";
import { QuestQueryTopLevel } from "../../utils/interfacesQuery";
import { GET_QUEST_DETAIL_QUERY } from "../quests/queries";

export const EditablePageHeader: FC = () => {
    const { questId } = useParams();

    const { data } = useQuery<QuestQueryTopLevel>(GET_QUEST_DETAIL_QUERY, {
        variables: {
            id: questId,
        },
        skip: !questId
    });

    const questData = data?.quests?.edges?.[0]?.node;

    const BreadcrumbsList = [
        {
            label: "Quests",
            link: "/quests"
        },
        {
            label: questData?.name || "Quest",
            link: `/tasks/${questId}`
        }
    ]

    return (
        <div className="flex flex-col gap-4 w-full">
            <div className="flex flex-row justify-between w-full items-center gap-2">
                <Breadcrumbs pages={BreadcrumbsList} />
                {questData && questData.tasks.edges && (
                    <CreateQuestDropdown
                        showLabel={true}
                        name={questData.name}
                        originalParms={questData.parameters || ""}
                        tome={questData.tome}
                        tasks={questData.tasks}
                    />
                )}
            </div>
        </div>
    );
};
