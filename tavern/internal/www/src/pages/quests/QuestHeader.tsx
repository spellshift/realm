import Breadcrumbs from "../../components/Breadcrumbs";
import Button from "../../components/tavern-base-ui/button/Button";
import { useCreateQuestModal } from "../../context/CreateQuestModalContext";

const QuestHeader = () => {
    const { openModal } = useCreateQuestModal();

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
                        onClick={() => openModal()}
                    >
                        Create quest
                    </Button>
                </div>
            </div>
        </div>
    );
}
export default QuestHeader;
