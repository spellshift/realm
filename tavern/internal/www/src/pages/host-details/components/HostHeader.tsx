import { useSearchParams } from "react-router-dom";
import { useHost } from "../../../context/HostContext";
import { CreateQuestDropdown } from "../../../components/create-quest-modal/CreateQuestDropdown";
import Breadcrumbs from "../../../components/Breadcrumbs";
import HostSubscriptionButton from "./HostSubscriptionButton";

const HostHeader = () => {
    const [, setSearchParams] = useSearchParams();
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
            <div className="flex flex-row gap-2 items-center">
                {host?.id && <HostSubscriptionButton hostId={host.id} />}
                <CreateQuestDropdown primaryOpenOptions={{
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
                }} />
            </div>
        </div>
    );
};

export default HostHeader;
