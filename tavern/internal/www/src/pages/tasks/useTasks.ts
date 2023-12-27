import { useQuery } from "@apollo/client";
import { useCallback, useEffect, useState } from "react";
import { GET_TASK_QUERY } from "../../utils/queries";

export enum TASK_PAGE_TYPE{   
    questIdQuery= "ID_QUERY",
    questDetailsQuery= "QUEST_DETAILS_QUERY",
}

export const useTasks = (defaultQuery?: TASK_PAGE_TYPE, id?: string) => {
    // store filters
    const [search, setSearch] = useState("");
    const [groups, setGroups] = useState<Array<string>>([]);
    const [services, setServices] = useState<Array<string>>([]);
    const [beacons, setBeacons] = useState<Array<string>>([]);
    const [hosts, setHosts] = useState<Array<string>>([]);
    const [platforms, setPlatforms] = useState<Array<string>>([]);

    const constructDefaultQuery = useCallback((searchText?: string) => {
      const query = {
        "where": {
          "and": [] as Array<any>
        }
      };
        switch(defaultQuery){
            case TASK_PAGE_TYPE.questIdQuery:
                const include = [{"hasQuestWith": {"id": id}}] as Array<any>;

                if(searchText){include.push({"outputContains": searchText})};

                query.where.and = include;
                break;
            case TASK_PAGE_TYPE.questDetailsQuery:
            default:
                const text = searchText || "";  
                query.where.and = [{
                                "or": [
                                  {"outputContains": text},
                                  {"hasQuestWith": {
                                      "nameContains": text
                                    }
                                  },
                                  {"hasQuestWith": 
                                    {"hasTomeWith": {"nameContains": text}}}
                                ]
                }];
                break;
        }
        return query;
    },[defaultQuery, id]);

    // get tasks
    const { loading, error, data, refetch} = useQuery(GET_TASK_QUERY,  {variables: constructDefaultQuery(),  notifyOnNetworkStatusChange: true});

    const updateTaskList = useCallback(() => {
        let fq = constructDefaultQuery(search) as any;

        if(beacons.length > 0){
            fq.where.and = fq.where.and.concat(
                {
                "hasBeaconWith": {"nameIn": beacons}
                }
            );
        }

        if(groups.length > 0){
            fq.where.and = fq.where.and.concat(
                {
                    "hasBeaconWith": { "hasHostWith": {
                      "hasTagsWith": {
                        "and": [
                          {"kind": "group"},
                          {"nameIn": groups}
                        ]
                      }
                    }}
                }
            );
        }

        if(services.length > 0){
            fq.where.and = fq.where.and.concat(
                {
                    "hasBeaconWith": { "hasHostWith": {
                      "hasTagsWith": {
                        "and": [
                          {"kind": "service"},
                          {"nameIn": services}
                        ]
                      }
                    }}
                }
            );
        }

        if(hosts.length > 0){
            fq.where.and = fq.where.and.concat(
                {
                    "hasBeaconWith": {
                      "hasHostWith": {"nameIn": hosts}
                    }
                }
            );
        }

        if(platforms.length > 0){
            fq.where.and = fq.where.and.concat(
                {
                    "hasBeaconWith": {
                      "hasHostWith": {
                        "platformIn": platforms
                      }
                    }
                  }
            );
        }

        refetch(fq);

    },[search, beacons, groups, services, hosts, platforms, constructDefaultQuery, refetch]);


    useEffect(()=> {
        updateTaskList();
    },[updateTaskList]);



    return {
        data,
        loading,
        error,
        setSearch,
        setBeacons,
        setGroups,
        setServices,
        setHosts,
        setPlatforms
    }
};