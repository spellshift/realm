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
        <div className="flex flex-col gap-2">
            <div className="flex flex-row justify-between items-center border-b border-gray-200 bg-white gap-2 py-2 sticky top-0 z-5 shadow-sm">
                <div className="flex flex-row gap-2 items-center">
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">Builders</h3>
                    <p className="text-md text-gray-600">{builders.length !== undefined && `(${builders.length})`}</p>
                </div>
                <div className="flex flex-row justify-end gap-2 w-full">
                    <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "gray", size: "sm" }}
                        leftIcon={<PlusIcon className="w-4 h-4" />}
                        onClick={() => setEnrolling(true)}
                    >
                        Enroll builder
                    </Button>
                </div>
            </div>
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
                <div className="rounded-md border border-gray-200 border-t-0 overflow-x-auto max-w-full">
                    <div className="min-w-[720px] w-full">
                        <Table data={builders} columns={builderColumns} />
                    </div>
                </div>
            )}
            <EnrollBuilderModal
                isOpen={enrolling}
                onClose={() => setEnrolling(false)}
                onEnrolled={onEnrolled}
            />
        </div>
    );
};

export default BuildersTab;
