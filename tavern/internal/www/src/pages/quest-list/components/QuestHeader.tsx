import React from "react";
import { Button } from "@chakra-ui/react";
import { useNavigate } from "react-router-dom";

const QuestHeader = () => {
    const navigate = useNavigate();

    return (
        <div className="border-b border-gray-200 pb-5 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex-1 flex flex-col gap-2">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Quests</h3>
                <div className="max-w-2xl text-sm">
                    Quests enable multi-beacon managment by taking a list of beacons and executing a tome with customized parameters against them. A quest is made up of tasks assocaited with a single beacon.
                </div>
            </div>
            <div>
                <Button size={"sm"}
                    onClick={() => navigate("/createQuest")}
                >
                    Create new quest
                </Button>
            </div>
        </div>
    );
}
export default QuestHeader;
