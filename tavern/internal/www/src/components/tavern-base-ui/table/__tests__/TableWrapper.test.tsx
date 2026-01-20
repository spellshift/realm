import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { TableWrapper } from '../TableWrapper';
import { ApolloError } from '@apollo/client';
import { MemoryRouter } from 'react-router-dom';
import { FilterProvider } from '../../../../context/FilterContext';
import { SortsProvider } from '../../../../context/SortContext';
import React from 'react';

// Mock EmptyState component
vi.mock('../../EmptyState', () => ({
  EmptyState: ({ type, label, children }: any) => (
    <div data-testid="empty-state" data-type={type}>
      {label}
      {children}
    </div>
  ),
  EmptyStateType: {
    error: 'error',
    loading: 'loading',
    noData: 'noData',
    noMatches: 'noMatches'
  }
}));

// Mock FilterControls and SortingControls since they're rendered internally
vi.mock('../../../../context/FilterContext/FilterControls', () => ({
  default: () => <div data-testid="filter-controls">Filter Controls</div>,
}));

vi.mock('../../../../context/SortContext/SortingControls', () => ({
  default: () => <div data-testid="sorting-controls">Sorting Controls</div>,
}));

// Mock Button component
vi.mock('../../button/Button', () => ({
  default: ({ children, onClick }: any) => (
    <button data-testid="clear-filters-button" onClick={onClick}>
      {children}
    </button>
  ),
}));

function TestWrapper({ children, path = '/hosts' }: { children: React.ReactNode; path?: string }) {
  return (
    <MemoryRouter initialEntries={[path]}>
      <SortsProvider>
        <FilterProvider>
          {children}
        </FilterProvider>
      </SortsProvider>
    </MemoryRouter>
  );
}

describe('TableWrapper', () => {
  const mockTable = <div data-testid="mock-table">Table Content</div>;
  const mockPagination = <div data-testid="mock-pagination">Pagination</div>;

  const defaultProps = {
    totalItems: 50,
    loading: false,
    error: undefined,
    table: mockTable,
    pagination: mockPagination
  };

  beforeEach(() => {
    sessionStorage.clear();
    vi.clearAllMocks();
  });

  describe('Error state rendering', () => {
    it('should render error EmptyState and hide table/pagination', () => {
      const error = new ApolloError({ errorMessage: 'Test error' });
      render(
        <TestWrapper>
          <TableWrapper {...defaultProps} error={error} />
        </TestWrapper>
      );

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
      render(
        <TestWrapper>
          <TableWrapper {...defaultProps} loading={true} />
        </TestWrapper>
      );

      const emptyState = screen.getByTestId('empty-state');
      expect(emptyState).toBeInTheDocument();
      expect(emptyState).toHaveAttribute('data-type', 'loading');
      expect(screen.getByText('Loading data...')).toBeInTheDocument();

      expect(screen.queryByTestId('mock-table')).not.toBeInTheDocument();
      expect(screen.queryByTestId('mock-pagination')).not.toBeInTheDocument();
    });

    it('should render loading EmptyState when totalItems is undefined', () => {
      render(
        <TestWrapper>
          <TableWrapper {...defaultProps} totalItems={undefined} loading={false} />
        </TestWrapper>
      );

      const emptyState = screen.getByTestId('empty-state');
      expect(emptyState).toBeInTheDocument();
      expect(emptyState).toHaveAttribute('data-type', 'loading');
    });
  });

  describe('Empty state rendering', () => {
    it('should render noData EmptyState when totalItems is 0 and hide table/pagination', () => {
      render(
        <TestWrapper>
          <TableWrapper {...defaultProps} totalItems={0} />
        </TestWrapper>
      );

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
      render(
        <TestWrapper>
          <TableWrapper {...defaultProps} />
        </TestWrapper>
      );

      expect(screen.getByTestId('mock-table')).toBeInTheDocument();
      expect(screen.getByTestId('mock-pagination')).toBeInTheDocument();
      expect(screen.queryByTestId('empty-state')).not.toBeInTheDocument();
    });
  });

  describe('Controls section rendering', () => {
    it('should render filter and sorting controls', () => {
      render(
        <TestWrapper>
          <TableWrapper {...defaultProps} />
        </TestWrapper>
      );

      expect(screen.getByTestId('filter-controls')).toBeInTheDocument();
      expect(screen.getByTestId('sorting-controls')).toBeInTheDocument();
    });

    it('should render custom title', () => {
      render(
        <TestWrapper>
          <TableWrapper {...defaultProps} title="Custom Title" />
        </TestWrapper>
      );

      expect(screen.getByText('Custom Title')).toBeInTheDocument();
    });

    it('should render default title when not provided', () => {
      render(
        <TestWrapper>
          <TableWrapper {...defaultProps} />
        </TestWrapper>
      );

      expect(screen.getByText('Table')).toBeInTheDocument();
    });
  });

  describe('State priority', () => {
    it('should prioritize error over loading', () => {
      const error = new ApolloError({ errorMessage: 'Test error' });
      render(
        <TestWrapper>
          <TableWrapper {...defaultProps} error={error} loading={true} />
        </TestWrapper>
      );

      const emptyState = screen.getByTestId('empty-state');
      expect(emptyState).toHaveAttribute('data-type', 'error');
      expect(screen.getByText('Error loading data')).toBeInTheDocument();
    });

    it('should prioritize loading over empty', () => {
      render(
        <TestWrapper>
          <TableWrapper {...defaultProps} loading={true} totalItems={0} />
        </TestWrapper>
      );

      const emptyState = screen.getByTestId('empty-state');
      expect(emptyState).toHaveAttribute('data-type', 'loading');
      expect(screen.getByText('Loading data...')).toBeInTheDocument();
    });
  });
});
