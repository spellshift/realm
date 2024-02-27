import React from "react";
import { useParams } from "react-router-dom";
import { EditablePageHeader } from "./EditablePageHeader";

export const TaskPageHeader = () => {
    const { questId } = useParams();

    if (questId) {
        return <EditablePageHeader />;
    }

    return (
        <div className="flex-1 flex flex-col gap-2">
            <h3 className="text-xl font-semibold leading-6 text-gray-900">Tasks</h3>
            <div className="max-w-2xl text-sm">
                A task is a single instance of a tome plus its parameters executed against a single beacon.
            </div>
        </div>
    )
}
