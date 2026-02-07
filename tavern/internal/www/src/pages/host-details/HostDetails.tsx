import { TabGroup, TabPanel, TabPanels } from "@headlessui/react";
import { HostContextProvider } from "../../context/HostContext";
import HostDetailsSection from "./components/HostDetailsSection";
import HostTabs from "./components/HostTabs";
import BeaconTab from "./components/BeaconTab";
import CredentialTab from "./components/CredentialTab";
import HostTaskTab from "./components/HostTaskTab";
import HostBreadcrumbs from "./components/HostBreadcrumbs";

const HostDetails = () => {
    return (
        <HostContextProvider>
            <HostBreadcrumbs />
            <HostDetailsSection />
            <div className="flex flex-col mt-2">
                <TabGroup>
                    <HostTabs />
                    <TabPanels>
                        <TabPanel>
                            <BeaconTab />
                        </TabPanel>
                        <TabPanel>
                            <HostTaskTab />
                        </TabPanel>
                        <TabPanel>
                            <CredentialTab />
                        </TabPanel>
                    </TabPanels>
                </TabGroup>
            </div>
        </HostContextProvider>
    );
};
export default HostDetails;
