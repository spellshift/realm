import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { TableWrapper } from '../TableWrapper';
import { ApolloError } from '@apollo/client';

// Mock EmptyState component
vi.mock('../../EmptyState', () => ({
  EmptyState: ({ type, label }: any) => (
    <div data-testid="empty-state" data-type={type}>
      {label}
    </div>
  ),
  EmptyStateType: {
    error: 'error',
    loading: 'loading',
    noData: 'noData'
  }
}));

describe('TableWrapper', () => {
  const mockTable = <div data-testid="mock-table">Table Content</div>;
  const mockPagination = <div data-testid="mock-pagination">Pagination</div>;
  const mockFilterControls = <div data-testid="mock-filters">Filters</div>;
  const mockSortingControls = <div data-testid="mock-sorting">Sorting</div>;

  const defaultProps = {
    totalItems: 50,
    loading: false,
    error: undefined,
    table: mockTable,
    pagination: mockPagination
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Error state rendering', () => {
    it('should render error EmptyState and hide table/pagination', () => {
      const error = new ApolloError({ errorMessage: 'Test error' });
      render(<TableWrapper {...defaultProps} error={error} />);

      const emptyState = screen.getByTestId('empty-state');
      expect(emptyState).toBeInTheDocument();
      expect(emptyState).toHaveAttribute('data-type', 'error');
      expect(screen.getByText('Error loading data')).toBeInTheDocument();

      expect(screen.queryByTestId('mock-table')).not.toBeInTheDocument();
      expect(screen.queryByTestId('mock-pagination')).not.toBeInTheDocument();
    });
  });

  describe('Loading state rendering', () => {
    it('should render loading EmptyState when loading is true and hide table/pagination', () => {
      render(<TableWrapper {...defaultProps} loading={true} />);

      const emptyState = screen.getByTestId('empty-state');
      expect(emptyState).toBeInTheDocument();
      expect(emptyState).toHaveAttribute('data-type', 'loading');
      expect(screen.getByText('Loading data...')).toBeInTheDocument();

      expect(screen.queryByTestId('mock-table')).not.toBeInTheDocument();
      expect(screen.queryByTestId('mock-pagination')).not.toBeInTheDocument();
    });

    it('should render loading EmptyState when totalItems is null', () => {
      render(<TableWrapper {...defaultProps} totalItems={null} loading={false} />);

      const emptyState = screen.getByTestId('empty-state');
      expect(emptyState).toBeInTheDocument();
      expect(emptyState).toHaveAttribute('data-type', 'loading');
    });
  });

  describe('Empty state rendering', () => {
    it('should render noData EmptyState when totalItems is 0 and hide table/pagination', () => {
      render(<TableWrapper {...defaultProps} totalItems={0} />);

      const emptyState = screen.getByTestId('empty-state');
      expect(emptyState).toBeInTheDocument();
      expect(emptyState).toHaveAttribute('data-type', 'noData');
      expect(screen.getByText('No data found')).toBeInTheDocument();

      expect(screen.queryByTestId('mock-table')).not.toBeInTheDocument();
      expect(screen.queryByTestId('mock-pagination')).not.toBeInTheDocument();
    });
  });

  describe('Success state with data', () => {
    it('should render table and pagination when data exists', () => {
      render(<TableWrapper {...defaultProps} />);

      expect(screen.getByTestId('mock-table')).toBeInTheDocument();
      expect(screen.getByTestId('mock-pagination')).toBeInTheDocument();
      expect(screen.queryByTestId('empty-state')).not.toBeInTheDocument();
    });
  });

  describe('Controls section rendering', () => {
    it('should render filter and sorting controls when provided', () => {
      render(
        <TableWrapper
          {...defaultProps}
          filterControls={mockFilterControls}
          sortingControls={mockSortingControls}
        />
      );

      expect(screen.getByTestId('mock-filters')).toBeInTheDocument();
      expect(screen.getByTestId('mock-sorting')).toBeInTheDocument();
    });

    it('should not render controls section when no controls provided', () => {
      const { container } = render(<TableWrapper {...defaultProps} />);

      const controlsDiv = container.querySelector('.flex.flex-row.justify-between.items-center');
      expect(controlsDiv).not.toBeInTheDocument();
    });

    it('should render custom title', () => {
      render(<TableWrapper {...defaultProps} title="Custom Title" filterControls={mockFilterControls} />);

      expect(screen.getByText('Custom Title')).toBeInTheDocument();
    });

    it('should apply sticky styles when stickyControls is true', () => {
      const { container } = render(
        <TableWrapper {...defaultProps} filterControls={mockFilterControls} stickyControls={true} />
      );

      const controlsDiv = container.querySelector('.sticky.top-0.z-20.shadow-sm');
      expect(controlsDiv).toBeInTheDocument();
    });

    it('should not apply sticky styles when stickyControls is false', () => {
      const { container } = render(
        <TableWrapper {...defaultProps} filterControls={mockFilterControls} stickyControls={false} />
      );

      const controlsDiv = container.querySelector('.sticky.top-0.z-20.shadow-sm');
      expect(controlsDiv).not.toBeInTheDocument();
    });
  });

  describe('State priority', () => {
    it('should prioritize error over loading', () => {
      const error = new ApolloError({ errorMessage: 'Test error' });
      render(<TableWrapper {...defaultProps} error={error} loading={true} />);

      const emptyState = screen.getByTestId('empty-state');
      expect(emptyState).toHaveAttribute('data-type', 'error');
      expect(screen.getByText('Error loading data')).toBeInTheDocument();
    });

    it('should prioritize loading over empty', () => {
      render(<TableWrapper {...defaultProps} loading={true} totalItems={0} />);

      const emptyState = screen.getByTestId('empty-state');
      expect(emptyState).toHaveAttribute('data-type', 'loading');
      expect(screen.getByText('Loading data...')).toBeInTheDocument();
    });
  });
});
