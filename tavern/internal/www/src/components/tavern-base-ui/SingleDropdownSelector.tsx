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
    defaultValue?: Option;
    /** "sm" matches Chakra's `<Input size="sm" />` (32px tall, 14px text) for use beside text inputs. */
    size?: "sm" | "md";
}
const SingleDropdownSelector: FC<SingleDropdownSelectorProps> = ({
    label,
    options,
    setSelectedOption,
    isSearchable = true,
    defaultValue,
    size = "md",
}) => {
    const compact = size === "sm";

    const styles = {
        container: (base: any) => ({
            ...base,
            ...(compact && { fontSize: "14px" }),
        }),
        valueContainer: (base: any) => ({
            ...base,
            // 10px + the 2px singleValue margin lines the text up with a 12px-padded Input.
            ...(compact && { padding: "0 10px" }),
        }),
        control: (base: any) => ({
            ...base,
            ...(compact && { minHeight: "32px" }),
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
            ...(compact && { padding: "5px" }),
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
