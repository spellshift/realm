import { ApolloError } from "@apollo/client";
import { PlusIcon } from "@heroicons/react/24/outline";
import Table from "../../../components/tavern-base-ui/table/Table";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import Button from "../../../components/tavern-base-ui/button/Button";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import { BuildTaskRow } from "../types";
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
}) => (
    <div className="flex flex-col gap-2">
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
            <>
                <div className="rounded-md border border-gray-200 border-t-0 overflow-x-auto max-w-full">
                    <div className="min-w-[720px] w-full">
                        <Table data={rows} columns={buildTaskColumns(onCreateLink, highlightTaskId)} />
                    </div>
                </div>
                <Button
                    buttonVariant="ghost"
                    buttonStyle={{ color: "purple", size: "sm" }}
                    leftIcon={<PlusIcon className="w-4 h-4" />}
                    onClick={onGenerateAgent}
                    className="w-fit"
                >
                    Generate agent
                </Button>
            </>
        )}
    </div>
);

export default AgentsTab;
