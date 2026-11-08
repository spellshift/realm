import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ChakraProvider } from '@chakra-ui/react';
import { MemoryRouter } from 'react-router-dom';
import { MockedProvider } from '@apollo/client/testing';
import { AddTagToAll } from './AddTagToAll';
import { FilterProvider } from '../../context/FilterContext';
import { SortsProvider } from '../../context/SortContext/SortContext';
import { GET_TAG_OPTIONS } from '../host-details/components/editable-tag/useTagOptions';
import { GET_HOST_IDS_QUERY, UPDATE_TAG_HOSTS_MUTATION } from './queries';
import { defaultSorts } from '../../context/SortContext/sortingUtils';
import { PageNavItem } from '../../utils/enums';
import { getHostIdsQuery } from './useHostIds';
import { Filters } from '../../context/FilterContext/FilterContext';

const defaultFilters: Filters = {
  questName: '',
  taskOutput: '',
  beaconFields: [],
  tomeFields: [],
  tomeMultiSearch: '',
  assetName: '',
  userId: '',
  agentOnly: false,
};

const groupTag = { id: 'tag-group-1', name: 'Group One', kind: 'group' };
const serviceTag = { id: 'tag-service-1', name: 'Service One', kind: 'service' };

const tagOptionsMocks = [
  {
    request: {
      query: GET_TAG_OPTIONS,
      variables: { where: { kind: 'group' } },
    },
    result: {
      data: { tags: { edges: [{ node: groupTag }] } },
    },
  },
  {
    request: {
      query: GET_TAG_OPTIONS,
      variables: { where: { kind: 'service' } },
    },
    result: {
      data: { tags: { edges: [{ node: serviceTag }] } },
    },
  },
];

const hostIdsVariables = getHostIdsQuery(
  defaultFilters,
  undefined,
  defaultSorts[PageNavItem.hosts],
  100,
  'group'
);

const hostIdsMock = {
  request: {
    query: GET_HOST_IDS_QUERY,
    variables: hostIdsVariables,
  },
  result: {
    data: {
      hosts: {
        edges: [{ node: { id: 'host-1' } }, { node: { id: 'host-2' } }],
        pageInfo: {
          hasNextPage: false,
          hasPreviousPage: false,
          startCursor: 'cursor-1',
          endCursor: 'cursor-2',
        },
      },
    },
  },
};

const updateMock = {
  request: {
    query: UPDATE_TAG_HOSTS_MUTATION,
    variables: { tagID: groupTag.id, input: { addHostIDs: ['host-1', 'host-2'] } },
  },
  result: {
    data: { updateTag: { id: groupTag.id } },
  },
};

function renderWithProviders(mocks: any[], totalCount = 2, onComplete = vi.fn()) {
  return {
    onComplete,
    ...render(
      <ChakraProvider>
        <MemoryRouter initialEntries={['/hosts']}>
          <MockedProvider mocks={mocks} addTypename={false}>
            <FilterProvider>
              <SortsProvider>
                <AddTagToAll totalCount={totalCount} onComplete={onComplete} />
              </SortsProvider>
            </FilterProvider>
          </MockedProvider>
        </MemoryRouter>
      </ChakraProvider>
    ),
  };
}

describe('AddTagToAll', () => {
  beforeEach(() => {
    sessionStorage.clear();
    vi.clearAllMocks();
  });

  it('renders an "Add Tag to All" button', async () => {
    renderWithProviders([...tagOptionsMocks]);
    expect(await screen.findByRole('button', { name: 'Add Tag to All' })).toBeInTheDocument();
  });

  it('shows Group and Service tag groups in the dropdown', async () => {
    const user = userEvent.setup();
    renderWithProviders([...tagOptionsMocks]);

    await user.click(await screen.findByRole('button', { name: 'Add Tag to All' }));
    await user.click(await screen.findByText('Select a Group or Service tag'));

    expect(await screen.findByText('Group')).toBeInTheDocument();
    expect(await screen.findByText('Service')).toBeInTheDocument();
    expect(await screen.findByText('Group One')).toBeInTheDocument();
    expect(await screen.findByText('Service One')).toBeInTheDocument();
  });

  it('keeps the add button disabled until a tag is selected', async () => {
    const user = userEvent.setup();
    renderWithProviders([...tagOptionsMocks]);

    await user.click(await screen.findByRole('button', { name: 'Add Tag to All' }));

    const addButton = await screen.findByRole('button', { name: 'Add tag to all' });
    expect(addButton).toBeDisabled();

    await user.click(await screen.findByText('Select a Group or Service tag'));
    await user.click(await screen.findByText('Group One'));

    await waitFor(() => expect(addButton).not.toBeDisabled());
  });

  it('applies the selected tag to every host in the current view', async () => {
    const user = userEvent.setup();
    const { onComplete } = renderWithProviders([...tagOptionsMocks, hostIdsMock, updateMock]);

    await user.click(await screen.findByRole('button', { name: 'Add Tag to All' }));
    await user.click(await screen.findByText('Select a Group or Service tag'));
    await user.click(await screen.findByText('Group One'));

    const addButton = await screen.findByRole('button', { name: 'Add tag to all' });
    await user.click(addButton);

    await waitFor(() => {
      expect(screen.getByText(/added to 2 hosts/i)).toBeInTheDocument();
    });
    expect(onComplete).toHaveBeenCalled();
  });

  it('reports how many hosts were skipped for already having that kind of tag', async () => {
    const user = userEvent.setup();
    renderWithProviders([...tagOptionsMocks, hostIdsMock, updateMock], 3);

    await user.click(await screen.findByRole('button', { name: 'Add Tag to All' }));
    await user.click(await screen.findByText('Select a Group or Service tag'));
    await user.click(await screen.findByText('Group One'));

    const addButton = await screen.findByRole('button', { name: 'Add tag to all' });
    await user.click(addButton);

    await waitFor(() => {
      expect(screen.getByText(/added to 2 hosts/i)).toBeInTheDocument();
    });
    expect(screen.getByText(/1 skipped/i)).toBeInTheDocument();
  });

  it('disables the add button when there are no hosts in the current view', async () => {
    const user = userEvent.setup();
    renderWithProviders([...tagOptionsMocks], 0);

    await user.click(await screen.findByRole('button', { name: 'Add Tag to All' }));

    const addButton = await screen.findByRole('button', { name: 'Add tag to all' });
    expect(addButton).toBeDisabled();
    expect(await screen.findByText(/no hosts in the current view/i)).toBeInTheDocument();
  });
});
