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
import { ShellTab } from "./shell-tab";
import { useCreateQuestModal } from "../../context/CreateQuestModalContext";
import Button from "../../components/tavern-base-ui/button/Button";

const TAB_NAMES = ["beacons", "tasks", "processes", "files", "credentials", "shells"] as const;

const HostDetails = () => {
    const [searchParams, setSearchParams] = useSearchParams();
    const { openModal } = useCreateQuestModal();

    const tabParam = searchParams.get("tab");
    const selectedIndex = Math.max(0, TAB_NAMES.indexOf(tabParam as typeof TAB_NAMES[number]));

    const handleTabChange = (index: number) => {
        setSearchParams({ tab: TAB_NAMES[index] }, { replace: true });
    };

    const handleOpenCreateQuest = () => {
        openModal();
    };

    return (
        <HostContextProvider>
            <div className="flex flex-row justify-between w-full items-center">
                <HostBreadcrumbs />
                <div>
                    <Button
                        buttonStyle={{ color: "purple", size: "md" }}
                        onClick={() => openModal()}
                    >
                        Create quest
                    </Button>
                </div>
            </div>
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
                            <ProcessTab handleOpenCreateQuest={handleOpenCreateQuest} />
                        </TabPanel>
                        <TabPanel>
                            <FilesTab />
                        </TabPanel>
                        <TabPanel>
                            <CredentialTab />
                        </TabPanel>
                        <TabPanel>
                            <ShellTab />
                        </TabPanel>
                    </TabPanels>
                </TabGroup>
            </div>
        </HostContextProvider>
    );
};
export default HostDetails;
