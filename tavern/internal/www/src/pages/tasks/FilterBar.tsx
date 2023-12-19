import { useQuery } from "@apollo/client";
import { Heading } from "@chakra-ui/react";
import { setgroups } from "process";
import React from "react"
import Select from 'react-select';

import { GET_SEARCH_FILTERS } from "../../utils/queries";
import FreeTextSearch from "./FreeTextSearch";

type Props = {
    setSearch: (arg: string) => void;
    setBeacons: (arg: Array<string>) => void;
    setGroups: (arg: Array<string>) => void;
    setServices: (arg: Array<string>) => void;
    setHosts: (arg: Array<string>) => void;
    setPlatforms: (arg: Array<string>) => void;
}
const FilterBar = (props: Props) => {
    const {setSearch, setBeacons, setGroups, setServices, setHosts, setPlatforms} = props;
    const PARAMS = {
        variables: { 
            groupTag: { kind: "group" },
            serviceTag: { kind: "service" },
        }
    }
    const platformEnum = [
        { value: 'Windows', label: 'Windows' },
        { value: 'Linux', label: 'Linux' },
        { value: 'MacOS', label: 'MacOS' },
        { value: 'BSD', label: 'BSD' },
        { value: 'Unknown', label: 'Unknown' }
    ];
    const {data, loading, error} = useQuery(GET_SEARCH_FILTERS, PARAMS);

    return (
        <div>
            {(!loading && !error && data) && (
            <div className="grid grid-cols-2 gap-2 p-4 bg-white rounded-lg shadow-lg mt-2">
                <FreeTextSearch setSearch={setSearch} />
                <Select
                    isMulti
                    name="beacons"
                    options={data?.beacons}
                    className="basic-multi-select"
                    classNamePrefix="select"
                    placeholder="Filter by beacons"
                    onChange={(newValue) => {
                        if(newValue?.length == 0){
                            setBeacons([]);
                        }
                        else{
                            setBeacons(newValue.map((value: any) => value?.label))
                        }
                    }}
                />
                <Select
                    isMulti
                    name="service"
                    options={data?.serviceTags}
                    className="basic-multi-select"
                    classNamePrefix="select"
                    placeholder="Filter by services"
                    onChange={(newValue) => {
                        if(newValue?.length == 0){
                            setServices([]);
                        }
                        else{
                            setServices(newValue.map((value: any) => value?.label))
                        }
                    }}
                />
                <Select
                    isMulti
                    name="group"
                    options={data?.groupTags}
                    className="basic-multi-select"
                    classNamePrefix="select"
                    placeholder="Filter by group"
                    onChange={(newValue) => {
                        if(newValue?.length == 0){
                            setGroups([]);
                        }
                        else{
                            setGroups(newValue.map((value: any) => value?.label))
                        }
                    }}
                />
                <Select
                    isMulti
                    name="hosts"
                    options={data?.hosts}
                    className="basic-multi-select"
                    classNamePrefix="select"
                    placeholder="Filter by hosts"
                    onChange={(newValue) => {
                        if(newValue?.length == 0){
                            setHosts([]);
                        }
                        else{
                            setHosts(newValue.map((value: any) => value?.label))
                        }
                    }}
                />
                <Select
                    isMulti
                    name="platform"
                    options={platformEnum}
                    className="basic-multi-select"
                    classNamePrefix="select"
                    placeholder="Filter by platform"
                    onChange={(newValue) => {
                        if(newValue?.length == 0){
                            setPlatforms([]);
                        }
                        else{
                            setPlatforms(newValue.map((value: any) => value?.label))
                        }
                    }}
                />
            </div>
            )}
        </div>
    );
};
export default FilterBar