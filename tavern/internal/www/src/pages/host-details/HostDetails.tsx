import { TabGroup, TabPanel, TabPanels } from "@headlessui/react";
import { useSearchParams } from "react-router-dom";
import { HostContextProvider } from "../../context/HostContext";
import HostDetailsSection from "./components/HostDetailsSection";
import HostTabs from "./components/HostTabs";
import CredentialTab from "./components/CredentialTab";
import { HostTaskTab } from "./task-tab";
import HostBreadcrumbs from "./components/HostBreadcrumbs";
import { BeaconTab } from "./beacon-tab";
import { ProcessTab } from "./process-tab";
import { FilesTab } from "./files-tab";

const TAB_NAMES = ["beacons", "tasks", "processes", "files", "credentials"] as const;

const HostDetails = () => {
    const [searchParams, setSearchParams] = useSearchParams();

    const tabParam = searchParams.get("tab");
    const selectedIndex = Math.max(0, TAB_NAMES.indexOf(tabParam as typeof TAB_NAMES[number]));

    const handleTabChange = (index: number) => {
        setSearchParams({ tab: TAB_NAMES[index] }, { replace: true });
    };

    return (
        <HostContextProvider>
            <HostBreadcrumbs />
            <HostDetailsSection />
            <div className="flex flex-col mt-2">
                <TabGroup selectedIndex={selectedIndex} onChange={handleTabChange}>
                    <HostTabs />
                    <TabPanels>
                        <TabPanel>
                            <BeaconTab />
                        </TabPanel>
                        <TabPanel>
                            <HostTaskTab />
                        </TabPanel>
                        <TabPanel>
                            <ProcessTab />
                        </TabPanel>
                        <TabPanel>
                            <FilesTab />
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
