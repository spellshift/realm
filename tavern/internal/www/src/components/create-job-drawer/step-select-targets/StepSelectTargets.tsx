import { Badge, Box, Button, Divider, Heading, ListItem, Stack, StackItem, Tab, TabList, TabPanel, TabPanels, Tabs, Text, Textarea, Icon, Grid, GridItem, Flex, Container, Checkbox, Card, CardBody, Spacer } from "@chakra-ui/react";
import React, { useState } from "react";
import Select from "react-select";
import { StepIcon } from "../step-icon";
import {MdFilterList} from "react-icons/md";
import { m } from "framer-motion";
import { TabFilterTargets } from "./tab-filter-targets";

type StepSelectTargetsParams = {
    step: number;
    currStep: number;
    setCurrStep: any;
}
export const StepSelectTargets = (props: StepSelectTargetsParams) => {
    const {step, currStep, setCurrStep} = props;

    const handleNext = () => {
        setCurrStep(step +1);
    };

    const handleBack = () => {
        setCurrStep(step -1);
    }

    const sessions = [
        {
            id: "15b9ec70-b3db-11ed-afa1-0242ac120002",
            principal: "admin",
            hostname:  "15b9ec70-b3db-11ed-afa1-0242ac120002",
            identifier: "",
            agentidentifier: "",
            hostidentifier: "",
            lastseenat: "",
            tags: [{
                id: "Relay",
                name: "Relay",
                kind: "service",
                sessions: []
                },
                {
                    id: "Team 1",
                    name: "Team 1",
                    kind: "group",
                    sessions: []
                }],
            tasks: [],
        },
        {
            id: "15b9f04e-b3db-11ed-afa1-0242ac120002",
            principal: "admin",
            hostname:  "15b9f04e-b3db-11ed-afa1-0242ac120002",
            identifier: "",
            agentidentifier: "",
            hostidentifier: "",
            lastseenat: "",
            tags: [{
                id: "Web",
                name: "Web",
                kind: "service",
                sessions: []
                },
                {
                    id: "Team 1",
                    name: "Team 1",
                    kind: "group",
                    sessions: []
                }],
            tasks: [],
        },
        {
            id: "15b9f99a-b3db-11ed-afa1-0242ac120002",
            principal: "admin",
            hostname:  "15b9f99a-b3db-11ed-afa1-0242ac120002",
            identifier: "",
            agentidentifier: "",
            hostidentifier: "",
            lastseenat: "",
            tags: [{
                id: "Relay",
                name: "Relay",
                kind: "service",
                sessions: []
                },
                {
                    id: "Team 1",
                    name: "Team 1",
                    kind: "group",
                    sessions: []
                }],
            tasks: [],
        },
        {
            id: "25b9ffb2-b3db-11ed-afa1-0242ac120002",
            principal: "admin",
            hostname:  "25b9ffb2-b3db-11ed-afa1-0242ac120002",
            identifier: "",
            agentidentifier: "",
            hostidentifier: "",
            lastseenat: "",
            tags: [{
                id: "Web",
                name: "Web",
                kind: "service",
                sessions: []
                },
                {
                    id: "Team 3",
                    name: "Team 3",
                    kind: "group",
                    sessions: []
                }],
            tasks: [],
        }
    ]

    const formatOptionLabel = ({ value , label, customAbbreviation }: {value: string, label: string, customAbbreviation: Array<string>}) => (
        <Stack direction={"row"} align={"start"} >
            <StackItem>
                <Heading size="xs">{label}</Heading>
            </StackItem>
            <Divider/>
            <StackItem>
                <Stack gap={1} direction="row" shouldWrapChildren>
                    {customAbbreviation.map((x) => {
                        return <StackItem><Text fontSize={"xs"}>{x}</Text></StackItem>
                    })}
                </Stack>
            </StackItem>
        </Stack>
      );

    return (
        <ListItem>
            <StepIcon step={step} currStep={currStep}/>
            Select targets of tome
            {currStep === step &&
                <Box px={8} pt={4}>
                    <Stack gap={4}>
                    <StackItem>
                    <Tabs size='md' variant='enclosed' colorScheme="purple">
                        <TabList>
                            <Tab>Targets to select</Tab>
                            <Tab>Targets selected (0)</Tab>
                        </TabList>
                        <TabPanels>
                            <TabFilterTargets sessions={sessions}/>
                            <TabPanel>
                            <p>two!</p>
                            </TabPanel>
                        </TabPanels>
                    </Tabs>

                    </StackItem>
                    <StackItem>
                    <Stack direction={"row"} gap={1}>
                    <StackItem>
                        <Button onClick={handleBack}>Back</Button>
                    </StackItem>
                    <StackItem>
                        <Button variant="solid" colorScheme={"purple"} onClick={handleNext}>Next</Button>
                    </StackItem>
                    </Stack>
                    </StackItem>
                </Stack>
                </Box>
            }
      </ListItem> 
    );
}