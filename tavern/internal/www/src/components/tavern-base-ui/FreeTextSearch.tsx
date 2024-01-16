import { SearchIcon } from "@chakra-ui/icons";
import { Heading, Input, InputGroup, InputLeftElement } from "@chakra-ui/react";
import React, { useEffect, useRef } from "react";
import { debounce } from "lodash"

type Props = {
    placeholder: string;
    setSearch: (args: string) => void;
}
const FreeTextSearch = (props: Props) => {
    const { placeholder, setSearch } = props;

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
        <div className="flex-1 gap-1">
            <Heading size="sm" mb={2}> {placeholder}</Heading>
            <InputGroup className=" border-gray-300">
                <InputLeftElement pointerEvents='none'>
                    <SearchIcon color='gray.300' />
                </InputLeftElement>
                <Input type='text' placeholder={placeholder} onChange={handleChange} />
            </InputGroup>
        </div>
    );
}
export default FreeTextSearch;
