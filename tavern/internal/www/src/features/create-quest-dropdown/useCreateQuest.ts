import { useNavigate } from "react-router-dom";

import { BeaconType, Tome } from "../../utils/consts";
import { checkIfBeaconOffline, constructTomeParams } from "../../utils/utils";

export type LimitedTaskNode = {
    node: {
        beacon: BeaconType
    }
};

export const useCreateQuest = () => {
    const nav = useNavigate();

    const formatBeaconList = (tasks: {
        edges: Array<LimitedTaskNode>
    }) => {
        const beaconList = [] as Array<string>;
        const uniqueBeacons = {} as {[key:string]: boolean};

        for(const taskIndex in tasks?.edges){
            const beaconName = tasks?.edges[taskIndex]?.node?.beacon.id;

            if( !( beaconName in uniqueBeacons) ){
                const beaconOffline = checkIfBeaconOffline(tasks?.edges[taskIndex]?.node?.beacon);
                uniqueBeacons[beaconName] = !beaconOffline;
            }
        };

        for(const beacon in uniqueBeacons){
            if(uniqueBeacons[beacon]){
                beaconList.push(beacon);
            }
        }
        return beaconList;
    };

    const handleCreateQuestWithNewTome = (name: string, tasks: {
        edges: Array<LimitedTaskNode>
    }) => {
        const beacons = formatBeaconList(tasks);
        nav("/createQuest", {
            state: {
              step: 1,
              beacons: beacons,
              name: name
            }
          });
    };


    const handleCreateQuestWithSameTome = (name: string, originalParms: string, tome: Tome, tasks: {
        edges: Array<LimitedTaskNode>
    }) => {
        const beacons = formatBeaconList(tasks);
        const params = constructTomeParams(originalParms, tome?.paramDefs);

        nav("/createQuest", {
            state: {
              step: 2,
              beacons: beacons,
              tome: tome,
              params: params,
              name: name
            }
          });
    };

    return {
        handleCreateQuestWithNewTome,
        handleCreateQuestWithSameTome
    };
}
