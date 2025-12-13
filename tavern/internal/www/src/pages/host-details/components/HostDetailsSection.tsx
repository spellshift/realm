import { useState } from "react";
import { ComputerDesktopIcon, MapPinIcon, TagIcon } from "@heroicons/react/20/solid";
import { useHost } from "../../../context/HostContext";
import PageHeader from "../../../components/tavern-base-ui/PageHeader";
import TagModal from "./TagModal";
import EditableTag from "./editable-tag/EditableTag";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import { Globe, Network } from "lucide-react";

const HostDetailsSection = () => {
    const [isOpen, setOpen] = useState(false);
    const { data: host } = useHost();

    const serviceTag = host?.tags?.edges && host.tags.edges[host.tags.edges.findIndex((tag) => tag.node.kind === "service")]?.node;
    const groupTag = host?.tags?.edges && host.tags.edges[host.tags.edges.findIndex((tag) => tag.node.kind === "group")]?.node;

    return (
        <div className="flex flex-col gap-4">
            <PageHeader title={(host && host?.name) ? host?.name : '-'} />
            <div className="">
                <div className="grid grid-cols-4 gap-2 ">
                    <div className="flex flex-col justify-between">
                        <div className="flex flex-row gap-2 items-center">
                            <MapPinIcon className="w-4 text-gray-700" />
                            <h4 className="text-gray-700">
                                IP Addresses
                            </h4>
                        </div>
                        <div className="text-gray-600 text-sm ml-6 min-h-[38px] flex flex-col justify-center gap-1">
                            {host?.primaryIP && (
                                <Badge leftIcon={<Network className="h-3 w-3" />}>{host?.primaryIP}</Badge>
                            )}
                            {host?.externalIP && (
                                <Badge leftIcon={<Globe className="h-3 w-3" />}>{host?.externalIP}</Badge>
                            )}
                        </div>
                    </div>
                    <div className="flex flex-col justify-between">
                        <div className="flex flex-row gap-2 items-center">
                            <ComputerDesktopIcon className="w-4 text-gray-700" />
                            <h4 className="text-gray-700">
                                Platform
                            </h4>
                        </div>
                        <div className="text-gray-600 text-sm ml-6  min-h-[38px] flex flex-col justify-center">
                            {host?.platform && (
                                <Badge>{host?.platform}</Badge>
                            )}
                        </div>
                    </div>
                    <div className="flex flex-col justify-between">
                        <div className="flex flex-row gap-2 items-center">
                            <TagIcon className="w-4 text-gray-700" />
                            <h4 className="text-gray-700">
                                Service
                            </h4>
                        </div>
                        <EditableTag tagSaved={serviceTag} kind="service" hostId={host?.id} />
                    </div>
                    <div className="flex flex-col justify-between">
                        <div className="flex flex-row gap-2 items-center">
                            <TagIcon className="w-4 text-gray-700" />
                            <h4 className="text-gray-700">
                                Group
                            </h4>
                        </div>
                        <EditableTag tagSaved={groupTag} kind="group" hostId={host?.id} />
                    </div>
                </div>
            </div>
            {isOpen && <TagModal isOpen={isOpen} setOpen={setOpen} />}
        </div>
    );
}
export default HostDetailsSection;
