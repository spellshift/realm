import React, { useState } from 'react';

import CreatableSelect from 'react-select/creatable';

export type CreatableDropdownOption = {
    label: string;
    value: string;
}

export type CreatableDropdownType = {
    defaultValue: CreatableDropdownOption,
    defaultOptions: Array<CreatableDropdownOption>,
    handleCreateOption: (inputValue: string) => void
    handleSelectOption: (inputValue: CreatableDropdownOption) => void
}

const createOption = (label: string) => ({
    label,
    value: label.toLowerCase().replace(/\W/g, ''),
});

const defaultOptions = [
    createOption('One'),
    createOption('Two'),
    createOption('Three'),
];

export default function CreatableTagDropdown() {
    const [isLoading, setIsLoading] = useState(false);
    const [options, setOptions] = useState(defaultOptions);
    const [value, setValue] = useState<CreatableDropdownOption | null>(null);

    const handleCreate = (inputValue: string) => {
        setIsLoading(true);
        setTimeout(() => {
            // const newOption = createOption(inputValue);
            // setIsLoading(false);
            // setOptions((prev) => [...prev, newOption]);
            // setValue(newOption);
        }, 1000);
    };

    return (
        <CreatableSelect
            isClearable
            isDisabled={isLoading}
            isLoading={isLoading}
            onChange={(newValue) => setValue(newValue)}
            onCreateOption={handleCreate}
            options={options}
            value={value}
        />
    );
};
