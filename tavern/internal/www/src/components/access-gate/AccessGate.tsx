import React, { useContext } from "react";
import { AuthorizationContext } from "../../context/AuthorizationContext";
import { EmptyState, EmptyStateType } from "../tavern-base-ui/EmptyState";

type Props = {
    children: any;
}
export const AccessGate = (props: Props) => {
     const {children} = props;
     const {data, isLoading, error} = useContext(AuthorizationContext);

    if(isLoading){
        return (
            <div className="flex flex-row w-sceen h-screen justify-center items-center">
                <EmptyState label="Loading authroization state" type={EmptyStateType.loading}/>
            </div>
        );
    }

    if(error){
        return (
            <div className="flex flex-row w-sceen h-screen justify-center items-center">
                <EmptyState label="Error fetching authroization state" type={EmptyStateType.error} details="Please contact your admin to diagnose the issue."/>
            </div>
        );
    }

    if(data?.me?.isActivated){
        return children;
    }

    return (
        <div className="flex flex-row w-sceen h-screen justify-center items-center">
            <EmptyState label="Account not approved" details={`Gain approval by providing your id (${data?.me?.id}) to an admin.`} type={EmptyStateType.noData}/>
        </div>
    );
 }
