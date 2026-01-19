import { describe, it, beforeEach, vi, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import FilterControls from '../FilterControls';
import { FilterProvider, useFilters } from '../FilterContext';
import { MemoryRouter } from 'react-router-dom';
import React from 'react';

// Mock the child components
vi.mock('../../../components/beacon-filter-bar', () => ({
  BeaconFilterBar: ({ setFiltersSelected, filtersSelected, isDisabled }: any) => (
    <div data-testid="beacon-filter-bar">
      <span data-testid="beacon-disabled">{isDisabled ? 'disabled' : 'enabled'}</span>
      <span data-testid="beacon-count">{filtersSelected.length}</span>
      <button onClick={() => setFiltersSelected([{ kind: 'beacon', id: '1', name: 'Test Beacon' }])}>
        Add Beacon
      </button>
    </div>
  ),
}));

vi.mock('../../../components/TomeFilterBar', () => ({
  TomeFilterBar: ({ setFiltersSelected, filtersSelected, isDisabled }: any) => (
    <div data-testid="tome-filter-bar">
      <span data-testid="tome-disabled">{isDisabled ? 'disabled' : 'enabled'}</span>
      <span data-testid="tome-count">{filtersSelected.length}</span>
      <button onClick={() => setFiltersSelected([{ kind: 'tome', id: 't1', name: 'Test Tome' }])}>
        Add Tome
      </button>
    </div>
  ),
}));

vi.mock('../../../components/tavern-base-ui/FreeTextSearch', () => ({
  default: ({ setSearch, defaultValue, placeholder, isDisabled }: any) => (
    <div data-testid={`free-text-search-${placeholder}`}>
      <input
        type="text"
        placeholder={placeholder}
        defaultValue={defaultValue}
        disabled={isDisabled}
        onChange={(e) => setSearch(e.target.value)}
        data-testid={`search-input-${placeholder}`}
      />
    </div>
  ),
}));

vi.mock('../../../components/ButtonDialogPopover', () => ({
  ButtonDialogPopover: ({ children, label }: any) => (
    <div data-testid="button-dialog-popover">
      <button data-testid="popover-button">{label}</button>
      <div data-testid="popover-content">{children}</div>
    </div>
  ),
}));

function FiltersDisplay() {
  const { filters } = useFilters();
  return <div data-testid="filters-display">{JSON.stringify(filters)}</div>;
}

function TestWrapper({ path, includeDisplay = false }: { path: string; includeDisplay?: boolean }) {
  return (
    <MemoryRouter initialEntries={[path]}>
      <FilterProvider>
        <FilterControls />
        {includeDisplay && <FiltersDisplay />}
      </FilterProvider>
    </MemoryRouter>
  );
}

describe('FilterControls', () => {
  beforeEach(() => {
    sessionStorage.clear();
    vi.clearAllMocks();
  });

  describe('Filter configuration by page type', () => {
    it('should render all filters for QUEST page', () => {
      render(<TestWrapper path="/quests" />);

      expect(screen.getByTestId('beacon-filter-bar')).toBeInTheDocument();
      expect(screen.getByTestId('tome-filter-bar')).toBeInTheDocument();
      expect(screen.getByTestId('free-text-search-Tome definition & values')).toBeInTheDocument();
      expect(screen.getByTestId('free-text-search-Quest name')).toBeInTheDocument();
      expect(screen.getByTestId('free-text-search-Task output')).toBeInTheDocument();
    });

    it('should render only beacon filter for HOST page', () => {
      render(<TestWrapper path="/hosts" />);

      expect(screen.getByTestId('beacon-filter-bar')).toBeInTheDocument();
      expect(screen.queryByTestId('tome-filter-bar')).not.toBeInTheDocument();
      expect(screen.queryByTestId('free-text-search-Quest name')).not.toBeInTheDocument();
      expect(screen.queryByTestId('free-text-search-Task output')).not.toBeInTheDocument();
    });

    it('should render beacon and task output filters for TASKS page', () => {
      render(<TestWrapper path="/tasks" />);

      expect(screen.getByTestId('beacon-filter-bar')).toBeInTheDocument();
      expect(screen.getByTestId('free-text-search-Task output')).toBeInTheDocument();
      expect(screen.queryByTestId('tome-filter-bar')).not.toBeInTheDocument();
      expect(screen.queryByTestId('free-text-search-Quest name')).not.toBeInTheDocument();
    });

    it('should render host task filters for host detail page', () => {
      render(<TestWrapper path="/hosts/123" />);

      expect(screen.getByTestId('tome-filter-bar')).toBeInTheDocument();
      expect(screen.getByTestId('free-text-search-Tome definition & values')).toBeInTheDocument();
      expect(screen.getByTestId('free-text-search-Quest name')).toBeInTheDocument();
      expect(screen.getByTestId('free-text-search-Task output')).toBeInTheDocument();
      expect(screen.queryByTestId('beacon-filter-bar')).not.toBeInTheDocument();
    });

    it('should not render filters on non-filterable pages', () => {
      render(<TestWrapper path="/admin" />);

      expect(screen.queryByTestId('button-dialog-popover')).not.toBeInTheDocument();
    });
  });

  describe('Filter label calculation', () => {
    it('should display "Filters (0)" when no filters are active', () => {
      render(<TestWrapper path="/quests" />);

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Filters (0)');
    });

    it('should count text search filters correctly', async () => {
      const user = userEvent.setup();
      render(<TestWrapper path="/quests" />);

      const questInput = screen.getByTestId('search-input-Quest name');
      await user.type(questInput, 'test-quest');

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Filters (1)');
    });

    it('should count beacon field filters by array length', async () => {
      const user = userEvent.setup();
      render(<TestWrapper path="/quests" />);

      const addButton = screen.getByText('Add Beacon');
      await user.click(addButton);

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Filters (1)');
      expect(screen.getByTestId('beacon-count')).toHaveTextContent('1');
    });

    it('should count multiple active filters correctly', async () => {
      const user = userEvent.setup();
      render(<TestWrapper path="/quests" />);

      await user.type(screen.getByTestId('search-input-Quest name'), 'test');
      await user.type(screen.getByTestId('search-input-Task output'), 'output');
      await user.click(screen.getByText('Add Beacon'));

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Filters (3)');
    });
  });

  describe('Filter lock/unlock toggle', () => {
    it('should toggle isLocked when lock button is clicked', async () => {
      const user = userEvent.setup();
      render(<TestWrapper path="/quests" includeDisplay />);

      const lockButton = screen.getByRole('button', { name: /lock filters/i });
      const getFilters = () => JSON.parse(screen.getByTestId('filters-display').textContent || '{}');

      expect(getFilters().isLocked).toBe(false);

      await user.click(lockButton);

      expect(getFilters().isLocked).toBe(true);
    });

    it('should disable filter components when isLocked is true', async () => {
      const user = userEvent.setup();
      render(<TestWrapper path="/quests" />);

      expect(screen.getByTestId('beacon-disabled')).toHaveTextContent('enabled');
      expect(screen.getByTestId('tome-disabled')).toHaveTextContent('enabled');

      const lockButton = screen.getByRole('button', { name: /lock filters/i });
      await user.click(lockButton);

      expect(screen.getByTestId('beacon-disabled')).toHaveTextContent('disabled');
      expect(screen.getByTestId('tome-disabled')).toHaveTextContent('disabled');
    });

    it('should show unlock button when filters are locked', async () => {
      const user = userEvent.setup();
      render(<TestWrapper path="/quests" />);

      const lockButton = screen.getByRole('button', { name: /lock filters/i });
      await user.click(lockButton);

      expect(screen.getByRole('button', { name: /unlock filters/i })).toBeInTheDocument();
    });
  });

  describe('Filter component interactions', () => {
    it('should update filter state when inputs change', async () => {
      const user = userEvent.setup();
      render(<TestWrapper path="/quests" includeDisplay />);

      const getFilters = () => JSON.parse(screen.getByTestId('filters-display').textContent || '{}');

      await user.type(screen.getByTestId('search-input-Quest name'), 'my-quest');
      expect(getFilters().questName).toBe('my-quest');

      await user.type(screen.getByTestId('search-input-Task output'), 'error output');
      expect(getFilters().taskOutput).toBe('error output');

      await user.type(screen.getByTestId('search-input-Tome definition & values'), 'tome search');
      expect(getFilters().tomeMultiSearch).toBe('tome search');
    });

    it('should update beaconFields and tomeFields arrays', async () => {
      const user = userEvent.setup();
      render(<TestWrapper path="/quests" includeDisplay />);

      const getFilters = () => JSON.parse(screen.getByTestId('filters-display').textContent || '{}');

      await user.click(screen.getByText('Add Beacon'));
      expect(getFilters().beaconFields).toHaveLength(1);
      expect(getFilters().beaconFields[0]).toEqual({
        kind: 'beacon',
        id: '1',
        name: 'Test Beacon',
      });

      await user.click(screen.getByText('Add Tome'));
      expect(getFilters().tomeFields).toHaveLength(1);
      expect(getFilters().tomeFields[0]).toEqual({
        kind: 'tome',
        id: 't1',
        name: 'Test Tome',
      });
    });

    it('should preserve filter values when toggling lock state', async () => {
      const user = userEvent.setup();
      render(<TestWrapper path="/quests" includeDisplay />);

      const getFilters = () => JSON.parse(screen.getByTestId('filters-display').textContent || '{}');

      await user.type(screen.getByTestId('search-input-Quest name'), 'preserved');

      const lockButton = screen.getByRole('button', { name: /lock filters/i });
      await user.click(lockButton); // lock

      const unlockButton = screen.getByRole('button', { name: /unlock filters/i });
      await user.click(unlockButton); // unlock

      expect(getFilters().questName).toBe('preserved');
      expect(getFilters().isLocked).toBe(false);
    });
  });
});
