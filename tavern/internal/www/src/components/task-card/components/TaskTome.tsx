import { FC } from "react";
import CodeBlock from "../../tavern-base-ui/CodeBlock";
import { TomeNode } from "../../../utils/interfacesQuery";
import { toDisplayString } from "../../../utils/utils";

interface TaskTomeProps {
    tome: TomeNode;
}

const TaskTome: FC<TaskTomeProps> = ({ tome }) => {
    const displayValue = toDisplayString(tome.eldritch);

    if (!displayValue) return <div>Tome code not found</div>;

    return (
        <div className="text-sm max-h-80 overflow-y-scroll overflow-x-scroll py-1" aria-label="task output">
            <CodeBlock
                code={displayValue}
                language="python"
            />
        </div>
    );
};

export default TaskTome;
