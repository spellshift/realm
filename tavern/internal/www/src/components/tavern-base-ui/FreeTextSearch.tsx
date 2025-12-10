import { SearchIcon } from "@chakra-ui/icons";
import { Input, InputGroup, InputLeftElement } from "@chakra-ui/react";
import React, { useEffect, useRef } from "react";
import { debounce } from "lodash"

type Props = {
    placeholder: string;
    defaultValue?: string;
    setSearch: (args: string) => void;
    isDisabled?: boolean;
}
const FreeTextSearch = (props: Props) => {
    const { placeholder, defaultValue, setSearch, isDisabled } = props;

    const debouncedSearch = useRef(
        debounce(async (criteria) => {
            setSearch(criteria);
        }, 300)
    ).current;

    async function handleChange(e: React.ChangeEvent<HTMLInputElement>) {
        debouncedSearch(e.target.value);
    }

    useEffect(() => {
        return () => {
            debouncedSearch.cancel();
        };
    }, [debouncedSearch]);

    return (
        <div className="flex flex-col gap-1">
            <label className="text-gray-700"> {placeholder}</label>
            <InputGroup className=" border-gray-300">
                <InputLeftElement pointerEvents='none'>
                    <SearchIcon color='gray.300' />
                </InputLeftElement>
                <Input type='text' defaultValue={defaultValue} placeholder={placeholder} onChange={handleChange} disabled={isDisabled} />
            </InputGroup>
        </div>
    );
}
export default FreeTextSearch;
