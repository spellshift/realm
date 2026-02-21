import { AdjustmentsHorizontalIcon } from "@heroicons/react/24/outline";
import { calculateTotalFilterCount, FilterFieldType, useFilters } from "./FilterContext";
import { BeaconFilterBar } from "../../components/beacon-filter-bar";
import { ButtonDialogPopover } from "../../components/ButtonDialogPopover";
import FreeTextSearch from "../../components/tavern-base-ui/FreeTextSearch";
import Button from "../../components/tavern-base-ui/button/Button";
import { LockKeyhole, UnlockKeyhole } from "lucide-react";
import { TomeFilterBar } from "../../components/TomeFilterBar";
import { Steps } from "@chakra-ui/react";
import { Tooltip } from '@/components/ui/tooltip';
import { useLocation } from "react-router-dom";
import UserFilterBar from "../../components/UserFilterBar";

function getFilterFields(pathname: string): FilterFieldType[] | null {
    if (pathname.startsWith('/hosts/')) {
        return [FilterFieldType.TOME_FIELDS, FilterFieldType.TOME_MULTI_SEARCH, FilterFieldType.QUEST_NAME, FilterFieldType.TASK_OUTPUT];
    }
    if (pathname === '/hosts') {
        return [FilterFieldType.BEACON_FIELDS];
    }
    if (pathname === '/quests' || pathname.startsWith('/quests/')) {
        return [FilterFieldType.BEACON_FIELDS, FilterFieldType.TOME_FIELDS, FilterFieldType.TOME_MULTI_SEARCH, FilterFieldType.QUEST_NAME, FilterFieldType.TASK_OUTPUT];
    }
    if (pathname === '/tasks' || pathname.startsWith('/tasks/')) {
        return [FilterFieldType.BEACON_FIELDS, FilterFieldType.TOME_FIELDS, FilterFieldType.TOME_MULTI_SEARCH, FilterFieldType.TASK_OUTPUT];
    }
    if (pathname === '/assets') {
        return [FilterFieldType.ASSET_NAME, FilterFieldType.USER];
    }

    return null;
}

export default function FilterControls() {
    const { pathname } = useLocation();
    const fieldsToRender = getFilterFields(pathname);

    const { filters, updateFilters, isLocked, setIsLocked } = useFilters();

    if (!fieldsToRender) return null;

    const getLabel = (): string => {
        const count = calculateTotalFilterCount(filters, fieldsToRender);
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
                        isDisabled={isLocked}
                    />
                </div>
            )
        }
        else if (field === FilterFieldType.QUEST_NAME) {
            return (
                <div key={field}>
                    <FreeTextSearch
                        key={field}
                        isDisabled={isLocked}
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
                        isDisabled={isLocked}
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
                        isDisabled={isLocked}
                    />
                </div>
            );
        }
        else if (field === FilterFieldType.TOME_MULTI_SEARCH) {
            return (
                <div key={field}>
                    <FreeTextSearch
                        key={field}
                        isDisabled={isLocked}
                        defaultValue={filters.tomeMultiSearch}
                        setSearch={(newValue) => updateFilters({ 'tomeMultiSearch': newValue })}
                        placeholder="Tome definition & values"
                    />
                </div>
            );
        }
        else if (field === FilterFieldType.ASSET_NAME) {
            return (
                <div key={field}>
                    <FreeTextSearch
                        key={field}
                        isDisabled={isLocked}
                        defaultValue={filters.assetName}
                        setSearch={(newValue) => updateFilters({ 'assetName': newValue })}
                        placeholder="Asset name"
                    />
                </div>
            );
        }
        else if (field === FilterFieldType.USER) {
            return (
                <div key={field}>
                    <UserFilterBar />
                </div>
            );
        }
        return null;
    }


    return (
        <ButtonDialogPopover label={getLabel()} leftIcon={<AdjustmentsHorizontalIcon className="w-4" />}>
            <div className="flex flex-col gap-1">
                <div className="flex flex-row justify-between pb-2 border-gray-100 border-b-2 items-center">
                    <h3 className="font-medium text-lg text-gray-700">Filters</h3>
                    <Tooltip
                        content={isLocked ? "Click to unlock filter state" : "Click to lock filter state"}
                        bg="white"
                        color="gray.600"
                        borderWidth="1px"
                        borderColor="gray.100"
                    >
                        <Button
                            buttonVariant="ghost"
                            buttonStyle={{ color: "purple", size: "md" }}
                            onClick={() => setIsLocked((prev) => !prev)}
                            leftIcon={isLocked ? <LockKeyhole className="w-5 h-5" /> : <UnlockKeyhole className="w-5 h-5" />}
                            aria-label={isLocked ? "Unlock filters" : "Lock filters"}
                            aria-pressed={isLocked}
                        />
                    </Tooltip>
                </div>
                {fieldsToRender.map((field) => {
                    return renderFilterComponent(field);
                })}
            </div>
        </ButtonDialogPopover>
    );
}
