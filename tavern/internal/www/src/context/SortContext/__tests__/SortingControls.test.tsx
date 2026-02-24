import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom/vitest';
import SortingControls from '../SortingControls';
import { SortsProvider, useSorts } from '../SortContext';
import { HostOrderField, OrderDirection, PageNavItem, QuestOrderField, TaskOrderField } from '../../../utils/enums';
import React from 'react';
import { SortablePageNavItem } from '../sortingUtils';

// Mock heroicons
vi.mock('@heroicons/react/24/outline', () => ({
  BarsArrowDownIcon: (props: any) => <svg data-testid="icon-down" {...props} />,
  BarsArrowUpIcon: (props: any) => <svg data-testid="icon-up" {...props} />,
}));

// Mock ButtonDialogPopover
vi.mock('../../../components/ButtonDialogPopover', () => ({
  ButtonDialogPopover: ({ children, label, leftIcon }: any) => (
    <div data-testid="button-dialog-popover">
      <button data-testid="popover-button">
        {leftIcon}
        {label}
      </button>
      <div data-testid="popover-content">{children}</div>
    </div>
  ),
}));

// Mock SingleDropdownSelector
vi.mock('../../../components/tavern-base-ui/SingleDropdownSelector', () => ({
  default: ({ label, options, setSelectedOption, defaultValue }: any) => (
    <div data-testid={`dropdown-${label.toLowerCase()}`}>
      <label>{label}</label>
      <select
        data-testid={`select-${label.toLowerCase()}`}
        defaultValue={defaultValue.value}
        onChange={(e) => {
          const selectedOption = options.find((opt: any) => opt.value === e.target.value);
          if (selectedOption) setSelectedOption(selectedOption);
        }}
      >
        {options.map((opt: any) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </div>
  ),
}));

function SortsDisplay() {
  const { sorts } = useSorts();
  return <div data-testid="sorts-display">{JSON.stringify(sorts)}</div>;
}

function TestWrapper({ sortType, includeDisplay = false }: { sortType: SortablePageNavItem; includeDisplay?: boolean }) {
  return (
    <SortsProvider>
      <SortingControls sortType={sortType} />
      {includeDisplay && <SortsDisplay />}
    </SortsProvider>
  );
}

describe('SortingControls', () => {
  beforeEach(() => {
    sessionStorage.clear();
    vi.clearAllMocks();
  });

  describe('Rendering for different page types', () => {
    it('should render with default host sort settings', () => {
      render(<TestWrapper sortType={PageNavItem.hosts} />);

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Sort (Created At)');
      expect(screen.getByTestId('icon-down')).toBeInTheDocument();
    });

    it('should render with default quest sort settings', () => {
      render(<TestWrapper sortType={PageNavItem.quests} />);

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Sort (Created At)');
      expect(screen.getByTestId('icon-down')).toBeInTheDocument();
    });

    it('should render with default task sort settings', () => {
      render(<TestWrapper sortType={PageNavItem.tasks} />);

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Sort (Last Modified At)');
      expect(screen.getByTestId('icon-down')).toBeInTheDocument();
    });

    it('should render with default process sort settings', () => {
      render(<TestWrapper sortType={PageNavItem.processes} />);

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Sort (Last Modified At)');
      expect(screen.getByTestId('icon-down')).toBeInTheDocument();
    });
  });

  describe('Sort field dropdown', () => {
    it('should display correct field options for hosts page', () => {
      render(<TestWrapper sortType={PageNavItem.hosts} />);

      const fieldSelect = screen.getByTestId('select-field');
      const options = Array.from(fieldSelect.querySelectorAll('option'));

      expect(options).toHaveLength(Object.keys(HostOrderField).length);
      expect(options.map((opt) => opt.textContent)).toContain('Created At');
      expect(options.map((opt) => opt.textContent)).toContain('Last Seen At');
    });

    it('should display correct field options for quests page', () => {
      render(<TestWrapper sortType={PageNavItem.quests} />);

      const fieldSelect = screen.getByTestId('select-field');
      const options = Array.from(fieldSelect.querySelectorAll('option'));

      expect(options).toHaveLength(Object.keys(QuestOrderField).length);
      expect(options.map((opt) => opt.textContent)).toContain('Created At');
      expect(options.map((opt) => opt.textContent)).toContain('Name');
    });

    it('should display correct field options for tasks page', () => {
      render(<TestWrapper sortType={PageNavItem.tasks} />);

      const fieldSelect = screen.getByTestId('select-field');
      const options = Array.from(fieldSelect.querySelectorAll('option'));

      expect(options).toHaveLength(Object.keys(TaskOrderField).length);
      expect(options.map((opt) => opt.textContent)).toContain('Last Modified At');
      expect(options.map((opt) => opt.textContent)).toContain('Created At');
    });
  });

  describe('Sort direction dropdown', () => {
    it('should display ascending and descending options', () => {
      render(<TestWrapper sortType={PageNavItem.hosts} />);

      const directionSelect = screen.getByTestId('select-direction');
      const options = Array.from(directionSelect.querySelectorAll('option'));

      expect(options).toHaveLength(2);
      expect(options.map((opt) => opt.textContent)).toEqual(['Ascending', 'Descending']);
    });

    it('should default to descending direction', () => {
      render(<TestWrapper sortType={PageNavItem.hosts} />);

      const directionSelect = screen.getByTestId('select-direction') as HTMLSelectElement;
      expect(directionSelect.value).toBe(OrderDirection.Desc);
    });
  });

  describe('Updating sort settings', () => {
    it('should update sort field when field dropdown changes', async () => {
      const user = userEvent.setup();
      render(<TestWrapper sortType={PageNavItem.hosts} includeDisplay />);

      const getSorts = () => JSON.parse(screen.getByTestId('sorts-display').textContent || '{}');

      const fieldSelect = screen.getByTestId('select-field');
      await user.selectOptions(fieldSelect, HostOrderField.LastSeenAt);

      expect(getSorts()[PageNavItem.hosts].field).toBe(HostOrderField.LastSeenAt);
      expect(getSorts()[PageNavItem.hosts].direction).toBe(OrderDirection.Desc);
    });

    it('should update sort direction when direction dropdown changes', async () => {
      const user = userEvent.setup();
      render(<TestWrapper sortType={PageNavItem.hosts} includeDisplay />);

      const getSorts = () => JSON.parse(screen.getByTestId('sorts-display').textContent || '{}');

      const directionSelect = screen.getByTestId('select-direction');
      await user.selectOptions(directionSelect, OrderDirection.Asc);

      expect(getSorts()[PageNavItem.hosts].direction).toBe(OrderDirection.Asc);
      expect(getSorts()[PageNavItem.hosts].field).toBe(HostOrderField.CreatedAt);
    });

    it('should update label when field changes', async () => {
      const user = userEvent.setup();
      render(<TestWrapper sortType={PageNavItem.quests} />);

      const fieldSelect = screen.getByTestId('select-field');
      await user.selectOptions(fieldSelect, QuestOrderField.Name);

      expect(screen.getByTestId('popover-button')).toHaveTextContent('Sort (Name)');
    });

    it('should update icon when direction changes to ascending', async () => {
      const user = userEvent.setup();
      render(<TestWrapper sortType={PageNavItem.hosts} />);

      const directionSelect = screen.getByTestId('select-direction');
      await user.selectOptions(directionSelect, OrderDirection.Asc);

      expect(screen.getByTestId('icon-up')).toBeInTheDocument();
    });

    it('should only update sorts for the current page type', async () => {
      const user = userEvent.setup();
      render(<TestWrapper sortType={PageNavItem.hosts} includeDisplay />);

      const getSorts = () => JSON.parse(screen.getByTestId('sorts-display').textContent || '{}');

      const fieldSelect = screen.getByTestId('select-field');
      await user.selectOptions(fieldSelect, HostOrderField.LastSeenAt);

      expect(getSorts()[PageNavItem.hosts].field).toBe(HostOrderField.LastSeenAt);
      expect(getSorts()[PageNavItem.quests].field).toBe(QuestOrderField.CreatedAt);
      expect(getSorts()[PageNavItem.tasks].field).toBe(TaskOrderField.LastModifiedAt);
    });
  });

});
