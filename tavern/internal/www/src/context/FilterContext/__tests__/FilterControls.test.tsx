import { describe, it, beforeEach, vi } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import FilterControls, { FilterPageType } from '../FilterControls';
import { FilterProvider, useFilters } from '../FilterContext';

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

function TestWrapper({ type, includeDisplay = false }: { type: FilterPageType; includeDisplay?: boolean }) {
  return (
    <FilterProvider>
      <FilterControls type={type} />
      {includeDisplay && <FiltersDisplay />}
    </FilterProvider>
  );
}

describe('FilterControls', () => {
  beforeEach(() => {
    sessionStorage.clear();
    vi.clearAllMocks();
  });

  describe('Filter configuration by page type', () => {
    it('should render all filters for QUEST page', () => {
      render(<TestWrapper type={FilterPageType.QUEST} />);

      expect(screen.getByTestId('beacon-filter-bar')).toBeInTheDocument();
      expect(screen.getByTestId('tome-filter-bar')).toBeInTheDocument();
      expect(screen.getByTestId('free-text-search-Tome definition & values')).toBeInTheDocument();
      expect(screen.getByTestId('free-text-search-Quest name')).toBeInTheDocument();
      expect(screen.getByTestId('free-text-search-Task output')).toBeInTheDocument();
    });

    it('should render only beacon filter for HOST page', () => {
      render(<TestWrapper type={FilterPageType.HOST} />);

      expect(screen.getByTestId('beacon-filter-bar')).toBeInTheDocument();
      expect(screen.queryByTestId('tome-filter-bar')).not.toBeInTheDocument();
      expect(screen.queryByTestId('free-text-search-Quest name')).not.toBeInTheDocument();
      expect(screen.queryByTestId('free-text-search-Task output')).not.toBeInTheDocument();
    });
  });

  describe('Filter label calculation', () => {
    it('should display "Filter (disabled)" when filters are disabled', async () => {
      const user = userEvent.setup();
      render(<TestWrapper type={FilterPageType.QUEST} />);

      const filterSwitch = screen.getByRole('checkbox');
      await user.click(filterSwitch);

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Filter (disabled)');
      expect(filterSwitch).not.toBeChecked();
    });

    it('should display "Filters (0)" when no filters are active', () => {
      render(<TestWrapper type={FilterPageType.QUEST} />);

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Filters (0)');
    });

    it('should count text search filters correctly', async () => {
      const user = userEvent.setup();
      render(<TestWrapper type={FilterPageType.QUEST} />);

      const questInput = screen.getByTestId('search-input-Quest name');
      await user.type(questInput, 'test-quest');

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Filters (1)');
    });

    it('should count beacon field filters by array length', async () => {
      const user = userEvent.setup();
      render(<TestWrapper type={FilterPageType.QUEST} />);

      const addButton = screen.getByText('Add Beacon');
      await user.click(addButton);

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Filters (1)');
      expect(screen.getByTestId('beacon-count')).toHaveTextContent('1');
    });

    it('should count multiple active filters correctly', async () => {
      const user = userEvent.setup();
      render(<TestWrapper type={FilterPageType.QUEST} />);

      await user.type(screen.getByTestId('search-input-Quest name'), 'test');
      await user.type(screen.getByTestId('search-input-Task output'), 'output');
      await user.click(screen.getByText('Add Beacon'));

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Filters (3)');
    });
  });

  describe('Filter enable/disable toggle', () => {
    it('should toggle filtersEnabled when switch is clicked', async () => {
      const user = userEvent.setup();
      render(<TestWrapper type={FilterPageType.QUEST} includeDisplay />);

      const filterSwitch = screen.getByRole('checkbox');
      expect(filterSwitch).toBeChecked();

      const getFilters = () => JSON.parse(screen.getByTestId('filters-display').textContent || '{}');
      expect(getFilters().filtersEnabled).toBe(true);

      await user.click(filterSwitch);

      expect(filterSwitch).not.toBeChecked();
      expect(getFilters().filtersEnabled).toBe(false);
    });

    it('should disable filter components when filtersEnabled is false', async () => {
      const user = userEvent.setup();
      render(<TestWrapper type={FilterPageType.QUEST} />);

      expect(screen.getByTestId('beacon-disabled')).toHaveTextContent('enabled');
      expect(screen.getByTestId('tome-disabled')).toHaveTextContent('enabled');

      await user.click(screen.getByRole('checkbox'));

      expect(screen.getByTestId('beacon-disabled')).toHaveTextContent('disabled');
      expect(screen.getByTestId('tome-disabled')).toHaveTextContent('disabled');
    });
  });

  describe('Filter component interactions', () => {
    it('should update filter state when inputs change', async () => {
      const user = userEvent.setup();
      render(<TestWrapper type={FilterPageType.QUEST} includeDisplay />);

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
      render(<TestWrapper type={FilterPageType.QUEST} includeDisplay />);

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

    it('should preserve filter values when toggling enabled/disabled', async () => {
      const user = userEvent.setup();
      render(<TestWrapper type={FilterPageType.QUEST} includeDisplay />);

      const getFilters = () => JSON.parse(screen.getByTestId('filters-display').textContent || '{}');

      await user.type(screen.getByTestId('search-input-Quest name'), 'preserved');
      await user.click(screen.getByRole('checkbox')); // disable
      await user.click(screen.getByRole('checkbox')); // enable

      expect(getFilters().questName).toBe('preserved');
      expect(getFilters().filtersEnabled).toBe(true);
    });
  });
});
