import { BarsArrowDownIcon, BarsArrowUpIcon } from "@heroicons/react/24/outline";
import { useSorts } from "./SortContext";
import { HostOrderField, OrderDirection, PageNavItem, QuestOrderField, TaskOrderField } from "../../utils/enums";
import { ButtonDialogPopover } from "../../components/ButtonDialogPopover";
import SingleDropdownSelector, { Option } from "../../components/tavern-base-ui/SingleDropdownSelector";
import { ReactElement } from "react";
import { useLocation } from "react-router-dom";
import { getNavItemFromPath, isHostDetailPath } from "../../utils/utils";

type SortPageType = PageNavItem.hosts | PageNavItem.quests | PageNavItem.tasks;

const sortablePages = new Set<PageNavItem>([PageNavItem.quests, PageNavItem.tasks, PageNavItem.hosts]);

function getSortPageType(pathname: string): SortPageType | null {
    // Host detail pages sort by tasks
    if (isHostDetailPath(pathname)) return PageNavItem.tasks;

    const navItem = getNavItemFromPath(pathname);
    return sortablePages.has(navItem) ? navItem as SortPageType : null;
}

const orderFieldOptionsMap = {
    [PageNavItem.hosts]: createOrderFieldOptions(HostOrderField),
    [PageNavItem.quests]: createOrderFieldOptions(QuestOrderField),
    [PageNavItem.tasks]: createOrderFieldOptions(TaskOrderField),
};

const directionOptions = [
    {
        label: "Ascending",
        value: OrderDirection.Asc
    },
    {
        label: "Descending",
        value: OrderDirection.Desc
    }
];


function formatEnumLabel(enumValue: string): string {
    return enumValue
        .split('_')
        .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
        .join(' ');
}

function getDirectionField(direction: OrderDirection): Option {
    if (direction === OrderDirection.Asc) {
        return directionOptions[0];
    };
    return directionOptions[1];
}

function getDirectionIcon(direction: OrderDirection): ReactElement {
    if (direction === OrderDirection.Asc) {
        return <BarsArrowUpIcon className="w-4" />

    };
    return <BarsArrowDownIcon className="w-4" />
}

function createOrderFieldOptions<T extends QuestOrderField | TaskOrderField | HostOrderField>(
    enumObj: Record<string, T>
): Array<Option> {
    return Object.values(enumObj).map((field) => ({
        label: formatEnumLabel(field),
        value: field,
    }));
}


export default function SortingControls() {
    const { pathname } = useLocation();
    const type = getSortPageType(pathname);
    const { sorts, updateSorts } = useSorts();

    if (!type) return null;

    const sortFieldsInUse = sorts[type];

    const activeFieldSortOption = {
        label: formatEnumLabel(sortFieldsInUse.field),
        value: sortFieldsInUse.field,
    };

    const activeDirectionSortOption = getDirectionField(sortFieldsInUse.direction);

    const orderFieldOptions = orderFieldOptionsMap[type];

    const leftIcon = getDirectionIcon(sortFieldsInUse.direction)

    return (
        <ButtonDialogPopover label={`Sort (${activeFieldSortOption.label})`} leftIcon={leftIcon}>
            <div className="flex flex-col gap-1">
                <div className="flex flex-row justify-between pb-2 border-gray-100 border-b-2">
                    <h3 className="font-medium text-lg text-gray-700">Sorting</h3>
                </div>
                <div className="grid grid-cols-2 gap-2">
                    <SingleDropdownSelector
                        label="Field"
                        options={orderFieldOptions}
                        setSelectedOption={(option: Option) => updateSorts({ [type]: { field: option.value, direction: activeDirectionSortOption.value } })}
                        isSearchable={false}
                        defaultValue={activeFieldSortOption}
                    />
                    <SingleDropdownSelector
                        label="Direction"
                        options={directionOptions}
                        setSelectedOption={(option: Option) => updateSorts({ [type]: { field: activeFieldSortOption.value, direction: option.value } })}
                        isSearchable={false}
                        defaultValue={activeDirectionSortOption}
                    />
                </div>
            </div>
        </ButtonDialogPopover>
    )
};
