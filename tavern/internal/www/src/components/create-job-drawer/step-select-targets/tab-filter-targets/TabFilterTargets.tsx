import { Heading, Text, Stack, StackItem, TabPanel, Icon, Box, Button, Card, CardBody, Checkbox, Flex } from "@chakra-ui/react";
import Select from "react-select"
import { MdInfoOutline } from "react-icons/md";
import { SelectOption, SelectOptionGroup } from "../../../utils/consts";
import { useState } from "react";

type TabFilterTargetsParams = {
    sessions: Array<any>;
}
export const TabFilterTargets = (props: TabFilterTargetsParams) => {
    const {sessions} = props;

    const [filteredSessions, setFilteredSessions] = useState(sessions);

    const options = [
        { 
        label: "Service", 
        options:[
            { value: "Relay", label: "Relay", type: "service" },
            { value: "Admin", label: "Admin", type: "service" },
            { value: "Web", label: "Web", type: "service" },
        ]
        },
        { 
            label: "Group", 
            options:[
                { value: "Team 1", label: "Team 1", type: "group" },
                { value: "Team 2", label: "Team 2", type: "group" },
                { value: "Team 3", label: "Team 3", type: "group" },
                { value: "Team 4", label: "Team 4", type: "group" },
                { value: "Team 5", label: "Team 5", type: "group" },
                { value: "Team 6", label: "Team 6", type: "group"}
            ]
        },
        { 
            label: "Session", 
            options:[
                { value: "15b9ec70-b3db-11ed-afa1-0242ac120002", label: "15b9ec70-b3db-11ed-afa1-0242ac120002", type: "session"},
                { value: "15b9f04e-b3db-11ed-afa1-0242ac120002", label: "15b9f04e-b3db-11ed-afa1-0242ac120002", type: "session"},
                { value: "15b9f99a-b3db-11ed-afa1-0242ac120002", label: "15b9f99a-b3db-11ed-afa1-0242ac120002", type: "session"},
                { value: "15b9fd82-b3db-11ed-afa1-0242ac120002", label: "15b9fd82-b3db-11ed-afa1-0242ac120002", type: "session"},
                { value: "15b9ffb2-b3db-11ed-afa1-0242ac120002", label: "15b9ffb2-b3db-11ed-afa1-0242ac120002", type: "session"},
                { value: "25b9ffb2-b3db-11ed-afa1-0242ac120002", label: "25b9ffb2-b3db-11ed-afa1-0242ac120002", type: "session"},
                { value: "35b9ffb2-b3db-11ed-afa1-0242ac120002", label: "35b9ffb2-b3db-11ed-afa1-0242ac120002", type: "session"},
                { value: "45b9ffb2-b3db-11ed-afa1-0242ac120002", label: "45b9ffb2-b3db-11ed-afa1-0242ac120002", type: "session"},
                { value: "55b9ffb2-b3db-11ed-afa1-0242ac120002", label: "55b9ffb2-b3db-11ed-afa1-0242ac120002", type: "session"},
                { value: "65b9ffb2-b3db-11ed-afa1-0242ac120002", label: "65b9ffb2-b3db-11ed-afa1-0242ac120002", type: "session"},
                { value: "75b9ffb2-b3db-11ed-afa1-0242ac120002", label: "75b9ffb2-b3db-11ed-afa1-0242ac120002", type: "session"},
                { value: "85b9ffb2-b3db-11ed-afa1-0242ac120002", label: "85b9ffb2-b3db-11ed-afa1-0242ac120002", type: "session"},
                { value: "95b9ffb2-b3db-11ed-afa1-0242ac120002", label: "95b9ffb2-b3db-11ed-afa1-0242ac120002", type: "session"}

            ]
        }
    ];

    const handleChange = (selected: any) => {
        if(selected.length < 1 ){
            setFilteredSessions(sessions);
        }
        else{
            const searchTypes = selected.reduce((accumulator:any, currentValue:any) => {
                if(currentValue.type === "session"){
                    accumulator.session.push(currentValue.value);
                }
                else if(currentValue.type === "service"){
                    accumulator.service.push(currentValue.value);
                }
                else if(currentValue.type === "group"){
                    accumulator.group.push(currentValue.value);
                }
                return accumulator;
            },
            {
                "session": [],
                "service": [],
                "group": []
            });

            const filtered = sessions.filter( (session) => {
                console.log(session);
                let group = (session?.tags).find( (obj : any) => {
                    return obj?.kind === "group"
                }) || null;

                let service = (session?.tags).find( (obj : any) => {
                    return obj?.kind === "service"
                }) || null;

                let match = true;

                if(searchTypes.session.length > 0){
                    // If a session filter is applied ignore other filters to just match the session
                    if(searchTypes.session.indexOf(session.id) > -1){
                        match = true;
                    } 
                    else{
                        return false;
                    }  
                }

                if(searchTypes.service.length > 0){
                    if(service && searchTypes.service.indexOf(service?.id) > -1){
                        match = true;
                    } 
                    else{
                        return false;
                    }   
                }

                if(searchTypes.group.length > 0){
                    if(group && searchTypes.group.indexOf(group?.id) > -1){
                        match = true;
                    } 
                    else{
                        return false;
                    }   
                }

                return match;
            });
            setFilteredSessions(filtered);
        }
    };

    return (
        <TabPanel >
            <Stack direction="column" gap="2">
                <StackItem>
                    <Heading size="sm" mb={2}> Use the dropdown to filter the list then select targets</Heading>
                    <Select
                        isSearchable={true}
                        isMulti
                        options={options}
                        onChange={handleChange}
                    />  
                </StackItem>
                <StackItem>
                    <Box p={2} className="md-scroll-container" borderRadius={"md"}>
                        <Stack direction="column" gap={2}>
                            <StackItem>
                                <Button size={"sm"}>Select all options below</Button>
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
                                                <Checkbox colorScheme={"purple"} size="lg">
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