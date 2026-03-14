import Button from "../../../components/tavern-base-ui/button/Button";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import FreeTextSearch from "../../../components/tavern-base-ui/FreeTextSearch";
import { SortingControls } from "../../../context/SortContext";
import { FilesTable } from "./FilesTable";
import { useFileIds } from "./useFileIds";
import { useParams } from "react-router-dom";
import { PageNavItem } from "../../../utils/enums";
import { useCreateQuestModal } from "../../../context/CreateQuestModalContext";
import { useFilesQuestFormData } from "./useFilesQuestFormData";
import { FileTerminal } from "lucide-react";

const FilesTab = () => {
    const { hostId } = useParams();

    const {
        fileIds,
        totalCount,
        initialLoading,
        error,
        searchTerm,
        setSearchTerm,
    } = useFileIds(hostId || "");

    const { openModal } = useCreateQuestModal();
    const { fetchFormData, loading } = useFilesQuestFormData();

    const handleClick = async () => {
        const initialFormData = await fetchFormData();
        openModal({
            initialFormData,
            refetchQueries: ["GetHostContext", "GetFileIds"],
        });
    };

        const renderTableContent = () => {
            if (error || !hostId) {
                return (
                    <EmptyState
                        type={EmptyStateType.error}
                        label="Error loading data"
                        details={error?.message || "Host ID not found"}
                    />
                );
            }

            if (initialLoading || totalCount === undefined) {
                return (
                    <EmptyState
                        type={EmptyStateType.loading}
                        label="Loading data..."
                    />
                );
            }

            if (totalCount === 0 && searchTerm.trim() !== "") {
                return (
                    <EmptyState
                        type={EmptyStateType.noMatches}
                        label="No data matching your filters"
                    >
                        <Button
                            onClick={() => setSearchTerm("")}
                            buttonVariant="solid"
                            buttonStyle={{ color: "purple", size: "md" }}
                        >
                            Clear filters
                        </Button>
                    </EmptyState>
                );
            }

            if (totalCount === 0 ) {
                return (
                    <EmptyState
                        type={EmptyStateType.noData}
                        label="No files reported"
                    >
                        <Button
                            leftIcon={<FileTerminal className="w-5 h-5" />}
                            onClick={handleClick}
                            buttonVariant="solid"
                            buttonStyle={{ color: "purple", size: "md" }}
                            disabled={loading}
                        >
                            {loading ? "Loading..." : "Report file"}
                        </Button>
                    </EmptyState>
                );
            }

            return (
                <FilesTable
                    hostId={hostId || ""}
                    fileIds={fileIds}
                />
            );
        };

        return (
            <div className="flex flex-col w-full gap-2 mt-2">
                <div className="flex flex-row justify-between items-center border-b border-gray-200 bg-white gap-2 py-2 sticky top-0 z-5 shadow-sm">
                    <div className='flex flex-row gap-2 items-center'>
                        <h3 className="text-xl font-semibold leading-6 text-gray-900">
                            Files
                        </h3>
                        <p className='text-md text-gray-600'>{totalCount !== undefined && `(${totalCount})`}</p>
                    </div>
                    <div className="flex flex-row justify-end gap-2 w-full">
                        <Button
                            leftIcon={<FileTerminal className="w-4 h-4" />}
                            onClick={handleClick}
                            buttonVariant="ghost"
                            buttonStyle={{ color: "gray", size: "sm" }}
                            disabled={loading}
                        >
                            {loading ? "Loading..." : "Report file"}
                        </Button>
                        <SortingControls sortType={PageNavItem.files} />
                        <FreeTextSearch
                            labelVisible={false}
                            placeholder="Search by path or hash"
                            defaultValue={searchTerm}
                            setSearch={setSearchTerm}
                        />
                    </div>
                </div>

                {renderTableContent()}
            </div>
        );
}

export default FilesTab;
