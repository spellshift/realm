import { useState } from "react";
import { useAssets } from "./useAssets";
import AssetsHeader from "./components/AssetsHeader";
import AssetsTable from "./components/AssetsTable";
import CreateLinkModal from "./components/CreateLinkModal";
import UploadAssetModal from "./components/UploadAssetModal";
import TableWrapper from "../../components/tavern-base-ui/table/TableWrapper";
import TablePagination from "../../components/tavern-base-ui/table/TablePagination";
import { useFilters } from "../../context/FilterContext";

export const Assets = () => {
    const rowLimit = 10;
    const { filters } = useFilters();
    const where = filters.assetName ? { nameContains: filters.assetName } : undefined;
    const { assets, loading, error, totalCount, pageInfo, refetch, updateAssets, page, setPage } = useAssets(rowLimit, where);

    const [createLinkModalOpen, setCreateLinkModalOpen] = useState(false);
    const [uploadAssetModalOpen, setUploadAssetModalOpen] = useState(false);
    const [selectedAsset, setSelectedAsset] = useState<{ id: string; name: string } | null>(null);

    const handleCreateLink = (assetId: string, assetName: string) => {
        setSelectedAsset({ id: assetId, name: assetName });
        setCreateLinkModalOpen(true);
    };

    const handleRefresh = () => {
        refetch();
    };

    return (
        <>            
            <div className="flex flex-col gap-6">
                <AssetsHeader setOpen={setUploadAssetModalOpen} />
                <TableWrapper
                    totalItems={totalCount}
                    loading={loading}
                    error={error}
                    title="Assets"
                    table={<AssetsTable assets={assets} onCreateLink={handleCreateLink} onAssetUpdate={refetch} />}
                    pagination={
                         <TablePagination
                            totalCount={totalCount || 0}
                            pageInfo={pageInfo || { hasNextPage: false, hasPreviousPage: false, startCursor: null, endCursor: null }}
                            refetchTable={updateAssets}
                            page={page}
                            setPage={setPage}
                            rowLimit={rowLimit}
                            loading={loading}
                        />
                    }
                />
            </div>

            {selectedAsset && (
                <CreateLinkModal
                    isOpen={createLinkModalOpen}
                    setOpen={setCreateLinkModalOpen}
                    assetId={selectedAsset.id}
                    assetName={selectedAsset.name}
                    onSuccess={handleRefresh}
                />
            )}

            <UploadAssetModal
                isOpen={uploadAssetModalOpen}
                setOpen={setUploadAssetModalOpen}
                onUploadSuccess={handleRefresh}
            />
        </>
    );
};

export default Assets;
