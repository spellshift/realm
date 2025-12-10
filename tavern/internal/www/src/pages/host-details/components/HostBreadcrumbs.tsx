import Breadcrumbs from "../../../components/Breadcrumbs";
import { useHost } from "../../../context/HostContext";

const HostBreadcrumbs = () => {
    const { data } = useHost();
    const hostName = data?.name;
    const hostId = data?.id;

    const BreadcrumbsList = [
        {
            label: "Hosts",
            link: "/hosts"
        },
        {
            label: hostName,
            link: `/hosts/${hostId}`
        }
    ]

    return (
        <Breadcrumbs pages={BreadcrumbsList} />
    );
}
export default HostBreadcrumbs;
