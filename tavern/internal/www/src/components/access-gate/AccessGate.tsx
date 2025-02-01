import { EmptyState, EmptyStateType } from "../tavern-base-ui/EmptyState";
import { UserType } from "../../utils/consts";
import { AuthorizationContextType } from "../../context/AuthorizationContext";

type Props = {
    children: any;
    authData: AuthorizationContextType;
    adminOnly: boolean;
}
export const AccessGate = (props: Props) => {
    const {children, authData, adminOnly} = props;

    let userData: UserType = authData.me;

    if (!userData.isActivated) {
    	return (
        	<div className="flex flex-row w-sceen h-screen justify-center items-center">
            	<EmptyState label="Account not approved" details={`Gain approval by providing your id (${userData.id}) to an admin.`} type={EmptyStateType.noData}/>
        	</div>
    	);
    }

    if (adminOnly && !userData.isAdmin) {
        return (
            <div className="flex flex-row w-sceen h-screen justify-center items-center">
                <EmptyState label="Not Authorized" type={EmptyStateType.error} details="You are not authorized to view this page. Please contact your admin if you believe this is a mistake."/>
            </div>
        );
    }

    return children;
}
