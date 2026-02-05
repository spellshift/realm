import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import TablePagination from '../TablePagination';
import { QueryPageInfo } from '../../../../utils/interfacesQuery';

// Mock Button component
vi.mock('../../button/Button', () => ({
  default: ({ children, disabled, onClick }: any) => (
    <button
      data-testid={`button-${children.toLowerCase()}`}
      disabled={disabled}
      onClick={onClick}
    >
      {children}
    </button>
  )
}));

describe('TablePagination', () => {
  const mockPageInfo: QueryPageInfo = {
    hasNextPage: true,
    hasPreviousPage: false,
    startCursor: 'start-cursor-123',
    endCursor: 'end-cursor-456'
  };

  const defaultProps = {
    totalCount: 100,
    pageInfo: mockPageInfo,
    refetchTable: vi.fn(),
    page: 1,
    setPage: vi.fn(),
    rowLimit: 10
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Pagination info display', () => {
    it('should display current page number', () => {
      render(<TablePagination {...defaultProps} page={5} />);

      expect(screen.getByText('5', { exact: false })).toBeInTheDocument();
    });

    it('should display total page count', () => {
      render(<TablePagination {...defaultProps} totalCount={100} rowLimit={10} />);

      // 100 / 10 = 10 pages
      const pageCountElements = screen.getAllByText('10');
      expect(pageCountElements.length).toBeGreaterThan(0);
    });

    it('should display total results count', () => {
      render(<TablePagination {...defaultProps} totalCount={150} />);

      expect(screen.getByText('150 results', { exact: false })).toBeInTheDocument();
    });
  });

  describe('Button states', () => {
    it.each([
      { page: 1, expectedPrevDisabled: true, expectedNextDisabled: false },
      { page: 10, expectedPrevDisabled: false, expectedNextDisabled: true },
    ])('should set button states based on page position (page: $page)',
      ({ page, expectedPrevDisabled, expectedNextDisabled }) => {
      render(<TablePagination {...defaultProps} page={page} />);

      const previousButton = screen.getByTestId('button-previous');
      const nextButton = screen.getByTestId('button-next');

      if (expectedPrevDisabled) {
        expect(previousButton).toBeDisabled();
      } else {
        expect(previousButton).not.toBeDisabled();
      }

      if (expectedNextDisabled) {
        expect(nextButton).toBeDisabled();
      } else {
        expect(nextButton).not.toBeDisabled();
      }
    });
  });

  describe('Navigation callbacks', () => {
    it('should call refetchTable with startCursor on Previous click', async () => {
      const user = userEvent.setup();
      const mockRefetch = vi.fn();

      render(<TablePagination {...defaultProps} page={5} refetchTable={mockRefetch} />);

      const previousButton = screen.getByTestId('button-previous');
      await user.click(previousButton);

      expect(mockRefetch).toHaveBeenCalledWith(null, 'start-cursor-123');
    });

    it('should call refetchTable with endCursor on Next click', async () => {
      const user = userEvent.setup();
      const mockRefetch = vi.fn();

      render(<TablePagination {...defaultProps} refetchTable={mockRefetch} />);

      const nextButton = screen.getByTestId('button-next');
      await user.click(nextButton);

      expect(mockRefetch).toHaveBeenCalledWith('end-cursor-456', null);
    });

    it('should decrement page on Previous click', async () => {
      const user = userEvent.setup();
      const pageInfo = { ...mockPageInfo, hasPreviousPage: true };
      const mockSetPage = vi.fn();

      render(<TablePagination {...defaultProps} pageInfo={pageInfo} page={5} setPage={mockSetPage} />);

      const previousButton = screen.getByTestId('button-previous');
      await user.click(previousButton);

      expect(mockSetPage).toHaveBeenCalled();
      // Call the function that was passed to setPage
      const setPageCallback = mockSetPage.mock.calls[0][0];
      expect(setPageCallback(5)).toBe(4);
    });

    it('should increment page on Next click', async () => {
      const user = userEvent.setup();
      const mockSetPage = vi.fn();

      render(<TablePagination {...defaultProps} page={3} setPage={mockSetPage} />);

      const nextButton = screen.getByTestId('button-next');
      await user.click(nextButton);

      expect(mockSetPage).toHaveBeenCalled();
      // Call the function that was passed to setPage
      const setPageCallback = mockSetPage.mock.calls[0][0];
      expect(setPageCallback(3)).toBe(4);
    });
  });

  describe('Page count calculation', () => {
    it('should calculate page count correctly and round up with remainder', () => {
      render(<TablePagination {...defaultProps} totalCount={95} rowLimit={10} />);

      // 95 / 10 = 9.5, should round up to 10
      const pageCountElements = screen.getAllByText('10');
      expect(pageCountElements.length).toBeGreaterThan(0);
    });
  });
});
