import React from "react";
import Breadcrumbs from "../../../components/Breadcrumbs";
import Button from "../../../components/tavern-base-ui/button/Button";
import { useNavigate } from "react-router-dom";

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
                        buttonStyle={{ color: "purple", size: "md" }}
                        onClick={() => navigate("/createQuest")}
                    >
                        Create new quest
                    </Button>
                </div>
            </div>
        </div>
    );
}
export default QuestHeader;
