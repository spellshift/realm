import React, { useState, useMemo } from 'react';
import { useToast, Input } from '@chakra-ui/react';
import { createColumnHelper } from '@tanstack/react-table';
import Button from '../../components/tavern-base-ui/button/Button';
import Table from '../../components/tavern-base-ui/table/Table';
import { EmptyState, EmptyStateType } from '../../components/tavern-base-ui/EmptyState';
import { DeviceAuthNode } from './Profile';
import { Trash2 } from 'lucide-react';

interface DeviceAuthProps {
    devices: { node: DeviceAuthNode }[];
    refetch: () => Promise<any>;
}

const DeviceAuth: React.FC<DeviceAuthProps> = ({ devices, refetch }) => {
    const toast = useToast();
    const [userCode, setUserCode] = useState('');
    const [approving, setApproving] = useState(false);

    const handleApproveDevice = async () => {
        if (!userCode.trim()) return;
        setApproving(true);
        try {
            const res = await fetch('/auth/rda/approve', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ user_code: userCode.trim() })
            });

            if (res.ok) {
                toast({ title: 'Device Approved', status: 'success', duration: 3000 });
                setUserCode('');
                // Ensure refetch completes before hiding the loading state to prevent UI flicker
                await refetch();
            } else {
                toast({ title: 'Failed to approve device', description: await res.text(), status: 'error', duration: 3000 });
            }
        } catch (err: any) {
            toast({ title: 'Error', description: err.message, status: 'error', duration: 3000 });
        } finally {
            setApproving(false);
        }
    };

    const handleRevokeDevice = async (userCodeToRevoke: string) => {
        try {
            const res = await fetch('/auth/rda/revoke', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ user_code: userCodeToRevoke })
            });
            if (res.ok) {
                toast({ title: 'Device Revoked', status: 'success', duration: 3000 });
                // Ensure refetch completes
                await refetch();
            } else {
                toast({ title: 'Failed to revoke', description: await res.text(), status: 'error', duration: 3000 });
            }
        } catch (err: any) {
            toast({ title: 'Error', description: err.message, status: 'error', duration: 3000 });
        }
    };

    const columnHelper = createColumnHelper<{ node: DeviceAuthNode }>();
    const columns = useMemo(() => [
        columnHelper.accessor('node.userCode', {
            header: 'Device Code',
            cell: info => info.getValue(),
        }),
        columnHelper.accessor('node.status', {
            header: 'Status',
            cell: info => info.getValue(),
        }),
        columnHelper.display({
            id: 'actions',
            header: '',
            cell: props => {
                const node = props.row.original.node;
                if (node.status === 'PENDING' || node.status === 'APPROVED') {
                    return (
                        <div className="flex justify-end">
                            <Button buttonVariant="outline" buttonStyle={{ color: 'red', size: 'xs' }} onClick={() => handleRevokeDevice(node.id)}>
                                <Trash2 size={16} />
                            </Button>
                        </div>
                    );
                }
                return null;
            }
        })
    ], [columnHelper]);

    return (
        <div className="bg-white p-6 shadow-sm rounded-md border border-gray-200">
            <p className="text-lg font-bold mb-2">Device Authentication</p>
            <p className="text-sm text-gray-600 mb-4">Enter the code displayed on your device to approve its sign-in request.</p>
            <div className="flex mb-6">
                <Input
                    type="text"
                    placeholder="Device Code (e.g. 5f4dcc3b)"
                    value={userCode}
                    onChange={(e) => setUserCode(e.target.value)}
                    className="mr-4 flex-1 font-mono"
                />
                <Button onClick={handleApproveDevice} isLoading={approving} buttonStyle={{ color: 'purple', size: 'md' }}>
                    Approve
                </Button>
            </div>

            <hr className="my-4 border-gray-200" />
            <p className="text-md font-bold mb-2">Approved Devices</p>
            <div className="mt-4">
                {devices.length === 0 ? (
                    <EmptyState label="No devices" type={EmptyStateType.noData} />
                ) : (
                    <Table data={devices} columns={columns as any} />
                )}
            </div>
        </div>
    );
};

export default DeviceAuth;
