import React from "react";

import Badge from "./tavern-base-ui/badge/Badge";

type Props = {
    task: any;
}
const TaskStatusBadge = (props: Props) => {
    const { task } = props;

    if (task.error.length > 0) return <Badge badgeStyle={{ color: 'red' }} >Error</Badge>;
    else if (task.execFinishedAt) return <Badge badgeStyle={{ color: 'green' }} >Finished</Badge>;
    else if (task.execStartedAt) return <Badge badgeStyle={{ color: 'gray' }} >In progress</Badge>;
    else return <Badge badgeStyle={{ color: 'gray' }} >Queued</Badge>;
}
export default TaskStatusBadge;
