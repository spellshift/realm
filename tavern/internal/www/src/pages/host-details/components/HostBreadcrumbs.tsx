import { useContext } from "react";
import Breadcrumbs from "../../../components/Breadcrumbs";
import { HostContext } from "../../../context/HostContext";

const HostBreadcrumbs = () => {
    const { data } = useContext(HostContext);
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
