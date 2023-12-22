import { useQuery } from "@apollo/client";
import { Accordion, AccordionButton, AccordionIcon, AccordionItem, AccordionPanel, Box } from "@chakra-ui/react";
import React from "react";
import { PageWrapper } from "../../components/page-wrapper";
import { GET_QUEST_QUERY } from "../../utils/queries";
import AccordinanQuestView from "./AccordianQuestView";

const Quests = () => {
    const { loading, data, error } = useQuery(GET_QUEST_QUERY);

    return (
        <PageWrapper>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Quest list</h3>
            </div>
                {loading ? (
                    <div>loading</div> 
                ) : error ? (
                    <div>error</div> 
                ) : (data && data.quests.length > 0 ) ? (
                    <AccordinanQuestView data={data.quests} />
                ) : (
                    <div>No results</div>
                )
            }
        </PageWrapper>
    );
};
export default Quests;