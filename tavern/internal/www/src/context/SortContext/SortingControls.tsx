import { BarsArrowDownIcon, BarsArrowUpIcon } from "@heroicons/react/24/outline";
import { useSorts } from "./SortContext";
import { OrderDirection } from "../../utils/enums";
import { ButtonDialogPopover } from "../../components/ButtonDialogPopover";
import SingleDropdownSelector, { Option } from "../../components/tavern-base-ui/SingleDropdownSelector";
import { ReactElement } from "react";
import { formatEnumLabel, orderFieldOptionsMap, SortablePageNavItem } from "./sortingUtils";

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

type SortingControlsProps = {
    sortType: SortablePageNavItem;
};

export default function SortingControls({ sortType }: SortingControlsProps) {
    const { sorts, updateSorts } = useSorts();

    const sortFieldsInUse = sorts[sortType];

    const activeFieldSortOption = {
        label: formatEnumLabel(sortFieldsInUse.field),
        value: sortFieldsInUse.field,
    };

    const activeDirectionSortOption = getDirectionField(sortFieldsInUse.direction);

    const orderFieldOptions = orderFieldOptionsMap[sortType];

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
                        setSelectedOption={(option: Option) => updateSorts({ [sortType]: { field: option.value, direction: activeDirectionSortOption.value } })}
                        isSearchable={false}
                        defaultValue={activeFieldSortOption}
                    />
                    <SingleDropdownSelector
                        label="Direction"
                        options={directionOptions}
                        setSelectedOption={(option: Option) => updateSorts({ [sortType]: { field: activeFieldSortOption.value, direction: option.value } })}
                        isSearchable={false}
                        defaultValue={activeDirectionSortOption}
                    />
                </div>
            </div>
        </ButtonDialogPopover>
    )
};
