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
                edges{
                    node{
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

    return (
        <div className="flex flex-row justify-between w-full">
            <div className="flex flex-row gap-2 items-center">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">
                    Quest tasks for
                </h3>
                {data?.quests?.edges[0]?.node?.name &&
                    <Link to="/quests">
                        <Button rightIcon={<CloseIcon />} colorScheme='purple' variant='outline' size="xs">
                            {data?.quests?.edges[0]?.node?.name}
                        </Button>
                    </Link>
                }
                {(error || (!data?.quests?.edges[0]?.node?.name && !loading)) &&
                    <Link to="/quests">
                        <Button rightIcon={<CloseIcon />} colorScheme='purple' variant='outline' size="xs">
                            {questId}
                        </Button>
                    </Link>
                }
            </div>
            {(questId && data?.quests?.edges && data.quests?.edges.length > 0) &&
                <CreateQuestDropdown
                    showLabel={true}
                    name={data?.quests?.edges[0]?.node?.name}
                    originalParms={data?.quests?.edges[0]?.node?.parameters}
                    tome={data?.quests?.edges[0]?.node?.tome}
                    tasks={data?.quests?.edges[0]?.node?.tasks}
                />
            }
        </div>
    );
};
