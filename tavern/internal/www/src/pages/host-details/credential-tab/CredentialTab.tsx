import { useParams } from "react-router-dom";
import { FormLabel, Switch } from "@chakra-ui/react";

import Button from "../../../components/tavern-base-ui/button/Button";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import FreeTextSearch from "../../../components/tavern-base-ui/FreeTextSearch";
import { SortingControls } from "../../../context/SortContext";
import { PageNavItem } from "../../../utils/enums";
import { useCredsTable } from "./useCredsTable";
import CredentialTable from "./CredentialTable";
import CredentialTableGrouped from "./CredentialTableGrouped";

const CredentialTab = () => {
    const { hostId } = useParams();
    const parsedHostId = hostId ? parseInt(hostId, 10) : NaN;
    const {
        loading,
        creds,
        totalCount,
        error,
        search,
        setSearch,
        groupByPrincipal,
        setGroupByPrincipal,
    } = useCredsTable(parsedHostId);

    const searchPlaceholder = "Search by Principal";

    const renderTableContent = () => {
        if (error || !hostId || Number.isNaN(parsedHostId)) {
            return (
                <EmptyState
                    type={EmptyStateType.error}
                    label="Error loading data"
                    details={error?.message || "Host ID not found"}
                />
            );
        }

        if (loading || totalCount === undefined) {
            return (
                <EmptyState
                    type={EmptyStateType.loading}
                    label="Loading data..."
                />
            );
        }

        if (totalCount === 0 && search.trim() !== "") {
            return (
                <EmptyState
                    type={EmptyStateType.noMatches}
                    label="No data matching your filters"
                >
                    <Button
                        onClick={() => setSearch("")}
                        buttonVariant="solid"
                        buttonStyle={{ color: "purple", size: "md" }}
                    >
                        Clear filters
                    </Button>
                </EmptyState>
            );
        }

        if (totalCount === 0) {
            return (
                <EmptyState
                    type={EmptyStateType.noData}
                    label="No credentials reported"
                />
            );
        }

        return groupByPrincipal ? (
            <CredentialTableGrouped hostId={parsedHostId} data={creds} />
        ) : (
            <CredentialTable hostId={parsedHostId} data={creds} />
        );
    };

    return (
        <div className="flex flex-col w-full gap-2 mt-2">
            <div className="flex flex-row justify-between items-center border-b border-gray-200 bg-white gap-2 py-2 sticky top-0 z-5 shadow-sm">
                <div className="flex flex-row gap-2 items-center">
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">
                        Credentials
                    </h3>
                    <p className="text-md text-gray-600">{totalCount !== undefined && `(${totalCount})`}</p>
                </div>
                <div className="flex flex-row justify-end items-center gap-2 w-full">
                    <div className="flex flex-row items-center gap-1 mr-2">
                        <FormLabel htmlFor="groupByPrincipal" className="mb-0 cursor-pointer">
                            <span className="text-sm font-medium text-gray-700 whitespace-nowrap">
                                Group by Principal/Kind
                            </span>
                        </FormLabel>
                        <Switch
                            id="groupByPrincipal"
                            colorScheme="purple"
                            isChecked={groupByPrincipal}
                            onChange={() => setGroupByPrincipal((value: boolean) => !value)}
                        />
                    </div>
                    <SortingControls sortType={PageNavItem.credentials} />
                    <FreeTextSearch
                        labelVisible={false}
                        placeholder={searchPlaceholder}
                        defaultValue={search}
                        setSearch={setSearch}
                    />
                </div>
            </div>

            {renderTableContent()}
        </div>
    );
};
export default CredentialTab;
