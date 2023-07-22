import { Heading, Text, Stack, StackItem, TabPanel, Icon, Box, Button, Card, CardBody, Checkbox, Flex } from "@chakra-ui/react";
import Select from "react-select"
import { MdInfoOutline } from "react-icons/md";
import { SelectOption, SelectOptionGroup } from "../../../utils/consts";
import { useState } from "react";
import { SearchFilter } from "./search-filter";

type TabFilterTargetsParams = {
    sessions: Array<any>;
    targets: any;
    setFieldValue: (arg1: any, arg2: any) => void;
}
export const TabFilterTargets = (props: TabFilterTargetsParams) => {
    const {sessions, targets, setFieldValue} = props;

    const [filteredSessions, setFilteredSessions] = useState(sessions);

    const toggleCheck = (inputName :any) => {
        const newState = {...targets};
        newState[inputName] = !targets[inputName];
        setFieldValue('sessions', newState);
    };

    const handleCheckAllFiltered = () => {
        const newState = { ...targets };
        filteredSessions.map((session) => {
            newState[session.id] = true;
        });
        setFieldValue('sessions', newState);
    }

    return (
        <TabPanel >
            <Stack direction="column" gap="2">
                <StackItem>
                    <Heading size="sm" mb={2}> Use the dropdown to filter the list then select targets</Heading>
                    <SearchFilter sessions={sessions} setFilteredSessions={setFilteredSessions} />
                </StackItem>
                <StackItem>
                    <Box p={2} className="md-scroll-container" borderRadius={"md"}>
                        <Stack direction="column" gap={2}>
                            <StackItem>
                                <Button size={"sm"} onClick={()=> handleCheckAllFiltered()}>Select all options below</Button>
                            </StackItem>
                            {filteredSessions.map((session: any) => {
                                // TODO change to map to avoid extra loop
                                let group = (session?.tags).find( (obj : any) => {
                                    return obj?.kind === "group"
                                });
                                let service = (session?.tags).find( (obj : any) => {
                                    return obj?.kind === "service"
                                });

                                return (
                                    <StackItem key={session?.id}>
                                        <Card>
                                            <CardBody>
                                                <Checkbox colorScheme={"purple"} size="lg" isChecked={targets[session.id]} onChange={()=> toggleCheck(session.id)}>
                                                    <Stack ml={4} w={"xl"}>
                                                        <StackItem>
                                                                <Text fontSize={"md"}>{session.hostname}</Text> 
                                                        </StackItem>
                                                        <StackItem>
                                                            <Flex direction="row" wrap={"wrap"}>
                                                                <Text fontSize={"sm"}>
                                                                    {group?.name} | {service?.name} | {session.principal}
                                                                </Text>
                                                            </Flex>
                                                        </StackItem>
                                                    </Stack>
                                                </Checkbox>
                                            </CardBody>
                                        </Card>
                                    </StackItem>
                                );
                            })}
                            {filteredSessions.length === 0 &&
                                <Text fontSize={"sm"} p={2}>Try adjusting filter. No results found.</Text>
                            }
                        </Stack>
                    </Box>
                </StackItem>
            </Stack>
        </TabPanel>
    );
}