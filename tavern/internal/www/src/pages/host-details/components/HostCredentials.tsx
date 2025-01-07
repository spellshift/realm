import { useParams } from "react-router-dom";

import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { useCredsTable } from "../hooks/useCredsTable";
import CredentialTable from "./CredentialTable";

const HostCredentials = () => {
    const { hostId } = useParams();
    const { loading, creds, error } = useCredsTable(parseInt(hostId!));

    return (
        <div className="flex flex-col gap-2 mt-4">
            <div className="flex flex-col justify-center items-center gap-6">
                {(loading) ? (
                    <EmptyState type={EmptyStateType.loading} label="Loading credentials..." />
                ) : error ? (
                    <EmptyState type={EmptyStateType.error} label="Error credentials..." />
                ) : (creds.length > 0) ? (
                    <div className="mt-2 flex flex-col gap-1 w-full">
                        <CredentialTable data={creds} />
                    </div>
                ) : (
                    <EmptyState type={EmptyStateType.noData} label="No credential data found" />
                )}
            </div>
        </div>
    );
}
export default HostCredentials;
