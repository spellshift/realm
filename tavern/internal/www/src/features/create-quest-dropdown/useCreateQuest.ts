import { useNavigate } from "react-router-dom";

import { Task, Tome } from "../../utils/consts";
import { checkIfBeaconOffline, constructTomeParams } from "../../utils/utils";

export const useCreateQuest = () => {
    const nav = useNavigate();

    const formatBeaconList = (tasks: Array<Task>) => {
        const beaconList = [] as Array<string>;
        const uniqueBeacons = {} as {[key:string]: boolean};

        for(const taskIndex in tasks){
            const beaconName = tasks[taskIndex].beacon.id;

            if( !( beaconName in uniqueBeacons) ){
                const beaconOffline = checkIfBeaconOffline(tasks[taskIndex].beacon);
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

    const handleCreateQuestWithNewTome = (name: string, tasks: Array<Task>) => {
        const beacons = formatBeaconList(tasks);

        nav("/createQuest", {
            state: {
              step: 1,
              beacons: beacons,
              name: name
            }
          });
    };


    const handleCreateQuestWithSameTome = (name: string, originalParms: string, tome: Tome, tasks: Array<Task>) => {
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
