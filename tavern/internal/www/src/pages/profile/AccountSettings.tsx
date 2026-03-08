import React from 'react';
import { useMutation, gql } from '@apollo/client';
import { useToast } from '@chakra-ui/react';
import { RefreshCcw, LogOut, Copy } from 'lucide-react';
import Button from '../../components/tavern-base-ui/button/Button';
import { ProfileUserNode } from './Profile';

const RESET_API_KEY = gql`
    mutation ResetUserAPIKey {
        resetUserAPIKey {
            id
            apiKey
        }
    }
`;

interface AccountSettingsProps {
    user: ProfileUserNode | undefined;
    refetch: () => Promise<any>;
}

const AccountSettings: React.FC<AccountSettingsProps> = ({ user, refetch }) => {
    const toast = useToast();

    const [resetAPIKey, { loading: resetting }] = useMutation(RESET_API_KEY, {
        onCompleted: async () => {
            // Await refetch to ensure the UI updates with the new API key
            await refetch();
            toast({ title: 'API Key Regenerated', status: 'success', duration: 3000 });
        },
        onError: (err) => {
            toast({ title: 'Failed to regenerate API Key', description: err.message, status: 'error', duration: 3000 });
        }
    });

    const handleSignout = async () => {
        try {
            const res = await fetch('/api/auth/signout', { method: 'POST' });
            if (res.ok) {
                window.location.href = '/';
            } else {
                toast({ title: 'Failed to signout', status: 'error' });
            }
        } catch (err) {
            console.error(err);
        }
    };

    const copyApiKey = () => {
        if (user?.apiKey) {
            navigator.clipboard.writeText(user.apiKey);
            toast({ title: 'Copied to clipboard', status: 'info', duration: 2000 });
        }
    };

    return (
        <div className="bg-white p-6 shadow-sm rounded-md border border-gray-200">
            <div className="flex items-center mb-4">
                {user?.photoURL ? (
                    <img src={user.photoURL} alt={user.name} className="h-16 w-16 rounded-full mr-4" />
                ) : (
                    <div className="h-16 w-16 bg-gray-200 rounded-full mr-4 flex items-center justify-center text-gray-500 font-bold text-xl">
                        {user?.name?.charAt(0) || 'U'}
                    </div>
                )}
                <div>
                    <p className="text-xl font-bold">{user?.name}</p>
                    <p className="text-gray-500">ID: {user?.id}</p>
                </div>
            </div>
            <hr className="my-4 border-gray-200" />

            <p className="text-lg font-bold mb-2">API Key</p>
            <p className="text-sm text-gray-600 mb-4">Use this key to authenticate Tavern CLI and API requests.</p>
            <div className="flex items-center">
                <code className="p-2 bg-gray-100 rounded-md w-64 mr-4 overflow-hidden text-ellipsis whitespace-nowrap">
                    {user?.apiKey ? "••••••••••••••••••••••••••••••••" : "No API Key"}
                </code>
                <div className="flex flex-row gap-2">
                    <Button onClick={copyApiKey} leftIcon={<Copy size={16} />}>Copy</Button>
                    <Button onClick={() => resetAPIKey()} isLoading={resetting} buttonVariant="outline" buttonStyle={{ color: 'red', size: 'md' }} leftIcon={<RefreshCcw size={16} />}>
                        Reset
                    </Button>
                </div>
            </div>

            <hr className="my-6 border-gray-200" />

            <Button onClick={handleSignout} leftIcon={<LogOut size={16} />} buttonStyle={{ color: 'gray', size: 'md' }}>
                Sign out
            </Button>
        </div>
    );
};

export default AccountSettings;
