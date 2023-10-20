import React from "react";
import { AdjustmentsHorizontalIcon, ExclamationTriangleIcon, NoSymbolIcon } from "@heroicons/react/24/outline";

export enum EmptyStateType{
    error= "Error",
    noData="No Data",
    noMatches="No Matches"
}

type Props ={
    label: string,
    details?: string,
    type: EmptyStateType
}
export const EmptyState = (props: Props) => {
    const {
        label, 
        details, 
        type=EmptyStateType.noData
    } = props;
    
    function getEmptyStateIcon(type: EmptyStateType){
        switch(type){
            case EmptyStateType.error:
                return <ExclamationTriangleIcon width={24} />
            case EmptyStateType.noMatches:
                return <AdjustmentsHorizontalIcon width={24} />
            default:
            case EmptyStateType.noData:
                return <NoSymbolIcon width={24} />
        }
    }

    return (
        <div className="flex flex-col w-full h-full items-center justify-center p-8">
            {getEmptyStateIcon(type)}
            <h2 className="mt-2 text-base font-semibold leading-6 text-gray-900 max-w-lg text-center">{label}</h2>
            {details && <p className="mt-1 text-sm text-gray-500 max-w-lg text-center">{details}</p>}
        </div>
    )
}