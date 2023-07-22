import React from "react";
import {Heading} from "@chakra-ui/react";
import Select,  { createFilter } from "react-select"
import { SessionType, TomeTag } from "../../utils/consts";

type Props = {
    setFiltersSelected: (arg1: any) => void;
    sessions: Array<SessionType>;
    groups: Array<TomeTag>;
    services: Array<TomeTag>;
}
export const SessionFilterBar = (props: Props) => {
    const {setFiltersSelected, sessions, groups, services} = props;

    const getFormattedOptions = (sessions: Array<SessionType>, groups: Array<TomeTag>, services: Array<TomeTag>) => {
        return [
            { 
                label: "Service", 
                options: services.map(function(service: TomeTag){
                    return {
                        ...service,
                        value: service?.id,
                        label: service?.name,
                        kind: service?.kind
                    }})
            },
            { 
                label: "Group", 
                options: groups.map(function(group: TomeTag){
                    return {
                        ...group,
                        value: group?.id,
                        label: group?.name,
                        kind: group?.kind
                    };
                })
            },
            { 
                label: "Session", 
                options: sessions.map(function(session: SessionType){
                    return {
                        ...session,
                        value: session?.id,
                        label: session?.name,
                        kind: "session"
                    };
                })
            },
        ];
    };

    return (
        <div>
            <Heading size="sm" mb={2}> Filter by service, group, and session</Heading>
            <Select
                isSearchable={true}
                isMulti
                options={getFormattedOptions(sessions, groups, services)}
                onChange={setFiltersSelected}
                filterOption={createFilter({
                    matchFrom: 'any',
                    stringify: option => `${option.label}`,
                  })}
            />  
        </div>
    );
}