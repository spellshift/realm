import { Badge, Box, Button, Divider, Heading, ListItem, Stack, StackItem, Tab, TabList, TabPanel, TabPanels, Tabs, Text, Textarea, Icon, Grid, GridItem, Flex, Container, Checkbox, Card, CardBody, Spacer } from "@chakra-ui/react";
import React, { useState } from "react";
import Select from "react-select";
import { StepIcon } from "../step-icon";
import {MdFilterList} from "react-icons/md";

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

    const options = [
        { 
        label: "Group 1", 
        options:[
            { value: "Abe", label: "Abe", customAbbreviation: ["ABC", "xyz"] },
            { value: "Abe", label: "Abe", customAbbreviation: ["ABC", "xyz"] },
            { value: "Abe", label: "Abe", customAbbreviation: ["ABC", "xyz"] },
            { value: "Abe", label: "Abe", customAbbreviation: ["ABC", "xyz"] },
            { value: "John", label: "John", customAbbreviation: ["JAF"] },
            { value: "Dustin", label: "Dustin", customAbbreviation: ["DAD"] }
        ]
        },
        { 
            label: "Group 2", 
            options:[
                { value: "Abe", label: "Abe", customAbbreviation: ["ABC", "xyz"] },
                { value: "Abe", label: "Abe", customAbbreviation: ["ABC", "xyz"] },
                { value: "Abe", label: "Abe", customAbbreviation: ["ABC", "xyz"] },
                { value: "Abe", label: "Abe", customAbbreviation: ["ABC", "xyz"] },
                { value: "John", label: "John", customAbbreviation: ["JAF"] },
                { value: "Dustin", label: "Dustin", customAbbreviation: ["DAD"] }
            ]
        }
    ];

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
                            <Tab>Hosts to select</Tab>
                            <Tab>Hosts selected (0)</Tab>
                        </TabList>
                        <TabPanels>
                            <TabPanel >
                                <Stack direction={"row"} gap="1">
                                    <StackItem>
                                        <Icon as={MdFilterList} />
                                    </StackItem>
                                    <StackItem>
                                        <Heading size="sm"> Filter by selecting service, group, or host:</Heading>
                                    </StackItem>
                                </Stack>
                                <Select 
                                    isMulti
                                    options={options}
                                />  
                                {/* <Grid templateColumns='repeat(12, 1fr)' gap={1}>
                                    <GridItem colSpan={6}>
                                        Team:
                                        <Select 
                                                isMulti
                                                options={options}
                                            />          
                                    </GridItem>
                                    <GridItem colSpan={6}>
                                        Service:
                                        <Select 
                                                isMulti
                                                options={options}
                                            />          
                                    </GridItem>
                                </Grid> */}
                                <Container mt={4} p={2} className="md-scroll-container" borderRadius={"md"}>
                                    <Stack direction="column" gap={1}>
                                        <StackItem>
                                            <Button>Select all options below</Button>
                                        </StackItem>
                                        <StackItem>
                                            <Card>
                                                <CardBody>
                                                    <Checkbox colorScheme={"purple"} size="lg">
                                                        <Stack ml={4} w={"xl"}>
                                                            <StackItem>
                                                                <Heading size={"sm"}>Host</Heading>
                                                            </StackItem>
                                                            <StackItem>
                                                                Team | Service
                                                            </StackItem>
                                                        </Stack>
                                                    </Checkbox>
                                                </CardBody>
                                            </Card>
                                        </StackItem>
                                        <StackItem>
                                            <Card>
                                                <CardBody>
                                                    <Checkbox colorScheme={"purple"} size="lg">
                                                        <Stack ml={4} w={"xl"}>
                                                            <StackItem>
                                                                <Heading size={"sm"}>Host</Heading>
                                                            </StackItem>
                                                            <StackItem>
                                                                Team | Service
                                                            </StackItem>
                                                        </Stack>
                                                    </Checkbox>
                                                </CardBody>
                                            </Card>
                                        </StackItem>
                                    </Stack>
                                </Container>
                            </TabPanel>
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