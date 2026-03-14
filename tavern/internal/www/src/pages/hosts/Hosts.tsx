import Breadcrumbs from "../../components/Breadcrumbs";
import Button from "../../components/tavern-base-ui/button/Button";
import { VirtualizedTableWrapper } from "../../components/tavern-base-ui/virtualized-table";
import { HostsTable } from "./HostsTable";
import { useHostIds } from "./useHostIds";
import { useCreateQuestModal } from "../../context/CreateQuestModalContext";
import { PageNavItem } from "../../utils/enums";
import { FileTerminal, SquareTerminal } from "lucide-react";

const Hosts = () => {
    const {
        data,
        hostIds,
        initialLoading,
        error,
        hasMore,
        loadMore,
    } = useHostIds();
    const { openModal } = useCreateQuestModal();

    return (
        <>
            <div className="flex flex-row justify-between w-full items-center">
                <Breadcrumbs
                    pages={[{
                        label: "Hosts",
                        link: "/hosts"
                    }]}
                />
                <div>
                    <Button
                        leftIcon={<FileTerminal className="w-5 h-5" />}
                        buttonStyle={{ color: "purple", size: "md" }}
                        onClick={() => openModal({navigateToQuest: true})}
                    >
                        Create a quest
                    </Button>
                </div>
            </div>
            <VirtualizedTableWrapper
                title="Hosts"
                totalItems={data?.hosts?.totalCount}
                loading={initialLoading}
                error={error}
                sortType={PageNavItem.hosts}
                table={
                    <HostsTable
                        hostIds={hostIds}
                        hasMore={hasMore}
                        onLoadMore={loadMore}
                    />
                }
            />
        </>
    );
}

export default Hosts;
