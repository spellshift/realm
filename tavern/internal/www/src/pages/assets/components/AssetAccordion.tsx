import { Copy, Link2Off } from "lucide-react";
import { format } from "date-fns";
import { AssetNode, LinkEdge } from "../../../utils/interfacesQuery";
import Button from "../../../components/tavern-base-ui/button/Button";
import { useToast, Tooltip } from "@chakra-ui/react";
import { useMemo } from "react";
import { useDisableLink } from "../useAssets";
import UserImageAndName from "../../../components/UserImageAndName";

const API_ENDPOINT = process.env.REACT_APP_API_ENDPOINT ?? 'http://localhost:8000';

type AssetAccordionProps = {
    asset: AssetNode;
    onUpdate: () => void;
};

const AssetAccordion = ({ asset, onUpdate }: AssetAccordionProps) => {
    const toast = useToast();
    const { disableLink } = useDisableLink();

    const handleCopyLink = (path: string) => {
        const url = `${API_ENDPOINT}/cdn/${path}`;
        navigator.clipboard.writeText(url);
        toast({
            title: "Link copied to clipboard",
            status: "success",
            duration: 2000,
            isClosable: true,
        });
    };

    const handleDisableLink = async (linkId: string) => {
        try {
            await disableLink({ variables: { linkID: linkId } });
            toast({
                title: "Link disabled",
                status: "success",
                duration: 2000,
                isClosable: true,
            });
            onUpdate();
        } catch (e: any) {
            toast({
                title: "Error disabling link",
                description: e.message,
                status: "error",
                duration: 4000,
                isClosable: true,
            });
        }
    };

    const sortedLinks = useMemo(() => {
        if (!asset.links?.edges) return [];

        return [...asset.links.edges].sort((a: LinkEdge, b: LinkEdge) => {
            const now = new Date();
            const aExpired = new Date(a.node.expiresAt) < now;
            const bExpired = new Date(b.node.expiresAt) < now;

            // 1. Unexpired first
            if (aExpired !== bExpired) {
                return aExpired ? 1 : -1;
            }

            // 2. Unlimited downloads first
            const aUnlimited = a.node.downloadLimit === null;
            const bUnlimited = b.node.downloadLimit === null;
            if (aUnlimited !== bUnlimited) {
                return aUnlimited ? -1 : 1;
            }

            // 3. Most remaining downloads first
            if (!aUnlimited && !bUnlimited) {
                const aRemaining = (a.node.downloadLimit || 0) - a.node.downloads;
                const bRemaining = (b.node.downloadLimit || 0) - b.node.downloads;
                return bRemaining - aRemaining;
            }

            return 0;
        });
    }, [asset.links?.edges]);

    return (
        <div className="px-12 py-4 flex flex-col gap-4 bg-gray-50 rounded-b-md border-t border-gray-100">
            <div className="flex flex-col gap-3">
                <h4 className="font-semibold text-sm text-gray-900">Links ({asset.links?.totalCount || 0})</h4>
                {sortedLinks.length > 0 ? (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                         {sortedLinks.map((edge) => {
                             const isExpired = new Date(edge.node.expiresAt) < new Date();
                             const hasDownloadsRemaining = edge.node.downloadLimit === null || (edge.node.downloadLimit - edge.node.downloads > 0);

                             return (
                                 <div key={edge.node.id} className="p-3 bg-white border border-gray-200 rounded-md shadow-sm flex flex-col gap-2 text-sm">
                                     <div className="flex justify-between items-start gap-2">
                                         <div className="font-mono text-xs break-all text-gray-600 p-1 rounded w-full">
                                             {`/cdn/${edge.node.path}`}
                                         </div>
                                         {(!isExpired && hasDownloadsRemaining) && (
                                             <div className="flex gap-1">
                                                 <Tooltip label="Copy Link" bg="white" color="black">
                                                     <Button
                                                         onClick={() => handleCopyLink(edge.node.path)}
                                                         buttonVariant="ghost"
                                                         buttonStyle={{ color: "gray", size: "xs" }}
                                                         leftIcon={<Copy className="w-3 h-3" />}
                                                         aria-label="Copy Link"
                                                     />
                                                 </Tooltip>
                                                 <Tooltip label="Expire Link" bg="white" color="black">
                                                     <Button
                                                         onClick={() => handleDisableLink(edge.node.id)}
                                                         buttonVariant="ghost"
                                                         buttonStyle={{ color: "red", size: "xs" }}
                                                         leftIcon={<Link2Off className="w-3 h-3" />}
                                                         aria-label="Expire Link"
                                                     />
                                                 </Tooltip>
                                             </div>
                                         )}
                                     </div>
                                     <div className="flex justify-between items-center text-xs text-gray-500 mt-1">
                                         <span>
                                             {isExpired
                                                ? "Expired"
                                                : `Expires: ${format(new Date(edge.node.expiresAt), "MMM d, yyyy HH:mm")}`
                                             }
                                         </span>
                                         <span>{edge.node.downloads} {edge.node.downloadLimit != null ? '/ '+edge.node.downloadLimit : ''} downloads</span>
                                     </div>
                                     <div className="flex justify-between items-center mt-1 border-t border-gray-100 pt-2">
                                         <UserImageAndName userData={edge.node.creator} />
                                     </div>
                                 </div>
                             );
                         })}
                    </div>
                ) : (
                    <div className="text-sm text-gray-500 italic">No links created</div>
                )}
            </div>
        </div>
    );
};

export default AssetAccordion;
