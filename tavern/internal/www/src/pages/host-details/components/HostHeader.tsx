import { useSearchParams } from "react-router-dom";
import { useHost } from "../../../context/HostContext";
import { useCreateQuestModal } from "../../../context/CreateQuestModalContext";
import Button from "../../../components/tavern-base-ui/button/Button";
import { FileTerminal } from "lucide-react";
import Breadcrumbs from "../../../components/Breadcrumbs";

const HostHeader = () => {
    const [_, setSearchParams] = useSearchParams();
    const { openModal } = useCreateQuestModal();
    const { data: host } = useHost();

    const BreadcrumbsList = [
        {
            label: "Hosts",
            link: "/hosts"
        },
        {
            label: host?.name,
            link: `/hosts/${host?.id}`
        }
    ]

    return (
        <div className="flex flex-row justify-between w-full items-center">
            <Breadcrumbs pages={BreadcrumbsList} />
            <div>
                <Button
                    leftIcon={<FileTerminal className="w-5 h-5" />}
                    buttonStyle={{ color: "purple", size: "md" }}
                    onClick={() => {
                        openModal({
                            initialFormData: host ? {
                                initialFilters: {
                                    beaconFields: [{
                                        id: host.id,
                                        name: host.name,
                                        value: host.id,
                                        label: host.name,
                                        kind: "host",
                                    }]
                                }
                            } : undefined,
                            onComplete: () => {
                                setSearchParams({ tab: "tasks" });
                            }
                        });
                    }}
                >
                    Create a quest
                </Button>
            </div>
        </div>
    );
};

export default HostHeader;
