import { useParams } from "react-router-dom";

import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { useCredsTable } from "../useCredsTable";
import CredentialTable from "./CredentialTable";
import FreeTextSearch from "../../../components/tavern-base-ui/FreeTextSearch";
import { FormLabel, Heading, Switch } from "@chakra-ui/react";
import CredentialTableGrouped from "./CredentialTableGrouped";

const CredentialTab = () => {
    const { hostId } = useParams();
    const { loading, creds, error, setSearch, groupByPrincipal, setGroupByPrincipal } = useCredsTable(parseInt(hostId!));

    const searchPlaceholder = "Search by Principal";

    return (
        <div className="flex flex-col gap-2 mt-4">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div className="col-span-1 md:col-span-2">
                    <FreeTextSearch placeholder={searchPlaceholder} setSearch={setSearch} />
                </div>
                <div className="flex flex-row-reverse md:flex-row gap-1 justify-center">
                    <FormLabel htmlFor='groupByPrincipal' className="mt-1">
                        <Heading size="sm" >Group by Principal/Kind</Heading>
                    </FormLabel>
                    <Switch id='groupByPrincipal' className="pt-1" colorScheme="purple" onChange={() => setGroupByPrincipal((value: boolean) => !value)} />
                </div>
            </div>
            <div className="flex flex-col justify-center items-center gap-6">
                {(loading) ? (
                    <EmptyState type={EmptyStateType.loading} label="Loading credentials..." />
                ) : error ? (
                    <EmptyState type={EmptyStateType.error} label="Error credentials..." />
                ) : (creds.length > 0) ? (
                    <div className="mt-2 flex flex-col gap-1 w-full">
                        {groupByPrincipal ? <CredentialTableGrouped data={creds} /> : <CredentialTable data={creds} />}
                    </div>
                ) : (
                    <EmptyState type={EmptyStateType.noData} label="No credential data found" />
                )}
            </div>
        </div>
    );
}
export default CredentialTab;
