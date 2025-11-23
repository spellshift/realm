import { useContext } from "react";
import { useFilters } from "../../context/FilterContext";
import { TagContext } from "../../context/TagContext";
import { BeaconFilterBar } from "../beacon-filter-bar";
import FreeTextSearch from "../tavern-base-ui/FreeTextSearch";
import { FilterControlWrapper } from "./FilterControlWrapper";
import { Switch } from "@chakra-ui/react";

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
};

const filterConfig: Record<FilterPageType, FilterFieldType[]> = {
    [FilterPageType.QUEST]: [FilterFieldType.BEACON_FIELDS, FilterFieldType.QUEST_NAME, FilterFieldType.TASK_OUTPUT],
    [FilterPageType.TASK]: [FilterFieldType.BEACON_FIELDS, FilterFieldType.TASK_OUTPUT],
    [FilterPageType.HOST_TASK]: [FilterFieldType.TASK_OUTPUT],
    [FilterPageType.HOST]: [FilterFieldType.BEACON_FIELDS],
};

export default function FilterControls({ type }: { type: FilterPageType }) {
    const { filters, updateFilters } = useFilters();
    const { data } = useContext(TagContext);
    const fieldsToRender = filterConfig[type];

    const calculateFilterCount = (field: FilterFieldType): number => {
        if (field === FilterFieldType.QUEST_NAME && filters.questName !== "") {
            return 1;
        }
        if (field === FilterFieldType.TASK_OUTPUT && filters.taskOutput !== "") {
            return 1;
        }
        if (field === FilterFieldType.BEACON_FIELDS && filters.beaconFields.length > 0) {
            return filters.beaconFields.length;
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
                        beacons={data?.beacons || []}
                        groups={data?.groupTags || []}
                        services={data?.serviceTags || []}
                        hosts={data?.hosts || []}
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
                        placeholder="Search by quest"
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
                        placeholder="Search by output"
                    />
                </div>
            );
        }
        return null;
    }


    return (
        <FilterControlWrapper label={getLabel()}>
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
        </FilterControlWrapper>
    )
}
