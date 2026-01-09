import { Link } from 'react-router-dom';
import { classNames } from '../../utils/utils';
import { ArrowRightOnRectangleIcon } from '@heroicons/react/24/outline';
import logo from '../../assets/eldrich.png';
import { usePageNavigation } from './usePageNavigation';
import { PollingCountdown } from '../../context/PollingContext';

type MinimizedSidebarNavProps = {
    currNavItem?: string;
    handleSidebarMinimized: (arg: boolean) => void
}

const MinimizedSidebarNav = ({ currNavItem, handleSidebarMinimized }: MinimizedSidebarNavProps) => {
    const navigation = usePageNavigation();

    return (
        <div className="hidden lg:fixed lg:inset-y-0 lg:z-50 lg:flex lg:w-24 lg:flex-col justify-between items-center bg-gray-900">
            {/* Sidebar component */}
            <div className="flex grow flex-col gap-y-5 overflow-y-auto px-6">
                <div className='flex flex-col h-28 justify-center'>
                    <button className='p-2 text-gray-500 -mr-4 hover:text-gray-300'>
                        <ArrowRightOnRectangleIcon className="h-6 w-6" onClick={() => handleSidebarMinimized(false)} />
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
                                                aria-label={item.name}
                                                to={item.href}
                                                className={classNames(
                                                    item.name === currNavItem
                                                        ? 'bg-gray-800 text-white'
                                                        : 'text-gray-400 hover:text-white hover:bg-gray-800',
                                                    'group flex gap-x-3 rounded-md py-2 px-6 text-sm leading-6 font-semibold'
                                                )}
                                            >
                                                <item.icon className="h-6 w-6 shrink-0" aria-hidden="true" />
                                            </Link>
                                        ) : (
                                            <a
                                                aria-label={item.name}
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
                                            </a>
                                        )}
                                    </li>
                                ))}
                            </ul>
                        </li>
                    </ul>
                </nav>
            </div>
            <PollingCountdown variant="minimal" />
            <div className="my-8">
                <a href='/'>
                    <img
                        className="h-8 w-auto"
                        src={logo}
                        alt="Realm"
                        width="32"
                        height="32"
                    />
                </a>
            </div>
        </div>
    );
}
export default MinimizedSidebarNav;
