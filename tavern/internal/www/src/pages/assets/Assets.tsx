import { useState } from "react";
import { useAssetIds } from "./useAssetIds";
import AssetsHeader from "./components/AssetsHeader";
import AssetsTable from "./AssetsTable";
import CreateLinkModal from "./components/CreateLinkModal/CreateLinkModal";
import UploadAssetModal from "./components/UploadAssetModal/UploadAssetModal";
import GenerateAgentModal from "./components/GenerateAgentModal/GenerateAgentModal";
import { VirtualizedTableWrapper } from "../../components/tavern-base-ui/virtualized-table";
import { PageNavItem } from "../../utils/enums";

export const Assets = () => {
    const {
        data,
        assetIds,
        initialLoading,
        error,
        hasMore,
        loadMore,
        refetch,
    } = useAssetIds();

    const [createLinkModalOpen, setCreateLinkModalOpen] = useState(false);
    const [uploadAssetModalOpen, setUploadAssetModalOpen] = useState(false);
    const [generateAgentModalOpen, setGenerateAgentModalOpen] = useState(false);
    const [selectedAsset, setSelectedAsset] = useState<{ id: string; name: string } | null>(null);

    const handleCreateLink = (assetId: string, assetName: string) => {
        setSelectedAsset({ id: assetId, name: assetName });
        setCreateLinkModalOpen(true);
    };

    return (
        <>
            <div className="flex flex-col gap-6">
                <AssetsHeader
                    setOpen={setUploadAssetModalOpen}
                    onGenerateAgent={() => setGenerateAgentModalOpen(true)}
                />
                <VirtualizedTableWrapper
                    title="Assets"
                    totalItems={data?.assets?.totalCount}
                    loading={initialLoading}
                    error={error}
                    sortType={PageNavItem.assets}
                    table={
                        <AssetsTable
                            assetIds={assetIds}
                            hasMore={hasMore}
                            onLoadMore={loadMore}
                            onCreateLink={handleCreateLink}
                            onAssetUpdate={refetch}
                        />
                    }
                />
            </div>

            {createLinkModalOpen && selectedAsset && (
                <CreateLinkModal
                    isOpen={createLinkModalOpen}
                    setOpen={setCreateLinkModalOpen}
                    assetId={selectedAsset.id}
                    assetName={selectedAsset.name}
                    onSuccess={refetch}
                />
            )}

            {uploadAssetModalOpen && (
                <UploadAssetModal
                    isOpen={uploadAssetModalOpen}
                    setOpen={setUploadAssetModalOpen}
                    onUploadSuccess={refetch}
                />
            )}

            {generateAgentModalOpen && (
                <GenerateAgentModal
                    isOpen={generateAgentModalOpen}
                    setOpen={setGenerateAgentModalOpen}
                    onSuccess={refetch}
                />
            )}
        </>
    );
};

export default Assets;
