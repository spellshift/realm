import Select, { createFilter, } from "react-select"
import { TomeSupportModel, TomeTactic, TomeFilterFieldKind } from "../utils/enums";
import { mapEnumToUIOptionField } from "../utils/utils";

type Props = {
    setFiltersSelected: (arg1: any) => void;
    filtersSelected?: any;
    initialFilters?: any;
    isDisabled?: boolean;
}

const TOME_FILTER_OPTIONS = [
    {
        label: "Support Model",
        options: mapEnumToUIOptionField(TomeSupportModel, TomeFilterFieldKind.SupportModel)
    },
    {
        label: "Tactic",
        options: mapEnumToUIOptionField(TomeTactic, TomeFilterFieldKind.Tactic)
    },
];

export const TomeFilterBar = ({ setFiltersSelected, filtersSelected, initialFilters, isDisabled }: Props) => {

    return (
        <div className="flex flex-col gap-1">
            <label className=" font-medium text-gray-700">Tome fields</label>
            <Select
                isDisabled={isDisabled}
                isSearchable={true}
                isMulti
                options={TOME_FILTER_OPTIONS}
                onChange={setFiltersSelected}
                filterOption={createFilter({
                    matchFrom: 'any',
                    stringify: option => `${option.label}`,
                })}
                value={filtersSelected || undefined}
                defaultValue={initialFilters || undefined}
            />
        </div>
    );
}
