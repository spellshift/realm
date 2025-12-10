import { FC } from "react";
import Select from "react-select"

export type Option = {
    label: string;
    value: string;
}
type SingleDropdownSelectorProps = {
    label: string;
    options: Array<Option>;
    setSelectedOption: any;
    isSearchable?: boolean;
    defaultValue?: Option
}
const SingleDropdownSelector: FC<SingleDropdownSelectorProps> = ({
    label,
    options,
    setSelectedOption,
    isSearchable = true,
    defaultValue
}) => {

    const styles = {
        control: (base: any) => ({
            ...base,
            border: "1px solid #cccccc",
            boxShadow: "none",
            "&:active": {
                borderColor: '#7429C6'
            },
            "&:select": {
                borderColor: '#7429C6'
            },
            "&:focus": {
                borderColor: '#7429C6'
            },
            "&:hover": {
                borderColor: '#7429C6',
                cursor: "pointer",
            }
        }),
        dropdownIndicator: (base: any) => ({
            ...base,
            color: "inherit",
        }),
        singleValue: (base: any) => ({
            ...base,
            color: "inherit"
        }),
        option: (base: any, state: any) => ({
            ...base,
            backgroundColor: state.isSelected ? '#7429C6' : 'inherit',
            "&:hover": {
                backgroundColor: "#D2D5DA",
                borderColor: "#D2D5DA",
                color: "#121826",
                cursor: "pointer"
            },
            "&:active": {
                backgroundColor: "#7429C6",
                borderColor: "#7429C6",
                color: "white"
            }
        })
    };

    return (
        <div className="min-w-[140px]">
            <Select
                isSearchable={isSearchable}
                styles={styles}
                defaultValue={defaultValue || options[0]}
                onChange={setSelectedOption}
                name={label}
                options={options}
            />
        </div>
    );
}
export default SingleDropdownSelector;
