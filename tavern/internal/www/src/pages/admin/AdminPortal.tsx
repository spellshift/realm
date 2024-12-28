import { PageWrapper } from "../../components/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import { Tab } from '@headlessui/react'
import { UserTableWrapper } from "./components/UserTableWrapper";

export const AdminPortal = () => {
    return (
        <PageWrapper currNavItem={PageNavItem.admin}>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Admin Portal</h3>
                <div className="max-w-2xl text-sm">
                    <span>This portal is for managing users allowed to access Tavern. You can also promote users to Administrator, who will be able to access this page.</span>
                </div>
            </div>
            <Tab.Group>
                <Tab.List className="flex flex-row space-x-4 border-b border-gray-200 w-full">
                    <Tab className={({ selected }) => `border-b-2 py-2 px-4 text-sm font-semibold ${selected ? 'border-purple-700 text-purple-800' : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'}`}>Users</Tab>
                </Tab.List>
                <Tab.Panels>
                    <Tab.Panel>
                        <UserTableWrapper />
                    </Tab.Panel>
                </Tab.Panels>
            </Tab.Group>
        </PageWrapper>
    );
}
