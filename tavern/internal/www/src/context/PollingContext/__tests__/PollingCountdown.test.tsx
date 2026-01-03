import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { PollingCountdown } from '../PollingCountdown';
import { PollingProvider } from '../PollingContext';

vi.mock('lucide-react', () => ({
    RotateCw: (props: any) => <svg data-testid="rotate-icon" {...props} />,
}));

const mockRefetchQueries = vi.fn();
vi.mock('@apollo/client', () => ({
    useApolloClient: () => ({
        refetchQueries: mockRefetchQueries,
    }),
}));

function TestWrapper({ variant }: { variant: 'full' | 'minimal' | 'icon-only' }) {
    return (
        <PollingProvider>
            <PollingCountdown variant={variant} />
        </PollingProvider>
    );
}

describe('PollingCountdown', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('Rendering variants', () => {
        it('should render icon-only variant correctly', () => {
            render(<TestWrapper variant="icon-only" />);

            const icon = screen.getByTestId('rotate-icon');
            expect(icon).toBeInTheDocument();

            // Should not display any text
            expect(screen.queryByText(/\d+s/)).not.toBeInTheDocument();
            expect(screen.queryByText('Next update:')).not.toBeInTheDocument();
        });

        it('should render minimal variant with icon and countdown', () => {
            render(<TestWrapper variant="minimal" />);

            const icon = screen.getByTestId('rotate-icon');
            expect(icon).toBeInTheDocument();

            // Should display countdown seconds
            expect(screen.getByText(/\d+s/)).toBeInTheDocument();

            // Should not display "Next update:" label
            expect(screen.queryByText('Next update:')).not.toBeInTheDocument();
        });

        it('should render full variant with icon, label, and countdown', () => {
            render(<TestWrapper variant="full" />);

            const icon = screen.getByTestId('rotate-icon');
            expect(icon).toBeInTheDocument();

            // Should display "Next update:" label
            expect(screen.getByText('Next update:')).toBeInTheDocument();

            // Should display countdown seconds
            expect(screen.getByText(/\d+s/)).toBeInTheDocument();
        });
    });

    describe('Countdown display', () => {
        it('should display countdown value in minimal variant', () => {
            render(<TestWrapper variant="minimal" />);

            // Default countdown is 30s
            expect(screen.getByText('30s')).toBeInTheDocument();
        });

        it('should display countdown value in full variant', () => {
            render(<TestWrapper variant="full" />);

            // Default countdown is 30s
            expect(screen.getByText('30s')).toBeInTheDocument();
        });

        it('should not display countdown in icon-only variant', () => {
            render(<TestWrapper variant="icon-only" />);

            expect(screen.queryByText('30s')).not.toBeInTheDocument();
        });
    });
});
