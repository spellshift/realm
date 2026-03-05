import { test, expect } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { CreateShellButton } from './CreateShellButton';
import { MockedProvider } from '@apollo/client/testing';
import { MemoryRouter } from 'react-router-dom';
import { GET_BEACONS_FOR_HOST_QUERY, CREATE_SHELL_MUTATION } from './queries';
import { SupportedTransports } from '../../utils/enums';
import { ChakraProvider } from '@chakra-ui/react';
import { vi } from 'vitest';

const mockHostId = 'mock-host-id';
const mockBeaconId = 'mock-beacon-id';
const mockShellId = 'mock-shell-id';

const mocks = [
  {
    request: {
      query: GET_BEACONS_FOR_HOST_QUERY,
      variables: { hostId: mockHostId },
    },
    result: {
      data: {
        beacons: {
          edges: [
            {
              node: {
                id: mockBeaconId,
                principal: 'root',
                interval: 10,
                // Far in the future
                lastSeenAt: '2035-01-01T00:00:00.000Z',
                nextSeenAt: '2035-01-01T00:00:10.000Z',
                transport: SupportedTransports.GRPC,
              },
            },
          ],
        },
      },
    },
  },
  {
    request: {
      query: CREATE_SHELL_MUTATION,
      variables: { beaconId: mockBeaconId },
    },
    result: {
      data: {
        createShell: {
          id: mockShellId,
        },
      },
    },
  },
];

test('CreateShellButton opens new tab on click', async () => {
  // Mock window.open
  const openMock = vi.fn();
  vi.stubGlobal('open', openMock);

  render(
    <ChakraProvider>
      <MemoryRouter>
        <MockedProvider mocks={mocks} addTypename={false}>
          <CreateShellButton hostId={mockHostId} />
        </MockedProvider>
      </MemoryRouter>
    </ChakraProvider>
  );

  // Wait for the button to appear (it renders after fetching beacons)
  const button = await screen.findByRole('button', { name: /create shell/i });
  expect(button).toBeDefined();

  // Click the button
  fireEvent.click(button);

  // Wait for window.open to have been called
  await waitFor(() => {
    expect(openMock).toHaveBeenCalledWith(`/shellv2/${mockShellId}`, '_blank');
  });

  vi.unstubAllGlobals();
});