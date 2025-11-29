import {
    DocumentDuplicateIcon,
    CommandLineIcon,
    WrenchScrewdriverIcon,
    BugAntIcon,
    PresentationChartBarIcon,
    BookOpenIcon,
    ClipboardDocumentListIcon,
    UserGroupIcon,
} from '@heroicons/react/24/outline';
import { useContext } from 'react';
import { AuthorizationContext } from '../../context/AuthorizationContext';
import { NavigationItemType } from '../../utils/consts';
import { PageNavItem } from '../../utils/enums';

export const usePageNavigation = () => {
    const { data } = useContext(AuthorizationContext);

    const navigation = [
        { name: PageNavItem.createQuest, href: '/createQuest', icon: CommandLineIcon, internal: true },
        { name: PageNavItem.dashboard, href: '/dashboard', icon: PresentationChartBarIcon, internal: true },
        { name: PageNavItem.hosts, href: '/hosts', icon: BugAntIcon, internal: true },
        { name: PageNavItem.quests, href: '/quests', icon: ClipboardDocumentListIcon, internal: true },
        { name: PageNavItem.tomes, href: '/tomes', icon: BookOpenIcon, internal: true },
        ...data?.me?.isAdmin ? [{ name: PageNavItem.admin, href: '/admin', icon: UserGroupIcon, internal: true, adminOnly: true }] : [],
        { name: PageNavItem.documentation, href: 'https://docs.realm.pub/', icon: DocumentDuplicateIcon, target: "__blank", internal: false },
        { name: PageNavItem.playground, href: '/playground', icon: WrenchScrewdriverIcon, target: "__blank", internal: false },
    ] as Array<NavigationItemType>;

    return navigation;
}
