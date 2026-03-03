import { useCallback, useState } from "react";
import {
    Heading,
    FormLabel,
    Switch,
    Tooltip,
} from "@chakra-ui/react";

import { BeaconFilterBar } from "../../beacon-filter-bar";
import Button from "../../tavern-base-ui/button/Button";
import { EmptyState, EmptyStateType } from "../../tavern-base-ui/EmptyState";
import { FilterBarOption } from "../../../utils/interfacesUI";
import { useOnlineBeaconIds } from "./useOnlineBeaconIds";
import { BeaconTable } from "./BeaconTable";
import { BeaconSelectionStepProps } from "./types";
import { MinusIcon, PlusIcon } from "lucide-react";

export const BeaconSelectionStep = ({ setCurrStep, formik, setOpen }: BeaconSelectionStepProps) => {
    const [typeFilters, setTypeFilters] = useState<FilterBarOption[]>([]);
    const [viewOnlySelected, setViewOnlySelected] = useState(false);
    const [viewOnePerHost, setViewOnePerHost] = useState(false);

    const selectedBeaconIds = formik.values.beacons;

    const {
        beaconIds,
        initialLoading,
    } = useOnlineBeaconIds({
        typeFilters,
        selectedBeaconIds,
        viewOnlySelected,
        viewOnePerHost,
    });

    const selectedCount = selectedBeaconIds.length;
    const hasBeaconSelected = selectedCount > 0;

    const toggleBeacon = useCallback(
        (beaconId: string) => {
            const currentBeacons = formik.values.beacons;
            const isSelected = currentBeacons.includes(beaconId);

            if (isSelected) {
                formik.setFieldValue(
                    "beacons",
                    currentBeacons.filter((id) => id !== beaconId)
                );
            } else {
                formik.setFieldValue("beacons", [...currentBeacons, beaconId]);
            }
        },
        [formik]
    );

    const handleSelectAllVisible = useCallback(() => {
        const currentBeacons = new Set(formik.values.beacons);
        beaconIds.forEach((id) => {
            currentBeacons.add(id);
        });
        formik.setFieldValue("beacons", Array.from(currentBeacons));
    }, [beaconIds, formik]);

    const handleDeselectAllVisible = useCallback(() => {
        const visibleIds = new Set(beaconIds);
        const remaining = formik.values.beacons.filter((id) => !visibleIds.has(id));
        formik.setFieldValue("beacons", remaining);
    }, [beaconIds, formik]);

    if (initialLoading) {
        return (
            <div className="flex flex-col gap-6">
                <div className="flex flex-col gap-1">
                    <h2 className="text-xl font-semibold text-gray-900">Select agent beacons</h2>
                    <p className="text-sm text-gray-700 italic">
                        Only active beacons are available for selection
                    </p>
                </div>
                <EmptyState type={EmptyStateType.loading} label="Loading beacons..." />
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-1">
                <h2 className="text-xl font-semibold text-gray-900">Select agent beacons</h2>
                <p className="text-sm text-gray-700 italic">
                    Only active beacons are available for selection
                </p>
            </div>

            <div className="flex flex-col gap-4">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div className="col-span-1 md:col-span-2">
                        <BeaconFilterBar
                            value={typeFilters}
                            onChange={setTypeFilters}
                            hideStatusFilter={true}
                        />
                    </div>
                    <div className="flex-1 flex md:flex-col gap-2 flex-row">
                        <div className="flex flex-row-reverse md:flex-row gap-1 justify-end">
                            <FormLabel htmlFor="isSelected" className="mt-1">
                                <Heading size="sm">Only selected</Heading>
                            </FormLabel>
                            <Switch
                                id="isSelected"
                                className="pt-1"
                                colorScheme="purple"
                                isChecked={viewOnlySelected}
                                onChange={() => setViewOnlySelected((value) => !value)}
                            />
                        </div>
                        <Tooltip
                            label="Show one beacon per host, prioritizing admin privileges and reliable transports"
                            placement="bottom"
                            bg="white"
                            color="gray.600"
                            borderWidth="1px"
                            borderColor="gray.100"
                        >
                            <div className="flex flex-row-reverse md:flex-row gap-1 justify-end">
                                <FormLabel htmlFor="onePerHost" className="mt-1">
                                    <Heading size="sm">One per host</Heading>
                                </FormLabel>
                                <Switch
                                    id="onePerHost"
                                    className="pt-1"
                                    colorScheme="purple"
                                    isChecked={viewOnePerHost}
                                    onChange={() => setViewOnePerHost((value) => !value)}
                                />
                            </div>
                        </Tooltip>
                    </div>
                </div>

                <div className="p-2 option-container flex flex-col gap-2">    
                    <div className="flex flex-row gap-2">
                        <Button
                            buttonStyle={{ color: "gray", size: "md" }}
                            leftIcon={<PlusIcon className="h-4 w-4" />}
                            onClick={handleSelectAllVisible}
                        >
                            Select visible ({beaconIds.length})
                        </Button>
                        <Button
                            buttonStyle={{ color: "gray", size: "md" }}
                            leftIcon={<MinusIcon className="h-4 w-4" />}
                            onClick={handleDeselectAllVisible}
                        >
                            Deselect visible
                        </Button>
                    </div>
                    <BeaconTable
                        beaconIds={beaconIds}
                        selectable={true}
                        selectedBeaconIds={selectedBeaconIds}
                        onToggle={toggleBeacon}
                        emptyMessage="No online beacons found. Try adjusting filters."
                    />
                </div>
                <div className="flex flex-row items-center justify-end w-full">
                    <Heading size="sm" className="text-right">
                        {selectedCount} beacon{selectedCount !== 1 ? "s" : ""} selected
                    </Heading>
                </div>
            </div>

            <div className="flex flex-row gap-2">
                <Button
                    buttonStyle={{ color: "gray", size: "md" }}
                    onClick={()=> setOpen(false)}
                    aria-label="close modal"
                >
                    Cancel
                </Button>
                <Button
                    buttonStyle={{ color: "purple", size: "md" }}
                    onClick={()=> setCurrStep(1)}
                    disabled={!hasBeaconSelected}
                    aria-label="continue beacon step"
                >
                    Continue
                </Button>
            </div>
        </div>
    );
};

export default BeaconSelectionStep;
