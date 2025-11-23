import { FC } from "react";
import { CreateQuestDropdown } from "../../features/create-quest-dropdown";
import Breadcrumbs from "../../components/Breadcrumbs";
import { useParams } from "react-router-dom";
import { GET_QUEST_BY_ID_QUERY } from "../../utils/queries";
import { useQuery } from "@apollo/client";

export const EditablePageHeader: FC = () => {
    const { questId } = useParams();
    const { data } = useQuery(GET_QUEST_BY_ID_QUERY, {
        variables: {
            "where": {
                "id": questId
            }
        }
    });

    const questsName = data?.quests?.edges[0]?.node?.name || questId;

    const BreadcrumbsList = [
        {
            label: "Quests",
            link: "/quests"
        },
        {
            label: questsName,
            link: `/tasks/${questId}`
        }
    ]

    return (
        <div className="flex flex-col gap-4 w-full">
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
        </div>
    );
};
