import { Tab, TabList } from "@headlessui/react";
import { Hammer, Settings2, Users } from "lucide-react";

const tabClass = ({ selected }: { selected: boolean }) =>
    `p-3 sm:p-4 flex flex-row gap-1 items-center flex-shrink-0 whitespace-nowrap border-t-2 border-l-2 border-r-2 rounded-t-lg ${
        selected
            ? "border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100"
            : "border-transparent hover:bg-white hover:border-t-purple-600"
    }`;

export interface AdminTabsProps {
    userCount?: number;
    profileCount?: number;
    builderCount?: number;
}

export const AdminTabs = ({
    userCount,
    profileCount,
    builderCount,
}: AdminTabsProps) => (
    <TabList className="flex flex-row gap-2 border-gray-200 w-full min-w-0 overflow-x-auto bg-gray-100">
        <Tab className={tabClass}>
            <Users className="w-4 h-4" />
            <div>Users</div>
            {userCount !== undefined && <div>({userCount})</div>}
        </Tab>
        <Tab className={tabClass}>
            <Settings2 className="w-4 h-4" />
            <div>Profiles</div>
            {profileCount !== undefined && <div>({profileCount})</div>}
        </Tab>
        <Tab className={tabClass}>
            <Hammer className="w-4 h-4" />
            <div>Builders</div>
            {builderCount !== undefined && <div>({builderCount})</div>}
        </Tab>
    </TabList>
);

export default AdminTabs;
