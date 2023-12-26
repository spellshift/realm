import { gql, useQuery } from "@apollo/client";
import { CloseIcon } from "@chakra-ui/icons";
import { Button } from "@chakra-ui/react";
import { Link, useParams } from "react-router-dom";

export const EditablePageHeader = () => {
    const { questId } = useParams();

    const GET_QUEST_NAME = gql`
        query GetQuests($where: QuestWhereInput) {
            quests(where: $where){
                id
                name
            }
        }`;

    const { loading, data, error } = useQuery(GET_QUEST_NAME, {variables: {
        where: {
            id: questId
        }
    }});

    return (
        <div className="flex flex-row justify-between w-full">
            <div className="flex flex-row gap-2 items-center">
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">
                        Quest outputs for
                    </h3>
                    {data?.quests[0]?.name &&
                        <Link to="/results">
                            <Button rightIcon={<CloseIcon />} colorScheme='purple' variant='outline' size="xs">
                                {data?.quests[0]?.name}
                            </Button>
                        </Link>
                    }
                    {(error || (!data?.quests[0]?.name && !loading)) &&
                        <h3 className="text-xl font-semibold leading-6 text-gray-900">Id: {questId}</h3>
                    }
            </div>
        </div>
    );
};