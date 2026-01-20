import { useContext, useState } from 'react'
import { AccessGate } from '../access-gate';
import FullSidebarNav from './FullSidebarNav';
import MobileNav from './MobileNav';
import MinimizedSidebarNav from './MinimizedSidebarNav';
import { UserPreferencesContext } from '../../context/UserPreferences';
import { classNames, getNavItemFromPath } from '../../utils/utils';
import { Outlet, useLocation } from 'react-router-dom'
import { FilterProvider } from '../../context/FilterContext';
import { SortsProvider } from '../../context/SortContext';

export const PageWrapper = () => {
  const { pathname } = useLocation();
  const currNavItem = getNavItemFromPath(pathname);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const { sidebarMinimized, setSidebarMinimized } = useContext(UserPreferencesContext);

  return (
    <AccessGate>
      <div>
        {sidebarMinimized ?
          <MinimizedSidebarNav currNavItem={currNavItem} handleSidebarMinimized={setSidebarMinimized} />
          :
          <FullSidebarNav currNavItem={currNavItem} handleSidebarMinimized={setSidebarMinimized} />
        }
        <MobileNav currNavItem={currNavItem} sidebarOpen={sidebarOpen} handleSidebarOpen={setSidebarOpen} />

        <main className={classNames("py-4", sidebarMinimized ? "lg:ml-24" : "lg:ml-72")}>
          <div className="px-4 sm:px-6 xl:px-8 flex flex-col gap-4">
            <FilterProvider>
              <SortsProvider>
                <Outlet />
              </SortsProvider>
            </FilterProvider>
          </div>
        </main>
      </div>
    </AccessGate>
  )
}
