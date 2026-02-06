import { ArrowDownToLine, Link, ChevronDown, ChevronRight } from "lucide-react";
import { format } from "date-fns";
import { AssetEdge, AssetNode } from "../../../utils/interfacesQuery";
import Button from "../../../components/tavern-base-ui/button/Button";
import { Tooltip } from "@chakra-ui/react";
import { Row } from "@tanstack/react-table";

type AssetAccordionProps = {
    asset: AssetNode;
};

const AssetAccordion = ({ asset }: AssetAccordionProps) => {
    return (
        <div className="px-8 py-4 flex flex-col gap-4 bg-gray-50 rounded-md">
            <div className="flex flex-col gap-2">
                <h4 className="font-semibold text-sm text-gray-700">Links ({asset.links?.totalCount || 0})</h4>
                {asset.links?.edges && asset.links.edges.length > 0 ? (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
                         {asset.links.edges.map((edge) => (
                             <div key={edge.node.id} className="p-2 bg-white border rounded shadow-sm flex flex-col gap-1 text-sm">
                                 <div className="font-mono text-xs break-all">{edge.node.path}</div>
                                 <div className="text-gray-500 text-xs">Expires: {format(new Date(edge.node.expiresAt), "yyyy-MM-dd HH:mm")}</div>
                                 <div className="text-gray-500 text-xs">Downloads remaining: {edge.node.downloadsRemaining}</div>
                             </div>
                         ))}
                    </div>
                ) : (
                    <div className="text-sm text-gray-500 italic">No links created</div>
                )}
            </div>

             <div className="flex flex-col gap-2">
                <h4 className="font-semibold text-sm text-gray-700">Tomes ({asset.tomes?.totalCount || 0})</h4>
                {asset.tomes?.edges && asset.tomes.edges.length > 0 ? (
                    <div className="flex flex-wrap gap-2">
                         {asset.tomes.edges.map((edge) => (
                             <div key={edge.node.id} className="px-2 py-1 bg-purple-100 text-purple-800 rounded text-sm border border-purple-200">
                                 {edge.node.name}
                             </div>
                         ))}
                    </div>
                ) : (
                    <div className="text-sm text-gray-500 italic">No associated tomes</div>
                )}
            </div>
        </div>
    );
};

export default AssetAccordion;

export { AssetAccordion }; // Export if needed elsewhere
