import { Heading, Text, Stack, StackItem, Box, FormLabel, Switch } from "@chakra-ui/react";
import { TrashIcon, PlusIcon } from "@heroicons/react/24/outline";
import { FC, useCallback } from "react";
import {
    AutoSizer as _AutoSizer,
    AutoSizerProps,
    Grid as _Grid,
    GridProps
} from "react-virtualized";

import { BeaconFilterBar } from "../../../components/beacon-filter-bar";
import BeaconOption from "../../../components/beacon-option/BeaconOption";
import { useBeaconFilter } from "../hooks/useBeaconFilter";
import Button from "../../../components/tavern-base-ui/button/Button";
import { BeaconNode, HostNode, TagNode } from "../../../utils/interfacesQuery";

const Grid = _Grid as unknown as FC<GridProps>;
const AutoSizer = _AutoSizer as unknown as FC<AutoSizerProps>;

type Props = {
    beacons: Array<BeaconNode>;
    groups: Array<TagNode>;
    services: Array<TagNode>;
    hosts: Array<HostNode>;
    selectedBeacons: any;
    setSelectedBeacons: any;
}


const BeaconStep = (props: Props) => {
    const CARD_HEIGHT = 100;
    const COLUMN_COUNT = 1;
    const { beacons, selectedBeacons, setSelectedBeacons } = props;

    const {
        filteredBeacons,
        setTypeFilters,
        setViewOnlySelected,
        setViewOnlyOnePerHost,
        typeFilters
    } = useBeaconFilter(beacons, selectedBeacons);

    const toggleCheck = useCallback((inputName: any) => {
        setSelectedBeacons((currentState: any) => {
            const newState = { ...currentState };
            newState[inputName] = !currentState[inputName];
            return newState;
        });
    }, [setSelectedBeacons]);

    const handleCheckAllFiltered = useCallback(() => {
        setSelectedBeacons((currentState: any) => {
            const newState = { ...currentState };
            filteredBeacons.forEach((beacon: BeaconNode) => {
                newState[beacon.id] = true;
            });
            return newState;
        });
    }, [filteredBeacons, setSelectedBeacons]);

    const handleUnCheckAllFiltered = useCallback(() => {
        setSelectedBeacons((currentState: any) => {
            const newState = { ...currentState };
            filteredBeacons.forEach((beacon: BeaconNode) => {
                newState[beacon.id] = false;
            });
            return newState;
        });
    }, [filteredBeacons, setSelectedBeacons]);

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
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div className="col-span-1 md:col-span-2">
                            <BeaconFilterBar filtersSelected={typeFilters} setFiltersSelected={setTypeFilters} hideStatusFilter={true} />
                        </div>
                        <div className="flex-1 flex flex-col gap-2">
                            <div className="flex flex-row-reverse md:flex-row gap-1 justify-end">
                                <FormLabel htmlFor='isSelected' className="mt-1">
                                    <Heading size="sm" >View only selected beacons</Heading>
                                </FormLabel>
                                <Switch id='isSelected' className="pt-1" colorScheme="purple" onChange={() => setViewOnlySelected((value) => !value)} />
                            </div>
                            <div className="flex flex-row-reverse md:flex-row gap-1 justify-end">
                                <FormLabel htmlFor='isOnePerHost' className="mt-1">
                                    <Heading size="sm" >View one beacon per host</Heading>
                                </FormLabel>
                                <Switch id='isOnePerHost' className="pt-1" colorScheme="purple" onChange={() => setViewOnlyOnePerHost((value) => !value)} />
                            </div>
                        </div>
                    </div>
                </StackItem>
                <StackItem>
                    <Box p={2} className="option-container" borderRadius={"md"}>
                        <Stack direction="column" gap={2} width="full" height="full">
                            <StackItem>
                                <Stack direction="row" gap={2} width="full" height="full">
                                    <StackItem>
                                        <Button
                                            buttonStyle={{ color: "gray", size: "md" }}
                                            leftIcon={<PlusIcon className="h-4 w-4" />}
                                            onClick={() => handleCheckAllFiltered()}>Select all ({filteredBeacons.length})
                                        </Button>
                                    </StackItem>
                                    <StackItem>
                                        <Button
                                            buttonStyle={{ color: "gray", size: "md" }}
                                            leftIcon={<TrashIcon className=" h-4 w-4" />}
                                            onClick={() => handleUnCheckAllFiltered()}>Clear selected</Button>
                                    </StackItem>
                                </Stack>
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
