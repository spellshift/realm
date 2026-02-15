import { ReactElement, useState } from "react";

import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import ImportRepositoryModal from "./import-modal/ImportRepositoryModal";
import { RepositoriesTable } from "./repository-table/RepositoriesTable";
import { useRepositoryIds } from "./hooks/useRepositoryIds";
import TomesHeader from "./TomesHeader";

const Tomes = (): ReactElement => {
    const [isOpen, setOpen] = useState<boolean>(false);
    const { repositoryIds, loading, error } = useRepositoryIds();

    return (
        <>
            <TomesHeader setOpen={setOpen} />
            <div>
                {loading ? (
                    <EmptyState type={EmptyStateType.loading} label="Loading quest repositories..." />
                ) : error ? (
                    <EmptyState type={EmptyStateType.error} label="Error loading repositories..." />
                ) : repositoryIds && repositoryIds.length > 0 ? (
                    <RepositoriesTable repositoryIds={repositoryIds} />
                ) : <EmptyState type={EmptyStateType.noData} label="No repository data" />
                }

            </div>
            {isOpen && <ImportRepositoryModal isOpen={isOpen} setOpen={setOpen} />}
        </>
    )
}

export default Tomes;
