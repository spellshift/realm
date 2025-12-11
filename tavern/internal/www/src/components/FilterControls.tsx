import { AdjustmentsHorizontalIcon } from "@heroicons/react/24/outline";
import { useFilters } from "../context/FilterContext";
import { BeaconFilterBar } from "./beacon-filter-bar";
import { ButtonDialogPopover } from "./ButtonDialogPopover";
import FreeTextSearch from "./tavern-base-ui/FreeTextSearch";
import { Switch } from "@chakra-ui/react";
import { TomeFilterBar } from "./TomeFilterBar";

export enum FilterPageType {
    QUEST = 'Quest',
    HOST = 'Host',
    TASK = 'Task',
    HOST_TASK = 'HOST_TASK',
};

export enum FilterFieldType {
    BEACON_FIELDS = 'beaconFields',
    TASK_OUTPUT = 'taskOutput',
    QUEST_NAME = 'questName',
    TOME_FIELDS = 'tomeFields',
    TOME_MULTI_SEARCH = "tomeMultiSearch"
};

const filterConfig: Record<FilterPageType, FilterFieldType[]> = {
    [FilterPageType.QUEST]: [FilterFieldType.BEACON_FIELDS, FilterFieldType.TOME_FIELDS, FilterFieldType.TOME_MULTI_SEARCH, FilterFieldType.QUEST_NAME, FilterFieldType.TASK_OUTPUT],
    [FilterPageType.TASK]: [FilterFieldType.BEACON_FIELDS, FilterFieldType.TASK_OUTPUT],
    [FilterPageType.HOST_TASK]: [FilterFieldType.TOME_FIELDS, FilterFieldType.TOME_MULTI_SEARCH, FilterFieldType.QUEST_NAME, FilterFieldType.TASK_OUTPUT],
    [FilterPageType.HOST]: [FilterFieldType.BEACON_FIELDS],
};

export default function FilterControls({ type }: { type: FilterPageType }) {
    const { filters, updateFilters } = useFilters();
    const fieldsToRender = filterConfig[type];

    const calculateFilterCount = (field: FilterFieldType): number => {
        if (field === FilterFieldType.QUEST_NAME && filters.questName !== "") {
            return 1;
        }
        else if (field === FilterFieldType.TASK_OUTPUT && filters.taskOutput !== "") {
            return 1;
        }
        else if (field === FilterFieldType.TOME_MULTI_SEARCH && filters.tomeMultiSearch !== "") {
            return 1;
        }
        else if (field === FilterFieldType.BEACON_FIELDS && filters.beaconFields.length > 0) {
            return filters.beaconFields.length;
        }
        else if (field === FilterFieldType.TOME_FIELDS && filters.tomeFields.length > 0) {
            return filters.tomeFields.length;
        }
        return 0;
    }

    const getLabel = (): string => {
        let count = 0;

        if (!filters.filtersEnabled) {
            return 'Filter (disabled)'
        }
        fieldsToRender.map((field) => count += calculateFilterCount(field));

        return `Filters (${count})`;
    };

    const renderFilterComponent = (field: FilterFieldType) => {
        if (field === FilterFieldType.BEACON_FIELDS) {
            return (
                <div key={field}>
                    <BeaconFilterBar
                        key={field}
                        setFiltersSelected={(newValue) => updateFilters({ 'beaconFields': newValue })}
                        filtersSelected={filters.beaconFields}
                        isDisabled={!filters.filtersEnabled}
                    />
                </div>
            )
        }
        else if (field === FilterFieldType.QUEST_NAME) {
            return (
                <div key={field}>
                    <FreeTextSearch
                        key={field}
                        isDisabled={!filters.filtersEnabled}
                        defaultValue={filters.questName}
                        setSearch={(newValue) => updateFilters({ 'questName': newValue })}
                        placeholder="Quest name"
                    />
                </div>
            )
        }
        else if (field === FilterFieldType.TASK_OUTPUT) {
            return (
                <div key={field}>
                    <FreeTextSearch
                        key={field}
                        isDisabled={!filters.filtersEnabled}
                        defaultValue={filters.taskOutput}
                        setSearch={(newValue) => updateFilters({ 'taskOutput': newValue })}
                        placeholder="Task output"
                    />
                </div>
            );
        }
        else if (field === FilterFieldType.TOME_FIELDS) {
            return (
                <div key={field}>
                    <TomeFilterBar
                        key={field}
                        setFiltersSelected={(newValue) => updateFilters({ 'tomeFields': newValue })}
                        filtersSelected={filters.tomeFields}
                        isDisabled={!filters.filtersEnabled}
                    />
                </div>
            );
        }
        else if (field === FilterFieldType.TOME_MULTI_SEARCH) {
            return (
                <div key={field}>
                    <FreeTextSearch
                        key={field}
                        isDisabled={!filters.filtersEnabled}
                        defaultValue={filters.tomeMultiSearch}
                        setSearch={(newValue) => updateFilters({ 'tomeMultiSearch': newValue })}
                        placeholder="Tome definition & values"
                    />
                </div>
            );
        }
        return null;
    }


    return (
        <ButtonDialogPopover label={getLabel()} leftIcon={<AdjustmentsHorizontalIcon className="w-4" />}>
            <div className="flex flex-col gap-1">
                <div className="flex flex-row justify-between pb-2 border-gray-100 border-b-2">
                    <h3 className="font-medium text-lg text-gray-700">Filters</h3>
                    <Switch
                        id='isSelected'
                        className="pt-1"
                        colorScheme="purple"
                        isChecked={filters.filtersEnabled}
                        onChange={() => updateFilters({ 'filtersEnabled': !filters.filtersEnabled })}
                    />
                </div>
                {fieldsToRender.map((field) => {
                    return renderFilterComponent(field);
                })}
            </div>
        </ButtonDialogPopover>
    )
}
