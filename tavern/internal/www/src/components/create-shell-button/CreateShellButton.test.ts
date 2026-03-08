import { selectBestBeaconId, BeaconCandidate } from './CreateShellButton';
import { PrincipalAdminTypes, SupportedTransports } from '../../utils/enums';
import { describe, it, expect } from 'vitest';

describe('selectBestBeaconId', () => {
    it('should return null for empty list', () => {
        expect(selectBestBeaconId([])).toBeNull();
    });

    it('should prioritize high priority principal over low priority', () => {
        const candidates: BeaconCandidate[] = [
            {
                id: '1',
                principal: 'user',
                interval: 10,
                lastSeenAt: new Date().toISOString(),
                transport: SupportedTransports.GRPC
            },
            {
                id: '2',
                principal: PrincipalAdminTypes.root,
                interval: 100, // worse interval
                lastSeenAt: new Date().toISOString(),
                transport: SupportedTransports.HTTP1
            }
        ];
        // id 2 has root (high priority), id 1 has user (low).
        // Principal priority is #1 check.
        expect(selectBestBeaconId(candidates)).toBe('2');
    });

    it('should prioritize gRPC over other transports given equal principal priority', () => {
        const candidates: BeaconCandidate[] = [
            {
                id: '1',
                principal: 'root',
                interval: 10,
                lastSeenAt: new Date().toISOString(),
                transport: SupportedTransports.HTTP1
            },
            {
                id: '2',
                principal: 'root',
                interval: 10,
                lastSeenAt: new Date().toISOString(),
                transport: SupportedTransports.GRPC
            }
        ];
        // Both root. id 2 is GRPC.
        expect(selectBestBeaconId(candidates)).toBe('2');
    });

    it('should prioritize shortest interval given equal principal and transport', () => {
        const candidates: BeaconCandidate[] = [
            {
                id: '1',
                principal: 'root',
                interval: 20,
                lastSeenAt: new Date().toISOString(),
                transport: SupportedTransports.GRPC
            },
            {
                id: '2',
                principal: 'root',
                interval: 10,
                lastSeenAt: new Date().toISOString(),
                transport: SupportedTransports.GRPC
            }
        ];
        // Both root, both GRPC. id 2 has shorter interval.
        expect(selectBestBeaconId(candidates)).toBe('2');
    });

    it('should prioritize soonest next check-in given everything else equal', () => {
        const now = new Date();
        const future1 = new Date(now.getTime() + 10000).toISOString();
        const future2 = new Date(now.getTime() + 5000).toISOString(); // Sooner

        const candidates: BeaconCandidate[] = [
            {
                id: '1',
                principal: 'root',
                interval: 10,
                lastSeenAt: now.toISOString(),
                nextSeenAt: future1,
                transport: SupportedTransports.GRPC
            },
            {
                id: '2',
                principal: 'root',
                interval: 10,
                lastSeenAt: now.toISOString(),
                nextSeenAt: future2,
                transport: SupportedTransports.GRPC
            }
        ];
        expect(selectBestBeaconId(candidates)).toBe('2');
    });

    it('should prioritize gRPC even with worse interval', () => {
         const candidates: BeaconCandidate[] = [
            {
                id: '1',
                principal: 'root',
                interval: 5, // Better interval
                lastSeenAt: new Date().toISOString(),
                transport: SupportedTransports.HTTP1
            },
            {
                id: '2',
                principal: 'root',
                interval: 60, // Worse interval
                lastSeenAt: new Date().toISOString(),
                transport: SupportedTransports.GRPC
            }
        ];
        // 1. Principal equal (root)
        // 2. Transport: 2 is GRPC, 1 is HTTP1. 2 wins.
        expect(selectBestBeaconId(candidates)).toBe('2');
    });

    it('should prioritize high priority principal over gRPC', () => {
         const candidates: BeaconCandidate[] = [
            {
                id: '1',
                principal: 'user', // Low priority
                interval: 5,
                lastSeenAt: new Date().toISOString(),
                transport: SupportedTransports.GRPC // Has GRPC
            },
            {
                id: '2',
                principal: PrincipalAdminTypes.root, // High priority
                interval: 60,
                lastSeenAt: new Date().toISOString(),
                transport: SupportedTransports.HTTP1 // No GRPC
            }
        ];
        // Principal is higher priority than transport.
        expect(selectBestBeaconId(candidates)).toBe('2');
    });
});
