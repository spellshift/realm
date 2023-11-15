import React from "react";
import { Ring } from '@uiball/loaders'
import { AdjustmentsHorizontalIcon, ExclamationTriangleIcon, NoSymbolIcon } from "@heroicons/react/24/outline";

export enum EmptyStateType{
    error= "Error",
    noData="No Data",
    noMatches="No Matches",
    loading="Loading",
    noIcon="No Icon"
}

type Props ={
    label: string,
    type: EmptyStateType,
    details?: string,
    children?: React.ReactNode
}
export const EmptyState = (props: Props) => {
    const {
        label, 
        details, 
        type=EmptyStateType.noData,
        children
    } = props;
    
    function getEmptyStateIcon(type: EmptyStateType){
        switch(type){
            case EmptyStateType.error:
                return <ExclamationTriangleIcon width={24} />
            case EmptyStateType.noMatches:
                return <AdjustmentsHorizontalIcon width={24} />
            case EmptyStateType.loading:
                return (<Ring 
                size={24}
                lineWeight={2}
                speed={2} 
                color="black" 
                />);
            case EmptyStateType.noData:
                return <NoSymbolIcon width={24} />
            case EmptyStateType.noIcon:
            default:
                return null;
        }
    }

    return (
        <div className="flex flex-col w-full h-full items-center justify-center p-8 gap-2">
            {getEmptyStateIcon(type)}
            <h2 className="text-base font-semibold leading-6 text-gray-900 max-w-lg text-center">{label}</h2>
            {details && <p className="text-sm text-gray-700 max-w-lg text-center">{details}</p>}
            {children && (<div className="py-2">{children}</div>)}
        </div>
    )
}