import { Link } from "react-router-dom";
import { EmptyState, EmptyStateType } from "../tavern-base-ui/EmptyState";

const EmptyStateNoQuests = () => {
    return (
        <EmptyState label="No quests found" type={EmptyStateType.noData} details="Get started by creating a new quest." >
            <Link to="/createQuest">
                <button
                    type="button"
                    className="inline-flex items-center rounded-md bg-purple-700 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-purple-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-700"
                >
                    Create new quest
                </button>
            </Link>
        </EmptyState>
    )
}
export default EmptyStateNoQuests;
