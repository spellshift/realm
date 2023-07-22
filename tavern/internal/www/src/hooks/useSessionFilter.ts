import { useCallback, useEffect, useState } from "react"
import { SessionType } from "../utils/consts";

export const useSessionFilter = (sessions: Array<SessionType>, selectedSessions: any) => {

    const [filteredSessions, setFilteredSessions] = useState(sessions);

    const [typeFilters, setTypeFilters] = useState([]);

    const [viewOnlySelected, setViewOnlySelected] = useState(false);

    function getSearchTypes(typeFilters: any){
        return typeFilters.reduce((accumulator:any, currentValue:any) => {
            if(currentValue.kind === "session"){
                accumulator.session.push(currentValue.value);
            }
            else if(currentValue.kind === "service"){
                accumulator.service.push(currentValue.value);
            }
            else if(currentValue.kind === "group"){
                accumulator.group.push(currentValue.value);
            }
            return accumulator;
        },
        {
            "session": [],
            "service": [],
            "group": []
        });
    };
    
    const filterByTypes = useCallback((filteredSessions: Array<SessionType>) => {
        if(typeFilters.length < 1){
            return filteredSessions;
        }

        const searchTypes = getSearchTypes(typeFilters);

        return filteredSessions.filter( (session) => {
            let group = (session?.tags).find( (obj : any) => {
                return obj?.kind === "group"
            }) || null;

            let service = (session?.tags).find( (obj : any) => {
                return obj?.kind === "service"
            }) || null;

            let match = true;

            if(searchTypes.session.length > 0){
                // If a session filter is applied ignore other filters to just match the session
                if(searchTypes.session.indexOf(session.id) > -1){
                    match = true;
                } 
                else{
                    return false;
                }  
            }

            if(searchTypes.service.length > 0){
                if(service && searchTypes.service.indexOf(service?.id) > -1){
                    match = true;
                } 
                else{
                    return false;
                }   
            }

            if(searchTypes.group.length > 0){
                if(group && searchTypes.group.indexOf(group?.id) > -1){
                    match = true;
                } 
                else{
                    return false;
                }   
            }

            return match;
        });
    },[typeFilters]);

    const filterBySelected = useCallback((sessions: Array<SessionType>, selectedSessions: any) => {
        if(viewOnlySelected){
            return sessions.filter((session: SessionType)=> selectedSessions[session?.id]);
        }
        else{
            return sessions;
        }
    },[viewOnlySelected]);

    useEffect(()=> {
       let filteredSessions = filterBySelected(sessions, selectedSessions);
       filteredSessions = filterByTypes(filteredSessions);
       setFilteredSessions(
        filteredSessions
       );
    },[sessions, selectedSessions, typeFilters, viewOnlySelected]);

    return {
        filteredSessions,
        setTypeFilters,
        viewOnlySelected,
        setViewOnlySelected
    }
}