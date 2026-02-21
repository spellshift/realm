import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Table } from '../Table';
import { ColumnDef } from '@tanstack/react-table';

// Mock TanStack React Table
const mockUseReactTable = vi.fn();
vi.mock('@tanstack/react-table', async () => {
  const actual = await vi.importActual('@tanstack/react-table');
  return {
    ...actual,
    useReactTable: (...args: any[]) => mockUseReactTable(...args),
    flexRender: vi.fn((template, context) => {
      if (typeof template === 'function') return template(context);
      return template;
    })
  };
});

// Mock react-icons
vi.mock('react-icons/lu', () => ({
  LuTriangle: (props: any) => <span data-testid="icon-sort-asc" {...props}>▲</span>, // Simplified mock for both directions as rotation is usually CSS
}));

describe('Table', () => {
  type TestData = {
    name: string;
    value: string;
  };

  const mockData: TestData[] = [
    { name: 'John', value: 'Developer' },
    { name: 'Jane', value: 'Designer' }
  ];

  const mockColumns: ColumnDef<TestData>[] = [
    {
      accessorKey: 'name',
      header: 'Name',
      size: 150
    },
    {
      accessorKey: 'value',
      header: 'Role'
    }
  ];

  const createMockTableInstance = (options: any = {}) => {
    const mockToggleSortingHandler = vi.fn();

    return {
      getHeaderGroups: vi.fn(() => [
        {
          id: 'header-group-1',
          headers: [
            {
              id: 'col-name',
              colSpan: 1,
              column: {
                getCanSort: () => options.sortable !== false,
                getToggleSortingHandler: () => mockToggleSortingHandler,
                getIsSorted: () => options.sortState || false,
                columnDef: { header: 'Name' },
                getSize: () => 150
              },
              isPlaceholder: false,
              getContext: () => ({}),
              getSize: () => 150
            },
            {
              id: 'col-value',
              colSpan: 1,
              column: {
                getCanSort: () => false,
                getToggleSortingHandler: () => vi.fn(),
                getIsSorted: () => false,
                columnDef: { header: 'Role' },
                getSize: () => 0
              },
              isPlaceholder: false,
              getContext: () => ({}),
              getSize: () => 0
            }
          ]
        }
      ]),
      getRowModel: vi.fn(() => ({
        rows: options.rows || [
          {
            id: 'row-1',
            original: mockData[0],
            getVisibleCells: () => [
              {
                id: 'cell-1-name',
                column: { getSize: () => 150, columnDef: { cell: 'John' } },
                getContext: () => ({})
              },
              {
                id: 'cell-1-value',
                column: { getSize: () => 0, columnDef: { cell: 'Developer' } },
                getContext: () => ({})
              }
            ],
            getIsExpanded: () => options.expanded || false
          },
          {
            id: 'row-2',
            original: mockData[1],
            getVisibleCells: () => [
              {
                id: 'cell-2-name',
                column: { getSize: () => 150, columnDef: { cell: 'Jane' } },
                getContext: () => ({})
              },
              {
                id: 'cell-2-value',
                column: { getSize: () => 0, columnDef: { cell: 'Designer' } },
                getContext: () => ({})
              }
            ],
            getIsExpanded: () => false
          }
        ]
      })),
      toggleSortingHandler: mockToggleSortingHandler
    };
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Table structure rendering', () => {
    it('should render table with basic structure', () => {
      mockUseReactTable.mockReturnValue(createMockTableInstance());
      const { container } = render(<Table data={mockData} columns={mockColumns} />);

      expect(container.querySelector('table')).toBeInTheDocument();
      expect(container.querySelector('thead')).toBeInTheDocument();
      expect(container.querySelector('tbody')).toBeInTheDocument();
    });
  });

  describe('Header rendering', () => {
    it('should render all column headers with text', () => {
      mockUseReactTable.mockReturnValue(createMockTableInstance());
      const { container } = render(<Table data={mockData} columns={mockColumns} />);

      const headers = container.querySelectorAll('thead th');
      expect(headers).toHaveLength(2);
      expect(container.querySelector('thead')).toHaveTextContent('Name');
      expect(container.querySelector('thead')).toHaveTextContent('Role');
    });
  });

  describe('Column sorting', () => {
    it('should show ascending icon when column sorted asc', () => {
      mockUseReactTable.mockReturnValue(createMockTableInstance({ sortState: 'asc' }));
      render(<Table data={mockData} columns={mockColumns} />);

      expect(screen.getByTestId('icon-sort-asc')).toBeInTheDocument();
    });

    it('should show descending icon when column sorted desc', () => {
      mockUseReactTable.mockReturnValue(createMockTableInstance({ sortState: 'desc' }));
      render(<Table data={mockData} columns={mockColumns} />);

      expect(screen.getByTestId('icon-sort-desc')).toBeInTheDocument();
    });

    it('should not show sort icon when column not sorted', () => {
      mockUseReactTable.mockReturnValue(createMockTableInstance({ sortState: false }));
      render(<Table data={mockData} columns={mockColumns} />);

      expect(screen.queryByTestId('icon-sort-asc')).not.toBeInTheDocument();
      expect(screen.queryByTestId('icon-sort-desc')).not.toBeInTheDocument();
    });

    it('should call toggle sorting handler on header click', () => {
      const mockInstance = createMockTableInstance({ sortable: true });
      mockUseReactTable.mockReturnValue(mockInstance);
      const { container } = render(<Table data={mockData} columns={mockColumns} />);

      const sortableHeader = container.querySelector('thead th');
      fireEvent.click(sortableHeader!);

      expect(mockInstance.toggleSortingHandler).toHaveBeenCalled();
    });

    it('should set column width from getSize()', () => {
      mockUseReactTable.mockReturnValue(createMockTableInstance());
      const { container } = render(<Table data={mockData} columns={mockColumns} />);

      const firstHeader = container.querySelector('thead th');
      expect(firstHeader).toHaveStyle({ width: '150px' });
    });
  });

  describe('Data row rendering', () => {
    it('should render rows with cell data', () => {
      mockUseReactTable.mockReturnValue(createMockTableInstance());
      const { container } = render(<Table data={mockData} columns={mockColumns} />);

      const rows = container.querySelectorAll('tbody tr');
      expect(rows).toHaveLength(2);

      const firstRow = container.querySelector('tbody tr');
      const cells = firstRow?.querySelectorAll('td');
      expect(cells).toHaveLength(2);

      const tbody = container.querySelector('tbody');
      expect(tbody).toHaveTextContent('John');
      expect(tbody).toHaveTextContent('Developer');
    });
  });

  describe('Row click handling', () => {
    it('should call onRowClick when row is clicked', async () => {
      const user = userEvent.setup();
      const mockOnRowClick = vi.fn();
      mockUseReactTable.mockReturnValue(createMockTableInstance());

      const { container } = render(
        <Table data={mockData} columns={mockColumns} onRowClick={mockOnRowClick} />
      );

      const firstRow = container.querySelector('tbody tr');
      await user.click(firstRow!);

      expect(mockOnRowClick).toHaveBeenCalled();
    });

    it('should call onRowClick on Enter key press', () => {
      const mockOnRowClick = vi.fn();
      mockUseReactTable.mockReturnValue(createMockTableInstance());

      const { container } = render(
        <Table data={mockData} columns={mockColumns} onRowClick={mockOnRowClick} />
      );

      const firstRow = container.querySelector('tbody tr');
      fireEvent.keyDown(firstRow!, { key: 'Enter' });

      expect(mockOnRowClick).toHaveBeenCalled();
    });

    it('should not call onRowClick on other key press', () => {
      const mockOnRowClick = vi.fn();
      mockUseReactTable.mockReturnValue(createMockTableInstance());

      const { container } = render(
        <Table data={mockData} columns={mockColumns} onRowClick={mockOnRowClick} />
      );

      const firstRow = container.querySelector('tbody tr');
      fireEvent.keyDown(firstRow!, { key: 'Space' });

      expect(mockOnRowClick).not.toHaveBeenCalled();
    });
  });

  describe('Row expansion', () => {
    it('should render sub-component when row is expanded', () => {
      const mockRenderSubComponent = vi.fn(({ row }) => (
        <div data-testid="sub-component">Expanded: {row.original.name}</div>
      ));

      mockUseReactTable.mockReturnValue(createMockTableInstance({ expanded: true }));

      render(
        <Table
          data={mockData}
          columns={mockColumns}
          renderSubComponent={mockRenderSubComponent}
        />
      );

      expect(screen.getByTestId('sub-component')).toBeInTheDocument();
      expect(mockRenderSubComponent).toHaveBeenCalled();
    });

    it('should render sub-component in full-width cell', () => {
      const mockRenderSubComponent = vi.fn(({ row }) => (
        <div data-testid="sub-component">Expanded content</div>
      ));

      mockUseReactTable.mockReturnValue(createMockTableInstance({ expanded: true }));

      const { container } = render(
        <Table
          data={mockData}
          columns={mockColumns}
          renderSubComponent={mockRenderSubComponent}
        />
      );

      const expandedRow = container.querySelectorAll('tbody tr')[1];
      const cell = expandedRow?.querySelector('td');
      expect(cell).toHaveAttribute('colSpan', '2');
    });

    it('should not render sub-component when row not expanded', () => {
      const mockRenderSubComponent = vi.fn(({ row }) => (
        <div data-testid="sub-component">Expanded content</div>
      ));

      mockUseReactTable.mockReturnValue(createMockTableInstance({ expanded: false }));

      render(
        <Table
          data={mockData}
          columns={mockColumns}
          renderSubComponent={mockRenderSubComponent}
        />
      );

      expect(screen.queryByTestId('sub-component')).not.toBeInTheDocument();
    });
  });

  describe('Empty data handling', () => {
    it('should render empty tbody when no data', () => {
      mockUseReactTable.mockReturnValue(createMockTableInstance({ rows: [] }));

      const { container } = render(<Table data={[]} columns={mockColumns} />);

      const tbody = container.querySelector('tbody');
      expect(tbody).toBeInTheDocument();
      expect(tbody?.querySelectorAll('tr')).toHaveLength(0);
    });
  });
});
