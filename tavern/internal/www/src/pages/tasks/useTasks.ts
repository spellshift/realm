import { useQuery } from "@apollo/client";
import { useCallback, useEffect, useState } from "react";
import { GET_TASK_QUERY } from "../../utils/queries";

export const useTasks = () => {
    // store filters
    const [search, setSearch] = useState("");
    const [groups, setGroups] = useState<Array<string>>([]);
    const [services, setServices] = useState<Array<string>>([]);
    const [beacons, setBeacons] = useState<Array<string>>([]);
    const [hosts, setHosts] = useState<Array<string>>([]);
    const [platforms, setPlatforms] = useState<Array<string>>([]);

    // get tasks
    const { loading, error, data, refetch } = useQuery(GET_TASK_QUERY,  {});

    const updateTaskList = useCallback(() => {
        let fq = {
            "where": {
                "and": [
                    {
                        "or": [
                          {"outputContains": search},
                          {"hasQuestWith": {
                              "nameContains": search
                            }
                          },
                          {"hasQuestWith": 
                            {"hasTomeWith": {"nameContains": search}}}
                        ]
                    },
                  ]
            }
        } as any;

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

    },[search, beacons, groups, services, hosts, platforms]);


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
}