import { FunctionComponent, useContext, useState } from 'react'
import { PageNavItem } from '../../utils/enums';
import { AccessGate } from '../../components/access-gate';
import FullSidebarNav from './FullSidebarNav';
import MobileNav from './MobileNav';
import MinimizedSidebarNav from './MinimizedSidebarNav';
import { UserPreferencesContext } from '../../context/UserPreferences';
import { classNames } from '../../utils/utils';

type Props = {
  children: any;
  currNavItem?: PageNavItem;
}

export const PageWrapper: FunctionComponent<Props> = ({ children, currNavItem }) => {
  const [sidebarOpen, setSidebarOpen] = useState(false)
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
          <div className="px-4 sm:px-6 xl:px-8 flex flex-col gap-4">{children}</div>
        </main>
      </div>
    </AccessGate>
  )
}
