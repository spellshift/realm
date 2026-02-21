import { Steps, Heading, Input, InputGroup, Icon } from "@chakra-ui/react";
import React, { useEffect, useRef } from "react";
import { debounce } from "lodash"
import { useParams } from "react-router-dom";
import { LuSearch } from 'react-icons/lu';

type Props = {
    placeholder?: string
    setSearch: (args: string) => void;
}
const FreeTextSearch = (props: Props) => {
    const { placeholder, questId } = useParams();
    const { placeholder: parentPlaceholder, setSearch } = props;
    const placeholderText = parentPlaceholder ? parentPlaceholder : placeholder ? placeholder : questId ? "Search by output" : "Search by tome name, quest name, or output";

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
            <InputGroup className=" border-gray-300" startElement={<Icon color='gray.300' as={LuSearch as any} />} startElementProps={{ pointerEvents: 'none' }}>
                <Input type='text' placeholder={placeholderText} onChange={handleChange} />
            </InputGroup>
        </div>
    );
}
export default FreeTextSearch;
