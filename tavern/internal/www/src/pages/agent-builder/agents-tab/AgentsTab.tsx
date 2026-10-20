import { useCallback } from "react";
import { ApolloError, useMutation } from "@apollo/client";
import { useToast } from "@chakra-ui/react";
import { PlusIcon } from "@heroicons/react/24/outline";
import Table from "../../../components/tavern-base-ui/table/Table";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import Button from "../../../components/tavern-base-ui/button/Button";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import { DELETE_ASSET } from "../../assets/queries";
import { BuildTaskRow } from "../types";
import { GET_BUILD_TASKS } from "../queries";
import { buildTaskColumns } from "./AgentsTable";

/**
 * "Built agents" tab -- one row per BuildTask (see buildTaskRows in
 * queries.ts), so queued/claimed/building tasks render with a live status
 * badge and no artifact yet instead of a separate "build in progress"
 * banner; the table itself is the live status view.
 */
export const AgentsTab = ({
    rows,
    loading,
    error,
    highlightTaskId,
    onCreateLink,
    onGenerateAgent,
}: {
    rows: BuildTaskRow[];
    loading: boolean;
    error?: ApolloError;
    highlightTaskId?: string | null;
    onCreateLink: (assetId: string, assetName: string) => void;
    onGenerateAgent: () => void;
}) => {
    const toast = useToast();
    const [deleteAsset] = useMutation(DELETE_ASSET, {
        refetchQueries: [{ query: GET_BUILD_TASKS }],
        onCompleted: () => {
            toast({ title: "Asset deleted", status: "success", duration: 2000, isClosable: true });
        },
        onError: (err) => {
            toast({ title: "Failed to delete asset", description: err.message, status: "error", duration: 4000, isClosable: true });
        },
    });
    const handleDeleteAsset = useCallback((assetId: string) => {
        deleteAsset({ variables: { assetID: assetId } });
    }, [deleteAsset]);

    return (
        <div className="flex flex-col gap-2">
            <div className="flex flex-row justify-between items-center border-b border-gray-200 bg-white gap-2 py-2 sticky top-0 z-5 shadow-sm">
                <div className="flex flex-row gap-2 items-center">
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">Built agents</h3>
                    <p className="text-md text-gray-600">{rows.length !== undefined && `(${rows.length})`}</p>
                </div>
                <div className="flex flex-row justify-end gap-2 w-full">
                    <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "gray", size: "sm" }}
                        leftIcon={<PlusIcon className="w-4 h-4" />}
                        onClick={onGenerateAgent}
                    >
                        Generate agent
                    </Button>
                </div>
            </div>
            {error && <AlertError label="Unable to load build tasks" details={error.message} />}
            {loading && rows.length === 0 ? (
                <p role="status">Loading built agents…</p>
            ) : rows.length === 0 ? (
                <div className="rounded-md border border-gray-200 border-t-0 overflow-hidden">
                    <EmptyState
                        type={EmptyStateType.noData}
                        label="No built agents yet"
                        details="Generate an agent to get started."
                    >
                        <Button
                            leftIcon={<PlusIcon className="w-4 h-4" />}
                            buttonStyle={{ color: "purple", size: "md" }}
                            onClick={onGenerateAgent}
                        >
                            Generate agent
                        </Button>
                    </EmptyState>
                </div>
            ) : (
                <div className="rounded-md border border-gray-200 border-t-0 overflow-x-auto max-w-full">
                    <div className="min-w-[1060px] w-full">
                        <Table data={rows} columns={buildTaskColumns(onCreateLink, highlightTaskId, handleDeleteAsset)} />
                    </div>
                </div>
            )}
        </div>
    );
};

export default AgentsTab;
