import { gql, useQuery } from "@apollo/client";
import { CloseIcon } from "@chakra-ui/icons";
import { Button } from "@chakra-ui/react";
import { Link, useParams } from "react-router-dom";
import { CreateQuestDropdown } from "../../features/create-quest-dropdown";

export const EditablePageHeader = () => {
    const { questId } = useParams();

    const GET_QUEST_NAME = gql`
        query GetQuests($where: QuestWhereInput) {
            quests(where: $where){
                id
                name
                parameters
                tome{
                    id
                    name
                    description
                    eldritch
                    tactic
                    paramDefs
                }
                tasks{
                    beacon{
                        id
                        lastSeenAt
                        interval
                    }
                }
            }
        }`;

    const { loading, data, error } = useQuery(GET_QUEST_NAME, {
        variables: {
            where: {
                id: questId
            }
        }
    });
    console.log(data);

    return (
        <div className="flex flex-row justify-between w-full">
            <div className="flex flex-row gap-2 items-center">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">
                    Quest outputs for
                </h3>
                {data?.quests[0]?.name &&
                    <Link to="/tasks">
                        <Button rightIcon={<CloseIcon />} colorScheme='purple' variant='outline' size="xs">
                            {data?.quests[0]?.name}
                        </Button>
                    </Link>
                }
                {(error || (!data?.quests[0]?.name && !loading)) &&
                    <Link to="/tasks">
                        <Button rightIcon={<CloseIcon />} colorScheme='purple' variant='outline' size="xs">
                            {questId}
                        </Button>
                    </Link>
                }
            </div>
            {(questId && data?.quests && data.quests.length > 0) &&
                <CreateQuestDropdown
                    showLabel={true}
                    name={data?.quests[0]?.name}
                    originalParms={data?.quests[0]?.parameters}
                    tome={data.quests[0].tome}
                    tasks={data.quests[0]?.tasks}
                />
            }
        </div>
    );
};
