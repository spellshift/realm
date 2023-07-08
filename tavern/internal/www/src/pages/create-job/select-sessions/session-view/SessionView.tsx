import { Heading, Text, Stack, StackItem, Box, Button, FormLabel, Switch, Divider } from "@chakra-ui/react";
import { TrashIcon, PlusIcon } from "@heroicons/react/24/outline";
import React, { ChangeEvent, FC, useCallback, useState } from "react";
import { SessionType, TomeTag } from "../../../../utils/consts";
import {
    AutoSizer as _AutoSizer,
    AutoSizerProps,
    Grid as _Grid,
    GridProps
  } from "react-virtualized";

import 'react-virtualized/styles.css';
import { SessionFilterBar } from "../../../../components/session-filter-bar";
import SessionOption from "../../../../components/session-option/SessionOption";
import { useSessionFilter } from "../../../../hooks/useSessionFilter";

const Grid = _Grid as unknown as FC<GridProps>;  
const AutoSizer = _AutoSizer as unknown as FC<AutoSizerProps>;

type Props = {
    sessions: Array<SessionType>;
    groups: Array<TomeTag>;
    services: Array<TomeTag>;
    selectedSessions: any;
    setSelectedSessions: any;
}

export const SessionView = (props: Props) => {
    const CARD_HEIGHT = 100;
    const COLUMN_COUNT = 1;

    const {sessions, groups, services, selectedSessions, setSelectedSessions} = props;

    const {
        filteredSessions,
        setTypeFilters,
        setViewOnlySelected
    } = useSessionFilter(sessions, selectedSessions);

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
            filteredSessions.map((session :any) => {
                newState[session.id] = true;
            });
            return newState;
        });
    }, [filteredSessions]);

    const handleUnCheckAllFiltered = useCallback( () => {
        setSelectedSessions((currentState: any) => {
            const newState = {...currentState};
            filteredSessions.map((session :any) => {
                newState[session.id] = false;
            });
            return newState;
        });
    }, [filteredSessions]);
      
    const cellRenderer = (props: any, width: any) => {
            const {columnIndex, key, rowIndex, style} = props;
            const index = rowIndex * COLUMN_COUNT + columnIndex;
            return (
                <div key={key} style={style}>
                    <SessionOption index={index} style={{ width: width, height: CARD_HEIGHT }} sessionsToDisplay={filteredSessions} toggleCheck={toggleCheck} sessionsSelected={selectedSessions}  />
                </div>
            );
    };


    function getSelectedCount(){
        let targetCount = 0;
        for (var key in selectedSessions) {
            if (selectedSessions[key] === true) {
                targetCount = targetCount +1;
            } 
        }
        return targetCount;
    }

    const selectedCount = getSelectedCount();

    return (
        <div className="flex flex-col gap-4">
            <Stack direction="column" gap="4">
                <StackItem>
                <div className="flex flex-row justify-between gap-8">
                    <div className=" flex-1">
                        <SessionFilterBar setFiltersSelected={setTypeFilters} groups={groups} services={services} sessions={sessions} />
                    </div>
                    <div className="flex flex-col gap-2">
                        <FormLabel htmlFor='isSelected'>
                            <Heading size="sm" >Filter by selected</Heading>
                        </FormLabel>
                        <Switch id='isSelected' className="pt-1" colorScheme="purple" onChange={() => setViewOnlySelected((value)=> !value)} />                 
                    </div>
                </div>
                </StackItem>
                <StackItem>
                    <Box p={2} className="option-container" borderRadius={"md"}>
                        <Stack direction="column" gap={2} width="full" height="full">
                            <StackItem>
                                    <StackItem>
                                        <Button leftIcon={<PlusIcon className="h-4 w-4"/>} size={"sm"} onClick={()=> handleCheckAllFiltered()}>Select all ({filteredSessions.length})</Button>
                                    </StackItem>
                                    <StackItem>
                                        <Button leftIcon={<TrashIcon className=" h-4 w-4"/>} size={"sm"} onClick={()=> handleUnCheckAllFiltered()}>Clear selected</Button>
                                    </StackItem>
                            </StackItem>
                            <StackItem className="md-scroll-container" >
                                <AutoSizer disableHeight>
                                    {({width}) => {
                                        return (
                                            <Grid
                                                    cellRenderer={(props) => cellRenderer(props, width)}
                                                    columnCount={COLUMN_COUNT}
                                                    columnWidth={width}
                                                    height={filteredSessions.length * CARD_HEIGHT}
                                                    rowCount={filteredSessions.length}
                                                    rowHeight={CARD_HEIGHT}
                                                    width={width}
                                            />
                                        )
                                    }}
                                </AutoSizer>
                            </StackItem>
                            {filteredSessions.length === 0 &&
                                <Text fontSize={"sm"} p={2}>Try adjusting filter. No results found.</Text>
                            }
                        </Stack>
                    </Box>
                </StackItem>
                <StackItem className="flex flex-row items-end justify-end w-full">
                    <Heading size="sm" mb={2} className=" self-end text-right">Total sessions selected ({selectedCount})</Heading>
                </StackItem>
            </Stack>
        </div>
    )
}