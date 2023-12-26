import React from "react";
import { useParams } from "react-router-dom";
import { EditablePageHeader } from "./EditablePageHeader";

export const TaskPageHeader = () => {
    const { questId } = useParams();

    if(questId){
        return <EditablePageHeader />;
    }

    return  (
        <h3 className="text-xl font-semibold leading-6 text-gray-900">
            Quest outputs
        </h3>
    )
}