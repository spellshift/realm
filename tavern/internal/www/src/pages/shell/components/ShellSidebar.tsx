import React, { useContext, useMemo } from "react";
import { Link } from "react-router-dom";
import { Avatar, Tooltip } from "@chakra-ui/react";
import {
  BookOpenIcon,
  BugAntIcon,
  DocumentDuplicateIcon,
  PresentationChartBarIcon,
  UserGroupIcon,
  WrenchScrewdriverIcon,
} from "@heroicons/react/24/outline";
import { FileTerminal, LibraryBigIcon } from "lucide-react";
import logo from "../../../assets/eldrich.png";
import { AuthorizationContext } from "../../../context/AuthorizationContext";
import { PageNavItem } from "../../../utils/enums";
import { classNames } from "../../../utils/utils";

export interface ShellSidebarProps {
  isOpen: boolean;
  currNavItem?: string;
}

interface ShellNavigationItem {
  name: string;
  href: string;
  icon: React.ElementType;
  target?: string;
  internal: boolean;
  adminOnly?: boolean;
}

const ShellSidebar: React.FC<ShellSidebarProps> = ({ isOpen, currNavItem }) => {
  const authContext = useContext(AuthorizationContext);
  const user = authContext?.data?.me;

  const navigation = useMemo<ShellNavigationItem[]>(
    () => [
      {
        name: PageNavItem.dashboard,
        href: "/dashboard",
        icon: PresentationChartBarIcon,
        internal: true,
      },
      {
        name: PageNavItem.hosts,
        href: "/hosts",
        icon: BugAntIcon,
        internal: true,
      },
      {
        name: PageNavItem.quests,
        href: "/quests",
        icon: FileTerminal,
        internal: true,
      },
      {
        name: PageNavItem.tomes,
        href: "/tomes",
        icon: BookOpenIcon,
        internal: true,
      },
      {
        name: PageNavItem.assets,
        href: "/assets",
        icon: LibraryBigIcon,
        internal: true,
      },
      ...(user?.isAdmin
        ? [
            {
              name: PageNavItem.admin,
              href: "/admin",
              icon: UserGroupIcon,
              internal: true,
              adminOnly: true,
            },
          ]
        : []),
      {
        name: PageNavItem.documentation,
        href: "https://docs.realm.pub/",
        icon: DocumentDuplicateIcon,
        target: "__blank",
        internal: false,
      },
      {
        name: PageNavItem.playground,
        href: "/playground",
        icon: WrenchScrewdriverIcon,
        target: "__blank",
        internal: false,
      },
    ],
    [user?.isAdmin]
  );

  if (!isOpen) {
    return null;
  }

  return (
    <aside
      data-testid="shell-sidebar"
      aria-label="Navigation sidebar"
      className="w-12 shrink-0 bg-gray-900 border-r border-gray-800 flex flex-col justify-between items-center py-3 select-none z-20"
    >
      <nav className="flex flex-1 flex-col items-center w-full overflow-y-auto no-scrollbar">
        <ul className="flex flex-col items-center space-y-1.5 w-full px-1.5">
          {navigation.map((item) => {
            const itemClasses = classNames(
              item.name === currNavItem
                ? "bg-gray-800 text-white"
                : "text-gray-400 hover:text-white hover:bg-gray-800",
              "group flex items-center justify-center rounded-md p-2 text-sm font-semibold transition-colors"
            );

            return (
              <li key={item.name}>
                <Tooltip
                  label={item.name}
                  placement="right"
                  bg="#111827"
                  color="white"
                  borderRadius="md"
                  border="1px solid #374151"
                  hasArrow
                >
                  {item.internal ? (
                    <Link aria-label={item.name} to={item.href} className={itemClasses}>
                      <item.icon className="h-5 w-5 shrink-0" aria-hidden="true" />
                    </Link>
                  ) : (
                    <a
                      aria-label={item.name}
                      href={item.href}
                      target={item.target ? "__blank" : undefined}
                      rel={item.target ? "noopener noreferrer" : undefined}
                      className={itemClasses}
                    >
                      <item.icon className="h-5 w-5 shrink-0" aria-hidden="true" />
                    </a>
                  )}
                </Tooltip>
              </li>
            );
          })}
        </ul>
      </nav>

      <div className="mt-auto pt-3 flex flex-col items-center gap-3 w-full px-1.5">
        <Tooltip
          label={user?.name ? `Profile (${user.name})` : "Profile"}
          placement="right"
          bg="#111827"
          color="white"
          borderRadius="md"
          border="1px solid #374151"
          hasArrow
        >
          <Link
            to="/profile"
            aria-label="Profile"
            className={classNames(
              currNavItem === "Profile"
                ? "bg-gray-800 text-white"
                : "text-gray-400 hover:bg-gray-800 hover:text-white",
              "group flex items-center justify-center rounded-md p-1.5 text-sm font-semibold transition-colors"
            )}
          >
            <Avatar size="xs" name={user?.name || ""} src={user?.photoURL || undefined} />
          </Link>
        </Tooltip>

        <Tooltip
          label="Realm"
          placement="right"
          bg="#111827"
          color="white"
          borderRadius="md"
          border="1px solid #374151"
          hasArrow
        >
          <a
            href="/"
            aria-label="Realm"
            className="flex items-center justify-center p-1 rounded-md hover:bg-gray-800 transition-colors"
          >
            <img className="h-6 w-auto" src={logo} alt="Realm" width="24" height="24" />
          </a>
        </Tooltip>
      </div>
    </aside>
  );
};

export default ShellSidebar;
