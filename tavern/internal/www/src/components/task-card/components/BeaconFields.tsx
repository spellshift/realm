import { FC } from "react";
import { Link } from "react-router-dom";
import { BeaconNode, TagEdge } from "../../../utils/interfacesQuery";
import { checkIfBeaconOffline, getEnumKey } from "../../../utils/utils";
import Badge from "../../tavern-base-ui/badge/Badge";
import { Bug, Globe, Network } from "lucide-react";
import { PrincipalAdminTypes, SupportedPlatforms, SupportedTransports } from "../../../utils/enums";

interface BeaconFieldsProps {
    beacon: BeaconNode;
}

const BeaconFields: FC<BeaconFieldsProps> = ({ beacon }) => {
    const { host, principal, name, transport } = beacon;
    const beaconOffline = checkIfBeaconOffline(beacon);
    const principalAdminValues = Object.values(PrincipalAdminTypes);
    const principalColor = principalAdminValues.includes(principal as PrincipalAdminTypes) ? "purple" : "gray";

    return (
        <div className="flex flex-row gap-2 items-center text-sm flex-wrap">
            <Bug fill="currentColor" className="w-4 h-4" />
            <Link
                to={`/hosts/${host?.id}`}
                className="text-purple-800 underline font-semibold cursor-pointer pr-2 text-wrap"
            >
                {name}@{host?.name}
            </Link>
            {principal && principal !== "" && (
                <Badge badgeStyle={{ color: principalColor }}>{principal}</Badge>
            )}
            {transport && (
                <Badge>{getEnumKey(SupportedTransports, transport)}</Badge>
            )}
            {host?.primaryIP && (
                <Badge leftIcon={<Network className="h-3 w-3" />}>{host.primaryIP}</Badge>
            )}
            {host?.externalIP && (
                <Badge leftIcon={<Globe className="h-3 w-3" />}>{host.externalIP}</Badge>
            )}
            {host?.platform && (
                <Badge>{getEnumKey(SupportedPlatforms, host.platform)}</Badge>
            )}
            {host?.tags?.edges?.map((tag: TagEdge) => (
                <Badge key={tag.node.id}>{tag.node.name}</Badge>
            ))}
            {beaconOffline && <Badge>Offline</Badge>}
        </div>
    );
};

export default BeaconFields;
