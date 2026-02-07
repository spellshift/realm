import { FC, useCallback } from "react";
import { VirtualizedCardItem } from "../../components/tavern-base-ui/virtualized-card-list";
import TaskCard from "../../components/task-card/TaskCard";
import { GET_TASK_QUERY } from "../../utils/queries";
import { TaskNode, TaskQueryTopLevel } from "../../utils/interfacesQuery";

interface TaskCardVirtualizedProps {
    itemId: string;
    isVisible: boolean;
}

const TaskCardSkeleton: FC = () => {
    return (
        <div className="rounded-lg shadow border-gray-200 border-2 animate-pulse">
            <div className="flex flex-row gap-4 items-center p-4 bg-gray-100 h-16">
            </div>
            <div className="h-60">
            </div>
        </div>
    );
};

export const TaskCardVirtualized: FC<TaskCardVirtualizedProps> = ({
    itemId,
    isVisible,
}) => {
    const getVariables = useCallback((id: string) => ({
        where: { id },
        first: 1,
    }), []);

    const extractData = useCallback((response: TaskQueryTopLevel): TaskNode | null => {
        return response?.tasks?.edges?.[0]?.node ?? null;
    }, []);

    const renderCard = useCallback((task: TaskNode) => {
        return <TaskCard task={task} />;
    }, []);

    const renderSkeleton = useCallback(() => {
        return <TaskCardSkeleton />;
    }, []);

    return (
        <VirtualizedCardItem<TaskNode>
            itemId={itemId}
            query={GET_TASK_QUERY}
            getVariables={getVariables}
            renderCard={renderCard}
            renderSkeleton={renderSkeleton}
            extractData={extractData}
            isVisible={isVisible}
        />
    );
};

export default TaskCardVirtualized;
