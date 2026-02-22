import { Tabs } from "@chakra-ui/react";
import { HostContextProvider } from "../../context/HostContext";
import HostDetailsSection from "./components/HostDetailsSection";
import HostTabs from "./components/HostTabs";
import CredentialTab from "./components/CredentialTab";
import { HostTaskTab } from "./task-tab";
import HostBreadcrumbs from "./components/HostBreadcrumbs";
import { BeaconTab } from "./beacon-tab";

const HostDetails = () => {
    return (
        <HostContextProvider>
            <HostBreadcrumbs />
            <HostDetailsSection />
            <div className="flex flex-col mt-2">
                <Tabs.Root defaultValue="beacons">
                    <HostTabs />
                    <Tabs.Content value="beacons">
                        <BeaconTab />
                    </Tabs.Content>
                    <Tabs.Content value="tasks">
                        <HostTaskTab />
                    </Tabs.Content>
                    <Tabs.Content value="credentials">
                        <CredentialTab />
                    </Tabs.Content>
                </Tabs.Root>
            </div>
        </HostContextProvider>
    );
};
export default HostDetails;
