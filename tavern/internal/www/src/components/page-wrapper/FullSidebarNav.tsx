import { Link } from 'react-router-dom';
import { classNames } from '../../utils/utils';
import logo from '../../assets/eldrich.png';
import { ArrowLeftOnRectangleIcon } from '@heroicons/react/24/outline';
import { usePageNavigation } from './usePageNavigation';
import { PollingCountdown } from './PollingCountdown';

type FullSidebarNavProps = {
    currNavItem?: string;
    handleSidebarMinimized: (arg: boolean) => void;
}

const FullSidebarNav = ({ currNavItem, handleSidebarMinimized }: FullSidebarNavProps) => {
    const navigation = usePageNavigation();

    return (
        <div className="hidden lg:fixed lg:inset-y-0 lg:z-50 lg:flex lg:w-72 lg:flex-col">
            {/* Sidebar component */}
            <div className="flex grow flex-col gap-y-5 overflow-y-auto bg-gray-900 px-6">
                <div className=' flex flex-row justify-between'>
                    <a href='/'>
                        <div className="flex h-28 shrink-0 items-center gap-2">
                            <img
                                className="h-10 w-auto"
                                src={logo}
                                alt="Realm"
                            />
                            <div className="text-white text-3xl leading-6 font-semibold tracking-wide">Realm</div>
                        </div>
                    </a>
                    <button className='p-2 text-gray-500 -mr-4 hover:text-gray-300' onClick={() => handleSidebarMinimized(true)}>
                        <ArrowLeftOnRectangleIcon className="h-6 w-6" />
                    </button>
                </div>
                <nav className="flex flex-1 flex-col">
                    <ul className="flex flex-1 flex-col gap-y-7">
                        <li>
                            <ul className="-mx-4 space-y-1">
                                {navigation.map((item) => (
                                    <li key={item.name}>
                                        {item.internal ? (
                                            <Link
                                                to={item.href}
                                                className={classNames(
                                                    item.name === currNavItem
                                                        ? 'bg-gray-800 text-white'
                                                        : 'text-gray-400 hover:text-white hover:bg-gray-800',
                                                    'group flex gap-x-3 rounded-md py-2 px-6 text-sm leading-6 font-semibold'
                                                )}
                                            >
                                                <item.icon className="h-6 w-6 shrink-0" aria-hidden="true" />
                                                {item.name}
                                            </Link>
                                        ) : (
                                            <a
                                                href={item.href}
                                                target={item?.target ? '__blank' : undefined}
                                                className={classNames(
                                                    item.name === currNavItem
                                                        ? 'bg-gray-800 text-white'
                                                        : 'text-gray-400 hover:text-white hover:bg-gray-800',
                                                    'group flex gap-x-3 rounded-md py-2 px-6 text-sm leading-6 font-semibold'
                                                )}
                                            >
                                                <item.icon className="h-6 w-6 shrink-0" aria-hidden="true" />
                                                {item.name}
                                            </a>
                                        )}
                                    </li>
                                ))}
                            </ul>
                        </li>
                    </ul>
                </nav>
                <PollingCountdown variant="full" />
            </div>
        </div>
    );
}
export default FullSidebarNav;
