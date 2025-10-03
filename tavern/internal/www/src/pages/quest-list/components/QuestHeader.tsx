import React from "react";
import { useNavigate } from "react-router-dom";
import Breadcrumbs from "../../../components/Breadcrumbs";
import Button from "../../../components/tavern-base-ui/button/Button";
import PageHeader from "../../../components/tavern-base-ui/PageHeader";

const QuestHeader = () => {
    const navigate = useNavigate();

    return (
        <div className="flex flex-col gap-4 justify-between">
            <div className="flex flex-row justify-between w-full items-center">
                <Breadcrumbs pages={[{
                    label: "Quests",
                    link: "/quests"
                }]} />
                <div>
                    <Button
                        buttonStyle={{ color: "gray", size: "md" }}
                        onClick={() => navigate("/createQuest")}
                    >
                        Create new quest
                    </Button>
                </div>
            </div>
            <PageHeader title="Quests" description="Quests enable multi-beacon managment by taking a list of beacons and executing a tome with customized parameters against them. A quest is made up of tasks associated with a single beacon." />
        </div>
    );
}
export default QuestHeader;
