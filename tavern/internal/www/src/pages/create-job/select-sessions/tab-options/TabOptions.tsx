import { Heading, Text, Stack, StackItem, TabPanel, Box, Button } from "@chakra-ui/react";
import { TrashIcon, PlusIcon } from "@heroicons/react/24/outline";
import React, { FC, useCallback, useState } from "react";
import Select from "react-select"
import { SessionType, Tome, TomeTag } from "../../../../utils/consts";
import {
    AutoSizer as _AutoSizer,
    List as _List,
    ListProps,
    AutoSizerProps,
    Grid as _Grid,
    GridProps
  } from "react-virtualized";

import 'react-virtualized/styles.css';
import TabOption from "./TabOption";

const Grid = _Grid as unknown as FC<GridProps>;  
const AutoSizer = _AutoSizer as unknown as FC<AutoSizerProps>;

type Props = {
    sessions: Array<SessionType>;
    groups: Array<TomeTag>;
    services: Array<TomeTag>;
    filteredSessions: any;
    setFilteredSessions: any;
    selectedSessions: any;
    setSelectedSessions: any;
}
export const TabOptions = (props: Props) => {
    const CARD_WIDTH = 800;
    const CARD_HEIGHT = 100;
    const COLUMN_COUNT = 1;

    const {sessions, groups, services, filteredSessions, setFilteredSessions, selectedSessions, setSelectedSessions} = props;
    const [filtersSelected, setFiltersSelected] = useState([]);

    const sessionsToDisplay = filtersSelected?.length > 0 ? filteredSessions : sessions;

    const getFormattedOptions = (sessions: Array<SessionType>, groups: Array<TomeTag>, services: Array<TomeTag>) => {
        return [
            { 
                label: "Service", 
                options: services.map(function(service: TomeTag){
                    return {
                        ...service,
                        value: service?.id,
                        label: service?.name,
                        kind: service?.kind
                    }})
            },
            { 
                label: "Group", 
                options: groups.map(function(group: TomeTag){
                    return {
                        ...group,
                        value: group?.id,
                        label: group?.name,
                        kind: group?.kind
                    };
                })
            },
            { 
                label: "Session", 
                options: sessions.map(function(session: SessionType){
                    return {
                        ...session,
                        value: session?.id,
                        label: session?.name,
                        kind: "session"
                    };
                })
            },
        ];
    };

    const handleChange = (selected: any) => {
        setFiltersSelected(selected);
        if(selected.length < 1 ){
            setFilteredSessions(sessions);
        }
        else{
            const searchTypes = selected.reduce((accumulator:any, currentValue:any) => {
                if(currentValue.kind === "session"){
                    accumulator.session.push(currentValue.value);
                }
                else if(currentValue.kind === "service"){
                    accumulator.service.push(currentValue.value);
                }
                else if(currentValue.kind === "group"){
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

    const toggleCheck = useCallback( (inputName :any) => {
        setSelectedSessions((currentState: any) => {
            const newState = {...currentState};
            newState[inputName] = !currentState[inputName];
            return newState;
        });
    }, []);

    const handleCheckAllFiltered = useCallback( () => {
        setSelectedSessions((currentState: any) => {
            const newState = {...currentState};
            sessionsToDisplay.map((session :any) => {
                newState[session.id] = true;
            });
            return newState;
        });
    }, []);
    const handleUnCheckAllFiltered = useCallback( () => {
        setSelectedSessions((currentState: any) => {
            const newState = {...currentState};
            sessionsToDisplay.map((session :any) => {
                newState[session.id] = false;
            });
            return newState;
        });
    }, []);
    console.log(sessionsToDisplay);
      
    const cellRenderer = (props: any) => {
            const {columnIndex, key, rowIndex, style} = props;
            const index = rowIndex * COLUMN_COUNT + columnIndex;
            return (
                <div key={key} style={style}>
                    <TabOption index={index} style={{ width: CARD_WIDTH, height: CARD_HEIGHT }} sessionsToDisplay={sessionsToDisplay} toggleCheck={toggleCheck} sessionsSelected={selectedSessions}  />
                </div>
            );
    };

    return (
        <TabPanel className="flex flex-col gap-4">
            <Stack direction="column" gap="2">
                <StackItem>
                    <Heading size="sm" mb={2}> Use the dropdown to filter the list then select targets</Heading>
                    <Select
                        isSearchable={true}
                        isMulti
                        options={getFormattedOptions(sessions, groups, services)}
                        onChange={handleChange}
                    />  
                </StackItem>
                <StackItem>
                    <Stack direction="row" gap={4}>
                        <StackItem>
                            <Button leftIcon={<PlusIcon className="h-4 w-4"/>} size={"sm"} onClick={()=> handleCheckAllFiltered()}>Select all bellow</Button>
                        </StackItem>
                        <StackItem>
                            <Button leftIcon={<TrashIcon className=" h-4 w-4"/>} size={"sm"} onClick={()=> handleUnCheckAllFiltered()}>Deselect all bellow</Button>
                        </StackItem>
                    </Stack>
                </StackItem>
                <StackItem>
                    <Box p={2} className="md-scroll-container" borderRadius={"md"}>
                        <Stack direction="column" gap={2} width="full" height="full">
                            <StackItem height="96">
                                <AutoSizer disableHeight>
                                    {({width}) => (
                                        <Grid
                                            cellRenderer={cellRenderer}
                                            columnCount={COLUMN_COUNT}
                                            columnWidth={CARD_WIDTH}
                                            height={sessionsToDisplay.length * CARD_HEIGHT}
                                            rowCount={sessionsToDisplay.length}
                                            rowHeight={CARD_HEIGHT}
                                            width={width}
                                    />
                                    )}
                                </AutoSizer>
                            </StackItem>
                            {/* {sessionsToDisplay.map((session: any) => {
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
                                                <Checkbox colorScheme={"purple"} size="lg" isChecked={formik?.values?.sessions[session.id]} onChange={()=> toggleCheck(session.id)}>
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
                            })} */}
                            {sessionsToDisplay.length === 0 &&
                                <Text fontSize={"sm"} p={2}>Try adjusting filter. No results found.</Text>
                            }
                        </Stack>
                    </Box>
                </StackItem>
            </Stack>
        </TabPanel>
    );
}