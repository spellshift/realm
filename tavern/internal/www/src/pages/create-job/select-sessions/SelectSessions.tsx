import { gql, useQuery } from "@apollo/client";
import { Tab, TabList, TabPanel, TabPanels, Tabs } from "@chakra-ui/react";
import React, { useState } from "react";
import { TabOptions } from "./tab-options";

type Props = {
    setCurrStep: (arg1: number) => void;
    formik: any;
}
export const SelectSessions = (props: Props) => {
    const {setCurrStep, formik} = props;
    const [filteredSessions, setFilteredSessions] = useState([])
    const [selectedSessions, setSelectedSessions] = useState({});

    const GET_TAGS = gql`
        query get_tags($where: TagWhereInput){
        tags(where: $where) {
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

    const SERVICE_PARAMS = {
        variables: { where: { kind: "service" }}
    }
    const GROUP_PARAMS = {
        variables: { where: { kind: "group" }}
    }

    const { loading: serviceTagLoading, error: serviceTagError, data: serviceTagData } = useQuery(GET_TAGS, SERVICE_PARAMS);
    const { loading: groupTagLoading, error: groupTagError, data: groupTagData } = useQuery(GET_TAGS, GROUP_PARAMS);
    const { loading: sessionsLoading, error: sessionsError, data: sessionsData } = useQuery(GET_SESSIONS);

    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-base font-semibold text-gray-900">Select agent sessions</h2>
            {serviceTagLoading || groupTagLoading || sessionsLoading ?
            (
                <div>
                    Loading...
                </div>
            ): (
                <Tabs size='md' variant='enclosed' colorScheme="purple">
                <TabList>
                    <Tab className="font-semibold py-6">Session options</Tab>
                    <Tab className="font-semibold py-6">Sessions selected</Tab>
                </TabList>
                <TabPanels>
                    <TabOptions sessions={sessionsData?.sessions} groups={groupTagData?.tags} services={serviceTagData?.tags} filteredSessions={filteredSessions} setFilteredSessions={setFilteredSessions} selectedSessions={selectedSessions} setSelectedSessions={setSelectedSessions} />
                    <TabPanel>
                        Here 2
                    </TabPanel>
                </TabPanels>
                </Tabs>
            )}
             <div className="flex flex-row gap-2">
                <button
                    className="inline-flex items-center rounded-md bg-gray-50 py-3 px-4 text-sm font-semibold text-purple-600 shadow-sm hover:bg-purple-100"
                    onClick={()=> setCurrStep(0)}
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