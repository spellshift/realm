import { Box, Button, Card, CardBody, Flex, Icon, IconButton, Stack, StackItem, TabPanel, Text } from "@chakra-ui/react";
import React from "react";

import { MdDelete } from "react-icons/md";

type TabSelectedTargetsParams = {
    targets: any;
    sessions: Array<any>;
    setFieldValue: (arg1: any, arg2: any) => void;
    targetCount: number;
}
export const TabSelectedTargets = (props: TabSelectedTargetsParams) => {
    const {targets, sessions, setFieldValue, targetCount} = props;

    const toggleCheck = (inputName :any) => {
        const newState = { ...targets };
        newState[inputName] = !targets[inputName];
        setFieldValue('sessions', newState);
    };
    
    const handleDeleteAll = () => {
        let newState = {...targets};
        for(let session in targets){
            newState[session] = false;
        }
        setFieldValue('sessions', newState);
    }

    return (
        <TabPanel>
            <Box p={2} className="md-scroll-container" borderRadius={"md"}>
                <Stack direction="column" gap={2}>
                    {targetCount > 0 && (
                        <StackItem>
                            <Button size={"sm"} onClick={()=> handleDeleteAll()}>Delete all options below</Button>
                        </StackItem>
                    )}
                    {sessions.map((session: any, index: number) => {
                        if(targets[session.id]){
                            let group = (session?.tags).find( (obj : any) => {
                                return obj?.kind === "group"
                            });
                            let service = (session?.tags).find( (obj : any) => {
                                return obj?.kind === "service"
                            });
                            return (
                                <StackItem key={session?.id} className="max-width">
                                    <Card className="max-width">
                                        <CardBody>
                                            <Stack direction="row" align="center">
                                                <StackItem>
                                                    <IconButton colorScheme="red" aria-label="delete-session" icon={<Icon as={MdDelete}/>} onClick={()=> toggleCheck(session.id)}/>
                                                </StackItem>
                                                <StackItem>
                                                    <Stack ml={4} direction={"column"}>
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
                                                </StackItem>
                                            </Stack>
                                        </CardBody>
                                    </Card>
                                </StackItem>
                            );
                        }
                    })}
                    {targetCount < 1 &&
                        <Text fontSize={"sm"} p={2}>No sessions selected</Text>
                    }
                </Stack>
            </Box>
        </TabPanel>
    );
}