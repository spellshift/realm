import { Copy, Link2Off } from "lucide-react";
import moment from "moment";
import { AssetNode } from "../../../utils/interfacesQuery";
import Button from "../../../components/tavern-base-ui/button/Button";
import { useToast, Tooltip } from "@chakra-ui/react";
import { useDisableLink } from "../useAssets";

type AssetAccordionProps = {
    asset: AssetNode;
};

const AssetAccordion = ({ asset }: AssetAccordionProps) => {
    const toast = useToast();
    const { disableLink } = useDisableLink();

    const handleCopyLink = (path: string) => {
        const url = `${window.location.origin}/cdn/${path}`;
        navigator.clipboard.writeText(url);
        toast({
            title: "Link copied to clipboard",
            status: "success",
            duration: 2000,
            isClosable: true,
        });
    };

    return (
        <div className="px-12 py-4 flex flex-col gap-4 bg-gray-50 rounded-b-md border-t border-gray-100">
            <div className="flex flex-col gap-3">
                <h4 className="font-semibold text-sm text-gray-900">Links ({asset.links?.totalCount || 0})</h4>
                {asset.links?.edges && asset.links.edges.length > 0 ? (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                         {asset.links.edges.map((edge) => (
                             <div key={edge.node.id} className="p-3 bg-white border border-gray-200 rounded-md shadow-sm flex flex-col gap-2 text-sm">
                                 <div className="flex justify-between items-start gap-2">
                                     <div className="font-mono text-xs break-all text-gray-600 bg-gray-50 p-1 rounded w-full">
                                         {`${window.location.origin}/cdn/${edge.node.path}`}
                                     </div>
                                     <div className="flex flex-row items-center">
                                        <Button
                                            onClick={() => handleCopyLink(edge.node.path)}
                                            buttonVariant="ghost"
                                            buttonStyle={{ color: "gray", size: "xs" }}
                                            leftIcon={<Copy className="w-3 h-3" />}
                                            aria-label="Copy Link"
                                        />
                                        <Tooltip label="Expire link" bg="white" color="black">
                                            <Button
                                                onClick={async () => {
                                                    try {
                                                        await disableLink({ variables: { linkID: edge.node.id } });
                                                        toast({
                                                            title: "Link expired",
                                                            status: "success",
                                                            duration: 2000,
                                                            isClosable: true,
                                                        });
                                                    } catch (e) {
                                                        toast({
                                                            title: "Failed to expire link",
                                                            status: "error",
                                                            duration: 2000,
                                                            isClosable: true,
                                                        });
                                                    }
                                                }}
                                                buttonVariant="ghost"
                                                buttonStyle={{ color: "gray", size: "xs" }}
                                                leftIcon={<Link2Off className="w-3 h-3" />}
                                                aria-label="Expire Link"
                                            />
                                        </Tooltip>
                                     </div>
                                 </div>
                                 <div className="flex justify-between items-center text-xs text-gray-500 mt-1">
                                     <span>Expires: {moment(edge.node.expiresAt).format("MMM D, YYYY HH:mm")}</span>
                                     <span>{edge.node.downloadsRemaining} downloads left</span>
                                 </div>
                             </div>
                         ))}
                    </div>
                ) : (
                    <div className="text-sm text-gray-500 italic">No links created</div>
                )}
            </div>
        </div>
    );
};

export default AssetAccordion;
