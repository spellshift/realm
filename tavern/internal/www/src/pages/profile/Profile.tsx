import React from 'react';
import { useQuery, gql } from '@apollo/client';
import { Ring } from '@uiball/loaders';
import PageHeader from '../../components/tavern-base-ui/PageHeader';
import AccountSettings from './AccountSettings';
import DeviceAuth from './DeviceAuth';

// Extend UserNode for local query
export interface DeviceAuthNode {
    id: string;
    userCode: string;
    createdAt: string;
    status: string;
}

export interface ProfileUserNode {
    id: string;
    name: string;
    photoURL?: string;
    isActivated: boolean;
    isAdmin: boolean;
    apiKey: string;
    deviceAuths?: {
        edges: { node: DeviceAuthNode }[];
    };
}

const GET_PROFILE = gql`
    query GetProfile {
        me {
            id
            name
            photoURL
            isActivated
            isAdmin
            apiKey
            deviceAuths {
                edges {
                    node {
                        id
                        userCode
                        createdAt
                        status
                    }
                }
            }
        }
    }
`;

const Profile = () => {
    const { data, loading, refetch } = useQuery<{ me: ProfileUserNode }>(GET_PROFILE, {
        fetchPolicy: 'network-only' // Ensure we get fresh data
    });

    if (loading) return (
        <div className="flex items-center justify-center p-8">
            <Ring size={40} lineWeight={5} speed={2} color="#4B5563" />
        </div>
    );

    const user = data?.me;
    const devices = user?.deviceAuths?.edges || [];

    return (
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-8">
            <PageHeader title="Profile" description="Manage your account settings, devices, and API key." />

            <div className="mt-8 grid grid-cols-1 gap-8 md:grid-cols-2">
                <AccountSettings user={user} refetch={refetch} />
                <DeviceAuth devices={devices} refetch={refetch} />
            </div>
        </div>
    );
};

export default Profile;
