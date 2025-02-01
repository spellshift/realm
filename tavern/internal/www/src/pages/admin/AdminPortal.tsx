import { PageWrapper } from "../../components/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import { Tab } from '@headlessui/react'
import { UserTableWrapper } from "./components/UserTableWrapper";

export const AdminPortal = () => {
    return (
        <PageWrapper currNavItem={PageNavItem.admin} adminOnly>
	    <div className="border-b border-gray-200 pb-5 flex flex-col sm:flex-row  sm:items-center sm:justify-between gap-4">
	    	<div className="flex-1 flex flex-col gap-2">
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">Admin Portal</h3>
                    <div className="max-w-2xl text-sm">
                        <span>This portal is for managing users allowed to access Tavern. You can also promote users to Administrator, who will be able to access this page.</span>
                    </div>
	        </div>
            </div>
	    <div>
	        <UserTableWrapper />
	    </div>
        </PageWrapper>
    );
}
