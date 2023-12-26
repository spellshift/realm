import { SearchIcon } from "@chakra-ui/icons";
import { FormLabel, Heading, Input, InputGroup, InputLeftElement } from "@chakra-ui/react";
import React, { useEffect, useRef } from "react";
import { debounce } from "lodash"

type Props = {
    setSearch: (args: string) => void;
}
const FreeTextSearch = (props: Props) => {
    const {setSearch} = props;

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
            <InputGroup className=" border-gray-300">
                <InputLeftElement pointerEvents='none'>
                <SearchIcon color='gray.300' />
                </InputLeftElement>
                <Input type='text' placeholder='Search by tome name, quest name, or output' onChange={handleChange}/>
            </InputGroup>
        </div>
    );
}
export default FreeTextSearch;