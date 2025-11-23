import React from "react";

import { PageNavItem } from "../../utils/enums";
import { Tab } from "@headlessui/react";
import { HostContextProvider } from "../../context/HostContext";
import HostDetailsSection from "./components/HostDetailsSection";
import { PageWrapper } from "../../features/page-wrapper";
import HostTabs from "./components/HostTabs";
import BeaconTab from "./components/BeaconTab";
import CredentialTab from "./components/CredentialTab";
import HostTaskTab from "./components/HostTaskTab";
import HostBreadcrumbs from "./components/HostBreadcrumbs";

const HostDetails = () => {
    return (
        <HostContextProvider>
            <PageWrapper currNavItem={PageNavItem.hosts}>
                <HostBreadcrumbs />
                <HostDetailsSection />
                <div className="flex flex-col gap-4 mt-2">
                    <Tab.Group>
                        <HostTabs />
                        <Tab.Panels>
                            <Tab.Panel>
                                <BeaconTab />
                            </Tab.Panel>
                            <Tab.Panel>
                                <HostTaskTab />
                            </Tab.Panel>
                            <Tab.Panel>
                                <CredentialTab />
                            </Tab.Panel>
                        </Tab.Panels>
                    </Tab.Group>
                </div>
            </PageWrapper>
        </HostContextProvider>
    );
};
export default HostDetails;
