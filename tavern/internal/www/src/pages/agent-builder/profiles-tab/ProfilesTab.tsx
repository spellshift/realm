import { ApolloError } from "@apollo/client";
import { PlusIcon } from "@heroicons/react/24/outline";
import Table from "../../../components/tavern-base-ui/table/Table";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import Button from "../../../components/tavern-base-ui/button/Button";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import { BuildProfile } from "../types";
import { profileColumns } from "./ProfilesTable";

/** "Build profiles" tab -- lists saved profiles, with "Add build profile" below the table. */
export const ProfilesTab = ({
    profiles,
    loading,
    error,
    onAddProfile,
}: {
    profiles: BuildProfile[];
    loading: boolean;
    error?: ApolloError;
    onAddProfile: () => void;
}) => (
    <>
        {error && <AlertError label="Unable to load build profiles" details={error.message} />}
        {loading && profiles.length === 0 ? (
            <p role="status">Loading build profiles…</p>
        ) : profiles.length === 0 ? (
            <div className="rounded-md border border-gray-200 border-t-0 overflow-hidden">
                <EmptyState
                    type={EmptyStateType.noData}
                    label="No build profiles available"
                    details="Add one to get started."
                >
                    <Button
                        leftIcon={<PlusIcon className="w-4 h-4" />}
                        buttonStyle={{ color: "purple", size: "md" }}
                        onClick={onAddProfile}
                    >
                        Add build profile
                    </Button>
                </EmptyState>
            </div>
        ) : (
            <div className="flex flex-col gap-2">
                <div className="rounded-md border border-gray-200 border-t-0 overflow-x-auto max-w-full">
                    <div className="min-w-[720px] w-full">
                        <Table data={profiles} columns={profileColumns} />
                    </div>
                </div>
                <Button
                    buttonVariant="ghost"
                    buttonStyle={{ color: "purple", size: "sm" }}
                    leftIcon={<PlusIcon className="w-4 h-4" />}
                    onClick={onAddProfile}
                    className="w-fit"
                >
                    Add build profile
                </Button>
            </div>
        )}
    </>
);

export default ProfilesTab;
