import { gql, useQuery } from "@apollo/client";
import { Tab, TabList, TabPanel, TabPanels, Tabs } from "@chakra-ui/react";
import React, { useState } from "react";
import { TabFilterTargets } from "../../../components/create-job-drawer/step-select-targets/tab-filter-targets";
import { TabSelectedTargets } from "../../../components/create-job-drawer/step-select-targets/tab-selected-targets";
import { TomeTag } from "../../../utils/consts";
import { TabOptions } from "./tab-options";

export const SelectSessions = () => {
    const [filteredSessions, setFilteredSessions] = useState([])

    const GET_TAGS = gql`
        query get_tags{
            tags {
                id
                name
                kind   
            }
        }
    `;
    const GET_SESSIONS = gql`
        query get_sessions{
            sessions {
                id
                name
                principal
                hostname
                tags {
                    id
                    kind
                    name
                }        
            }
        }
    `;
    const { loading: tagLoading, error: tagError, data: tagData } = useQuery(GET_TAGS);
    const { loading: sessionsLoading, error: sessionsError, data: sessionsData } = useQuery(GET_SESSIONS);

    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-base font-semibold text-gray-900">Select agent sessions</h2>
            <Tabs size='md' variant='enclosed' colorScheme="purple">
                    <TabList>
                        <Tab className="font-semibold py-6">Session options</Tab>
                        <Tab className="font-semibold py-6">Sessions selected</Tab>
                    </TabList>
                    <TabPanels>
                        <TabOptions sessions={sessionsData} setFilteredSessions={setFilteredSessions}  />
                        <TabPanel>
                            Here 2
                        </TabPanel>
                    </TabPanels>
            </Tabs>
             <div className="flex flex-row gap-2">
                <button
                    className="inline-flex items-center rounded-md bg-gray-50 py-3 px-4 text-sm font-semibold text-purple-600 shadow-sm hover:bg-purple-100"
                >
                     Back
                 </button>
                <button
                    className="btn-primary"
                    onClick={() => null}
                    disabled={true}
                >
                    Continue
                </button>
             </div>
        </div>
    );
}