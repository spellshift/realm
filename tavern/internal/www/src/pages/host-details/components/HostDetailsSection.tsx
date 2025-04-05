import React, { useContext } from "react";
import { ComputerDesktopIcon, MapPinIcon, TagIcon } from "@heroicons/react/20/solid";
import { HostContext } from "../../../context/HostContext";
import PageHeader from "../../../components/tavern-base-ui/PageHeader";

const HostDetailsSection = () => {
    // const [isOpen, setOpen] = useState<boolean>(false);
    const { data: host, loading, error } = useContext(HostContext);

    const serviceTag = host?.tags && host.tags[host.tags.findIndex((tomeTag) => tomeTag.kind === "service")];
    const groupTag = host?.tags && host.tags[host.tags.findIndex((tomeTag) => tomeTag.kind === "group")];

    return (
        <div className="flex flex-col gap-4">
            <PageHeader title={(host && host?.name) ? host?.name : '-'} />
            <div className="">
                <div className="grid grid-cols-4 gap-2">
                    <div className="flex flex-col">
                        <div className="flex flex-row gap-2 items-center">
                            <MapPinIcon className="w-4 text-gray-700" />
                            <h4 className="text-gray-700">
                                IP Address
                            </h4>
                        </div>
                        <div className="text-gray-600 text-sm ml-6">
                            {(host && host?.primaryIP) ? host?.primaryIP : '-'}
                        </div>
                    </div>
                    <div className="flex flex-col">
                        <div className="flex flex-row gap-2 items-center">
                            <ComputerDesktopIcon className="w-4 text-gray-700" />
                            <h4 className="text-gray-700">
                                Platform
                            </h4>
                        </div>
                        <div className="text-gray-600 text-sm ml-6">
                            {(host && host?.platform ? host?.platform : '-')}
                        </div>
                    </div>
                    <div className="flex flex-col">
                        <div className="flex flex-row gap-2 items-center">
                            <TagIcon className="w-4 text-gray-700" />
                            <h4 className="text-gray-700">
                                Service
                            </h4>
                        </div>
                        <div className="text-gray-600 text-sm ml-6 flex flex-row gap-1 items-center">
                            {serviceTag?.name}
                            {/* <Button
                                buttonVariant="ghost"
                                className="p-0"
                                leftIcon={<PencilSquareIcon className="w-4" />}
                                buttonStyle={{ color: "gray", size: "md" }}
                                aria-label="Edit service tag"
                                onClick={() => setOpen(true)}
                            /> */}
                        </div>
                    </div>
                    <div className="flex flex-col">
                        <div className="flex flex-row gap-2 items-center">
                            <TagIcon className="w-4 text-gray-700" />
                            <h4 className="text-gray-700">
                                Group
                            </h4>
                        </div>
                        <div className="text-gray-600 text-sm ml-6 flex flex-row gap-2 items-center">
                            {groupTag?.name}
                            {/* <Button
                                buttonVariant="ghost"
                                className="p-0"
                                leftIcon={<PencilSquareIcon className="w-4" />}
                                buttonStyle={{ color: "gray", size: "md" }}
                                aria-label="Edit group tag"
                            /> */}
                        </div>
                    </div>
                </div>
            </div>
            {/* {isOpen && <ModifyHostTagModal isOpen={isOpen} setOpen={setOpen} />} */}
        </div>
    );
}
export default HostDetailsSection;
