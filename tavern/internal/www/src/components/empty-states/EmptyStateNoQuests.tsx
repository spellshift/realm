import { Link } from "react-router-dom";
import { EmptyState, EmptyStateType } from "../tavern-base-ui/EmptyState";
import Button from "../tavern-base-ui/button/Button";

const EmptyStateNoQuests = () => {
    return (
        <EmptyState label="No quests found" type={EmptyStateType.noData} details="Get started by creating a new quest." >
            <Link to="/createQuest">
                <Button
                    type="button"
                >
                    Create new quest
                </Button>
            </Link>
        </EmptyState>
    )
}
export default EmptyStateNoQuests;
