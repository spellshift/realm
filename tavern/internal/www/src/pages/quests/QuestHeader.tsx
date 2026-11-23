import Breadcrumbs from "../../components/Breadcrumbs";
import { CreateQuestDropdown } from "../../components/create-quest-modal/CreateQuestDropdown";

const QuestHeader = () => {
    return (
        <div className="flex flex-col gap-4 justify-between">
            <div className="flex flex-row justify-between w-full items-center">
                <Breadcrumbs pages={[{
                    label: "Quests",
                    link: "/quests"
                }]} />
                <div>
                    <CreateQuestDropdown />
                </div>
            </div>
        </div>
    );
}
export default QuestHeader;
