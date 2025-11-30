import { useState } from "react";
import { ComputerDesktopIcon, MapPinIcon, TagIcon } from "@heroicons/react/20/solid";
import { useHost } from "../../../context/HostContext";
import PageHeader from "../../../components/tavern-base-ui/PageHeader";
import TagModal from "./TagModal";
import EditableTag from "./editable-tag/EditableTag";

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
                                IP Address
                            </h4>
                        </div>
                        <div className="text-gray-600 text-sm ml-6 min-h-[38px] flex flex-col justify-center">
                            <div>
                                {(host && host?.primaryIP) ? host?.primaryIP : '-'}
                            </div>
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
                            <div>
                                {(host && host?.platform ? host?.platform : '-')}
                            </div>
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
