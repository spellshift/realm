import { useState } from "react";
import { useAssets } from "./useAssets";
import AssetsHeader from "./components/AssetsHeader";
import AssetsTable from "./components/AssetsTable";
import CreateLinkModal from "./components/CreateLinkModal/CreateLinkModal";
import UploadAssetModal from "./components/UploadAssetModal/UploadAssetModal";
import TableWrapper from "../../components/tavern-base-ui/table/TableWrapper";
import TablePagination from "../../components/tavern-base-ui/table/TablePagination";
import { useFilters } from "../../context/FilterContext";

export const Assets = () => {
    const rowLimit = 10;
    const { filters } = useFilters();
    const where: any = filters.assetName ? { nameContains: filters.assetName } : {};
    if (filters.creatorId) {
        where.hasCreatorWith = [{ id: filters.creatorId }];
    }

    const { assets, loading, error, totalCount, pageInfo, refetch, updateAssets, page, setPage } = useAssets(rowLimit, where);
    const [createLinkModalOpen, setCreateLinkModalOpen] = useState(false);
    const [uploadAssetModalOpen, setUploadAssetModalOpen] = useState(false);
    const [selectedAsset, setSelectedAsset] = useState<{ id: string; name: string } | null>(null);

    const handleCreateLink = (assetId: string, assetName: string) => {
        setSelectedAsset({ id: assetId, name: assetName });
        setCreateLinkModalOpen(true);
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
        </>
    );
};

export default Assets;
