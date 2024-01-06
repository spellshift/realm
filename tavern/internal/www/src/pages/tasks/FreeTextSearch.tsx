import { SearchIcon } from "@chakra-ui/icons";
import { Heading, Input, InputGroup, InputLeftElement } from "@chakra-ui/react";
import React, { useEffect, useRef } from "react";
import { debounce } from "lodash"
import { useParams } from "react-router-dom";

type Props = {
    setSearch: (args: string) => void;
}
const FreeTextSearch = (props: Props) => {
    const { questId } = useParams();
    const placeholderText = questId ? "Search by output" : "Search by tome name, quest name, or output";
    const { setSearch } = props;

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
            <Heading size="sm" mb={2}> {placeholderText}</Heading>
            <InputGroup className=" border-gray-300">
                <InputLeftElement pointerEvents='none'>
                    <SearchIcon color='gray.300' />
                </InputLeftElement>
                <Input type='text' placeholder={placeholderText} onChange={handleChange} />
            </InputGroup>
        </div>
    );
}
export default FreeTextSearch;