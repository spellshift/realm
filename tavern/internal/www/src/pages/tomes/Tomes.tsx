import { ReactElement, useState } from "react";

import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import ImportRepositoryModal from "./import-modal/ImportRepositoryModal";
import RepositoryTable from "./components/RepositoryTable";
import { useRepositoryView } from "./hooks/useRepositoryView";
import TomesHeader from "./TomesHeader";

export const Tomes = (): ReactElement => {
    const [isOpen, setOpen] = useState<boolean>(false);
    const { loading, repositories, error } = useRepositoryView();

    return (
        <>
            <TomesHeader setOpen={setOpen} />
            <div>
                {loading ? (
                    <EmptyState type={EmptyStateType.loading} label="Loading quest repositories..." />
                ) : error ? (
                    <EmptyState type={EmptyStateType.error} label="Error loading repositories..." />
                ) : repositories && repositories.length > 0 ? (
                    <RepositoryTable repositories={repositories} />
                ) : <EmptyState type={EmptyStateType.noData} label="No repository data" />
                }

            </div>
            {isOpen && <ImportRepositoryModal isOpen={isOpen} setOpen={setOpen} />}
        </>
    )
}
