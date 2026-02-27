import React from "react";
import { Link } from "react-router-dom";
import { Bug } from "lucide-react";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Breadcrumbs from "../../../components/Breadcrumbs";
import { Tooltip } from "@chakra-ui/react";
import PlaceholderUser from "../../../assets/PlaceholderUser.png";

interface ShellHeaderProps {
  shellData: any;
  activeUsers?: { id: string; name: string; photoURL?: string }[];
}

const ShellHeader: React.FC<ShellHeaderProps> = ({ shellData, activeUsers = [] }) => {
  const beaconName = shellData?.node?.beacon?.name;
  const hostName = shellData?.node?.beacon?.host?.name;
  const hostId = shellData?.node?.beacon?.host?.id;

  return (
    <div className="flex items-center gap-4 mb-4">
      <Breadcrumbs pages={[{ label: "Shell", link: window.location.pathname }]} />
      <Badge badgeStyle={{ color: "red" }}>BETA</Badge>
      <h1 className="text-xl font-bold">
        {beaconName} @ <Link to={`/hosts/${hostId}`} className="text-blue-400 hover:text-blue-300 underline">{hostName}</Link>
      </h1>
      <a
        href="https://github.com/spellshift/realm/issues/new?template=bug_report.md&labels=bug&title=%5Bbug%5D%%20Shell%3A%20%3CYOUR%20ISSUE%3E"
        target="_blank"
        rel="noopener noreferrer"
        className="text-gray-400 hover:text-white transition-colors"
        title="Report a bug"
      >
        <Bug size={20} />
      </a>

      {/* Active Users Display */}
      <div className="ml-auto flex items-center gap-2">
        {activeUsers.length > 0 && (
          <div className="flex -space-x-2">
            {activeUsers.map((user) => {
              const userImage = (user.photoURL && user.photoURL !== "") ? user.photoURL : PlaceholderUser;
              return (
                <Tooltip key={user.id} label={user.name} hasArrow>
                  <img
                    src={userImage}
                    alt={user.name}
                    className="w-8 h-8 rounded-full border-2 border-[#1e1e1e] cursor-help object-cover"
                    title={user.name}
                  />
                </Tooltip>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};

export default ShellHeader;
