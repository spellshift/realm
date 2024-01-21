import { Heading, Text, Stack, StackItem, Box, Button, FormLabel, Switch, Divider } from "@chakra-ui/react";
import { TrashIcon, PlusIcon } from "@heroicons/react/24/outline";
import React, { FC, useCallback } from "react";
import {
    AutoSizer as _AutoSizer,
    AutoSizerProps,
    Grid as _Grid,
    GridProps
} from "react-virtualized";

import 'react-virtualized/styles.css';
import { BeaconFilterBar } from "../../../components/beacon-filter-bar";
import BeaconOption from "../../../components/beacon-option/BeaconOption";
import { BeaconType, HostType, TomeTag } from "../../../utils/consts";
import { useBeaconFilter } from "../hooks/useBeaconFilter";

const Grid = _Grid as unknown as FC<GridProps>;
const AutoSizer = _AutoSizer as unknown as FC<AutoSizerProps>;

type Props = {
    beacons: Array<BeaconType>;
    groups: Array<TomeTag>;
    services: Array<TomeTag>;
    hosts: Array<HostType>;
    selectedBeacons: any;
    setSelectedBeacons: any;
}


const BeaconStep = (props: Props) => {
    const CARD_HEIGHT = 100;
    const COLUMN_COUNT = 1;

    const { beacons, groups, services, hosts, selectedBeacons, setSelectedBeacons } = props;

    const {
        filteredBeacons,
        setTypeFilters,
        setViewOnlySelected
    } = useBeaconFilter(beacons, selectedBeacons);

    const toggleCheck = useCallback((inputName: any) => {
        setSelectedBeacons((currentState: any) => {
            const newState = { ...currentState };
            newState[inputName] = !currentState[inputName];
            return newState;
        });
    }, []);

    const handleCheckAllFiltered = useCallback(() => {
        setSelectedBeacons((currentState: any) => {
            const newState = { ...currentState };
            filteredBeacons.map((beacon: any) => {
                newState[beacon.id] = true;
            });
            return newState;
        });
    }, [filteredBeacons]);

    const handleUnCheckAllFiltered = useCallback(() => {
        setSelectedBeacons((currentState: any) => {
            const newState = { ...currentState };
            filteredBeacons.map((beacon: any) => {
                newState[beacon.id] = false;
            });
            return newState;
        });
    }, [filteredBeacons]);

    const cellRenderer = (props: any, width: any) => {
        const { columnIndex, key, rowIndex, style } = props;
        const index = rowIndex * COLUMN_COUNT + columnIndex;
        return (
            <div key={key} style={style}>
                <BeaconOption index={index} style={{ width: width, height: CARD_HEIGHT }} beaconsToDisplay={filteredBeacons} toggleCheck={toggleCheck} beaconsSelected={selectedBeacons} />
            </div>
        );
    };


    function getSelectedCount() {
        let targetCount = 0;
        for (var key in selectedBeacons) {
            if (selectedBeacons[key] === true) {
                targetCount = targetCount + 1;
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
                            <BeaconFilterBar setFiltersSelected={setTypeFilters} groups={groups} services={services} beacons={beacons} hosts={hosts} />
                        </div>
                        <div className="flex flex-col gap-2">
                            <FormLabel htmlFor='isSelected'>
                                <Heading size="sm" >Filter by selected</Heading>
                            </FormLabel>
                            <Switch id='isSelected' className="pt-1" colorScheme="purple" onChange={() => setViewOnlySelected((value) => !value)} />
                        </div>
                    </div>
                </StackItem>
                <StackItem>
                    <Box p={2} className="option-container" borderRadius={"md"}>
                        <Stack direction="column" gap={2} width="full" height="full">
                            <StackItem>
                                <StackItem>
                                    <Button leftIcon={<PlusIcon className="h-4 w-4" />} size={"sm"} onClick={() => handleCheckAllFiltered()}>Select all ({filteredBeacons.length})</Button>
                                </StackItem>
                                <StackItem>
                                    <Button leftIcon={<TrashIcon className=" h-4 w-4" />} size={"sm"} onClick={() => handleUnCheckAllFiltered()}>Clear selected</Button>
                                </StackItem>
                            </StackItem>

                            {filteredBeacons.length === 0 && (
                                <StackItem>
                                    <Text fontSize={"sm"} p={2} textAlign="center">
                                        {filteredBeacons.length !== beacons.length && "Try adjusting filter. "}
                                        No results found.
                                    </Text>
                                </StackItem>
                            )}
                            <StackItem className="md-scroll-container" >
                                <AutoSizer disableHeight>
                                    {({ width }) => {
                                        return (
                                            <Grid
                                                cellRenderer={(props) => cellRenderer(props, width)}
                                                columnCount={COLUMN_COUNT}
                                                columnWidth={width}
                                                height={filteredBeacons.length * CARD_HEIGHT}
                                                rowCount={filteredBeacons.length}
                                                rowHeight={CARD_HEIGHT}
                                                width={width}
                                            />
                                        )
                                    }}
                                </AutoSizer>
                            </StackItem>
                        </Stack>
                    </Box>
                </StackItem>
                <StackItem className="flex flex-row items-end justify-end w-full">
                    <Heading size="sm" mb={2} className=" self-end text-right">Total beacons selected ({selectedCount})</Heading>
                </StackItem>
            </Stack>
        </div>
    )
}
export default BeaconStep;
