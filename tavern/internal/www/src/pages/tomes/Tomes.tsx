import { ReactElement, useState } from "react";
import { PageWrapper } from "../../features/page-wrapper";
import { PageNavItem } from "../../utils/enums";

import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import ImportRepositoryModal from "./components/ImportRepositoryModal";
import RepositoryTable from "./components/RepositoryTable";
import { useRepositoryView } from "./hooks/useRepositoryView";
import TomesHeader from "./components/TomesHeader";

export const Tomes = (): ReactElement => {
    const [isOpen, setOpen] = useState<boolean>(false);
    const { loading, repositories, error } = useRepositoryView();

    return (
        <PageWrapper currNavItem={PageNavItem.tomes}>
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
        </PageWrapper>
    )
}
