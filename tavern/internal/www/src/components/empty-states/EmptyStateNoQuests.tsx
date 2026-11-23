import { EmptyState, EmptyStateType } from "../tavern-base-ui/EmptyState";
import { CreateQuestDropdown } from "../create-quest-modal/CreateQuestDropdown";

const EmptyStateNoQuests = () => {
    return (
        <EmptyState label="No quests found" type={EmptyStateType.noData} details="Get started by creating a new quest." >
            <CreateQuestDropdown />
        </EmptyState>
    )
}
export default EmptyStateNoQuests;
