import {  Box, Button, ListItem, Stack, StackItem, Tab, TabList, TabPanels, Tabs } from "@chakra-ui/react";
import React, { useState } from "react";
import { StepIcon } from "../step-icon";
import { TabFilterTargets } from "./tab-filter-targets";
import { TabSelectedTargets } from "./tab-selected-targets";

type StepSelectTargetsParams = {
    step: number;
    currStep: number;
    setCurrStep: any;
    targets: any;
    setFieldValue: (arg1: any, arg2: any) => void;
    handleSubmit: (arg1: any) => void;
}
export const StepSelectTargets = (props: StepSelectTargetsParams) => {
    const {step, currStep, setCurrStep, targets, setFieldValue, handleSubmit} = props;

    const handleBack = () => {
        setCurrStep(step -1);
    }

    const beacons = [
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
                beacons: []
                },
                {
                    id: "Team 1",
                    name: "Team 1",
                    kind: "group",
                    beacons: []
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
                beacons: []
                },
                {
                    id: "Team 1",
                    name: "Team 1",
                    kind: "group",
                    beacons: []
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
                beacons: []
                },
                {
                    id: "Team 1",
                    name: "Team 1",
                    kind: "group",
                    beacons: []
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
                beacons: []
                },
                {
                    id: "Team 3",
                    name: "Team 3",
                    kind: "group",
                    beacons: []
                }],
            tasks: [],
        }
    ];

    function getSelectedTargetCount(){
        let targetCount = 0;
        for (var key in targets) {
            if (targets[key] === true) {
                targetCount = targetCount +1;
            }
        }
        return targetCount;
    }
    const targetCount = getSelectedTargetCount();

    return (
        <ListItem>
            <StepIcon step={step} currStep={currStep}/>
            Select beacons to run the tome on
            {currStep === step &&
                <Box px={8} pt={4}>
                    <Stack gap={4}>
                    <StackItem>
                    <Tabs size='md' variant='enclosed' colorScheme="purple">
                        <TabList>
                            <Tab>Beacon options</Tab>
                            <Tab>Beacons selected ({targetCount})</Tab>
                        </TabList>
                        <TabPanels>
                            <TabFilterTargets beacons={beacons} targets={targets} setFieldValue={setFieldValue}/>
                            <TabSelectedTargets beacons={beacons} targets={targets} setFieldValue={setFieldValue} targetCount={targetCount} />
                        </TabPanels>
                    </Tabs>

                    </StackItem>
                    <StackItem>
                    <Stack direction={"row"} gap={1}>
                    <StackItem>
                        <Button onClick={handleBack}>Back</Button>
                    </StackItem>
                    <StackItem>
                        <Button variant="solid" colorScheme={"purple"} isDisabled={targetCount < 1 ? true : false} onClick={handleSubmit}>Submit job</Button>
                    </StackItem>
                    </Stack>
                    </StackItem>
                </Stack>
                </Box>
            }
      </ListItem>
    );
}