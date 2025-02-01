import { useContext, useState } from 'react'
import {
  DocumentDuplicateIcon,
  CommandLineIcon,
  WrenchScrewdriverIcon,
  BugAntIcon,
  PresentationChartBarIcon,
  BookOpenIcon,
  ClipboardDocumentListIcon,
} from '@heroicons/react/24/outline'

import { PageNavItem } from '../../utils/enums';
import { AccessGate } from '../access-gate';
import FullSidebarNav from './FullSidebarNav';
import MobileNav from './MobileNav';
import MinimizedSidebarNav from './MinimizedSidebarNav';
import { UserPreferencesContext } from '../../context/UserPreferences';

const navigation = [
  { name: PageNavItem.createQuest, href: '/createQuest', icon: CommandLineIcon, internal: true },
  { name: PageNavItem.dashboard, href: '/dashboard', icon: PresentationChartBarIcon, internal: true },
  { name: PageNavItem.hosts, href: '/hosts', icon: BugAntIcon, internal: true },
  { name: PageNavItem.quests, href: '/quests', icon: ClipboardDocumentListIcon, internal: true },
  { name: PageNavItem.tomes, href: '/tomes', icon: BookOpenIcon, internal: true },
  { name: PageNavItem.documentation, href: 'https://docs.realm.pub/', icon: DocumentDuplicateIcon, target: "__blank", internal: false },
  { name: PageNavItem.playground, href: '/playground', icon: WrenchScrewdriverIcon, target: "__blank", internal: false },
]

function classNames(...classes: string[]) {
  return classes.filter(Boolean).join(' ')
}

type Props = {
  children: any;
  currNavItem?: PageNavItem;
}
export const PageWrapper = (props: Props) => {
  const { children, currNavItem } = props;
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const { sidebarMinimized, setSidebarMinimized } = useContext(UserPreferencesContext);

  return (
    <AccessGate>
      <div>
        {sidebarMinimized ?
          <MinimizedSidebarNav currNavItem={currNavItem} navigation={navigation} handleSidebarMinimized={setSidebarMinimized} />
          :
          <FullSidebarNav currNavItem={currNavItem} navigation={navigation} handleSidebarMinimized={setSidebarMinimized} />
        }
        <MobileNav currNavItem={currNavItem} navigation={navigation} sidebarOpen={sidebarOpen} handleSidebarOpen={setSidebarOpen} />

        <main className={classNames("py-10", sidebarMinimized ? "lg:ml-24" : "lg:ml-72")}>
          <div className="px-4 sm:px-6 xl:px-8">{children}</div>
        </main>
      </div>
    </AccessGate>
  )
}
