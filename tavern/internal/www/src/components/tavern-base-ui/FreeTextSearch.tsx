import { Input, Group, InputElement, Icon } from "@chakra-ui/react";
import React, { useEffect, useRef } from "react";
import { debounce } from "lodash"
import { LuSearch } from 'react-icons/lu';

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
        }, 400)
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
            <Group attached>
                <InputElement pointerEvents='none'>
                    <Icon color="gray.300" asChild>
                        {/* @ts-ignore */}
                        <LuSearch />
                    </Icon>
                </InputElement>
                <Input type='text' defaultValue={String(defaultValue)} placeholder={placeholder} onChange={handleChange} disabled={isDisabled} />
            </Group>
        </div>
    );
}
export default FreeTextSearch;
