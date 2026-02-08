import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import CreateLinkModal from '../CreateLinkModal/CreateLinkModal';
import { render } from '../../../../test-utils';

// Mock useCreateLink
const mockCreateLink = vi.fn();

vi.mock('../../useAssets', () => ({
    useCreateLink: () => ({
        createLink: mockCreateLink,
        loading: false,
        error: null,
        data: null
    }),
}));

// Mock Modal to simplify testing
vi.mock('../../../components/tavern-base-ui/Modal', () => ({
    default: ({ children, isOpen, setOpen }: any) => {
        if (!isOpen) return null;
        return (
            <div role="dialog">
                <button onClick={() => setOpen(false)} aria-label="Close panel">Close</button>
                {children}
            </div>
        );
    }
}));

describe('CreateLinkModal', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders and initializes form values correctly', async () => {
        const setOpen = vi.fn();
        render(
            <CreateLinkModal
                isOpen={true}
                setOpen={setOpen}
                assetId="123"
                assetName="Test Asset"
            />
        );

        expect(screen.getByText('Create link for Test Asset')).toBeInTheDocument();

        // Check path initialization (random string)
        const pathInput = screen.getByLabelText(/path/i) as HTMLInputElement;
        expect(pathInput.value).not.toBe('');
        expect(pathInput.value).toHaveLength(12);
        expect(pathInput.value).toMatch(/^[A-Za-z0-9]+$/);
    });

    it('submits form with correct values', async () => {
        const user = userEvent.setup();
        const setOpen = vi.fn();
        const onSuccess = vi.fn();

        // Mock successful response
        mockCreateLink.mockResolvedValue({
            data: {
                createLink: {
                    path: 'new-link-path'
                }
            }
        });

        render(
            <CreateLinkModal
                isOpen={true}
                setOpen={setOpen}
                assetId="123"
                assetName="Test Asset"
                onSuccess={onSuccess}
            />
        );

        // Fill form
        // Path is pre-filled, let's keep it or change it
        const pathInput = screen.getByLabelText(/path/i);
        await user.clear(pathInput);
        await user.type(pathInput, 'custom-path');

        // Submit
        const submitButton = screen.getByText('Create Link');
        await user.click(submitButton);

        await waitFor(() => {
            expect(mockCreateLink).toHaveBeenCalledWith({
                variables: {
                    input: expect.objectContaining({
                        assetID: '123',
                        path: 'custom-path',
                        downloadLimit: null, // Default
                        // expiryMode 0 -> 10 mins from now roughly
                    })
                }
            });
        });

        // Success UI should show
        expect(await screen.findByText('Link Created!')).toBeInTheDocument();
        expect(screen.getByText(/new-link-path/)).toBeInTheDocument();
    });

    it('validates required fields', async () => {
        const user = userEvent.setup();
        const setOpen = vi.fn();

        render(
            <CreateLinkModal
                isOpen={true}
                setOpen={setOpen}
                assetId="123"
                assetName="Test Asset"
            />
        );

        const pathInput = screen.getByLabelText(/path/i);
        await user.clear(pathInput);

        const submitButton = screen.getByText('Create Link');
        await user.click(submitButton);

        // Path is required
        expect(await screen.findByText('Path is required')).toBeInTheDocument();
        expect(mockCreateLink).not.toHaveBeenCalled();
    });
});
