import { useState } from "react";
import { ApolloError } from "@apollo/client";
import { PlusIcon } from "@heroicons/react/24/outline";
import Table from "../../../components/tavern-base-ui/table/Table";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import Button from "../../../components/tavern-base-ui/button/Button";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import EnrollBuilderModal from "../components/EnrollBuilderModal";
import { Builder } from "../types";
import { builderColumns } from "./BuildersTable";

/** "Builders" tab -- lists enrolled builders, with "Enroll builder" below the table. */
export const BuildersTab = ({
    builders,
    loading,
    error,
    onEnrolled,
}: {
    builders: Builder[];
    loading: boolean;
    error?: ApolloError;
    onEnrolled: () => void;
}) => {
    const [enrolling, setEnrolling] = useState(false);

    return (
        <>
            {error && <AlertError label="Unable to load builders" details={error.message} />}
            {loading && builders.length === 0 ? (
                <p role="status">Loading builders…</p>
            ) : builders.length === 0 ? (
                <div className="rounded-md border border-gray-200 border-t-0 overflow-hidden">
                    <EmptyState
                        type={EmptyStateType.noData}
                        label="No builders available"
                        details="Add one to get started."
                    >
                        <Button
                            leftIcon={<PlusIcon className="w-4 h-4" />}
                            buttonStyle={{ color: "purple", size: "md" }}
                            onClick={() => setEnrolling(true)}
                        >
                            Enroll builder
                        </Button>
                    </EmptyState>
                </div>
            ) : (
                <div className="flex flex-col gap-2">
                    <div className="rounded-md border border-gray-200 border-t-0 overflow-x-auto max-w-full">
                        <div className="min-w-[720px] w-full">
                            <Table data={builders} columns={builderColumns} />
                        </div>
                    </div>
                    <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "purple", size: "sm" }}
                        leftIcon={<PlusIcon className="w-4 h-4" />}
                        onClick={() => setEnrolling(true)}
                        className="w-fit"
                    >
                        Enroll builder
                    </Button>
                </div>
            )}
            <EnrollBuilderModal
                isOpen={enrolling}
                onClose={() => setEnrolling(false)}
                onEnrolled={onEnrolled}
            />
        </>
    );
};

export default BuildersTab;
