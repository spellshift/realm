import React from "react";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Breadcrumbs from "../../../components/Breadcrumbs";

interface ShellHeaderProps {
  shellData: any;
}

const ShellHeader: React.FC<ShellHeaderProps> = ({ shellData }) => {
  return (
    <div className="flex items-center gap-4 mb-4">
      <Breadcrumbs pages={[{ label: "Shell", link: window.location.pathname }]} />
      <Badge badgeStyle={{ color: "purple" }}>Pre-alpha release</Badge>
      <h1 className="text-xl font-bold">Eldritch Shell for {shellData?.node?.beacon?.name}</h1>
    </div>
  );
};

export default ShellHeader;
