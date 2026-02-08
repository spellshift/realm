import { AssetEdge } from "../../../utils/interfacesQuery";
import Table from "../../../components/tavern-base-ui/table/Table";
import AssetAccordion from "./AssetAccordion";
import { useState, useEffect } from "react";
import { useAssetColumns } from "../hooks/useAssetColumns";

type AssetsTableProps = {
    assets: AssetEdge[];
    onCreateLink: (assetId: string, assetName: string) => void;
    onAssetUpdate: () => void;
};

const AssetsTable = ({ assets, onCreateLink, onAssetUpdate }: AssetsTableProps) => {
    const [windowWidth, setWindowWidth] = useState(window.innerWidth);

    useEffect(() => {
        const handleResize = () => {
            setWindowWidth(window.innerWidth);
        };
        window.addEventListener('resize', handleResize);
        return () => window.removeEventListener('resize', handleResize);
    }, []);

    const columns = useAssetColumns({ onCreateLink });

    const getVisibleColumns = () => {
        // Name and Actions always shown
        const visibleIds = ["expander", "name", "actions"];

        if (windowWidth >= 800) {
            visibleIds.push("creator");
        }
        if (windowWidth >= 1150) {
            visibleIds.push("size");
        }
        if (windowWidth > 1400) {
            visibleIds.push("lastModifiedAt");
        }
        if (windowWidth >= 1600) {
            visibleIds.push("hash");
        }
        if (windowWidth > 1900) {
            visibleIds.push("createdAt");
        }

        return columns.filter(col => visibleIds.includes(col.id as string));
    };

    const visibleColumns = getVisibleColumns();


    return (
        <Table
            data={assets}
            columns={visibleColumns}
            getRowCanExpand={(row) => row.original.node.links.totalCount > 0}
            onRowClick={(row, event) => {
                if (row.original.node.links.totalCount > 0) {
                    const toggle = row.getToggleExpandedHandler();
                    toggle();
                }
            }}
            renderSubComponent={({ row }) => <AssetAccordion asset={row.original.node} onUpdate={onAssetUpdate} />}
        />
    );
};

export default AssetsTable;
