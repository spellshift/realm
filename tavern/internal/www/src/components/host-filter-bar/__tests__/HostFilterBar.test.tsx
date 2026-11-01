import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { HostFilterBar } from '../HostFilterBar';
import { convertLegacyBeaconFilters } from '../constants';
import { FilterBarOption, HostFilterRow } from '../../../utils/interfacesUI';

vi.mock('../../beacon-filter-bar/useBeaconFilterBar', () => ({
  useBeaconFilterBar: () => ({
    isLoading: false,
    error: undefined,
    options: [
      {
        label: 'Platform',
        options: [
          { id: 'PLATFORM_WINDOWS', name: 'PLATFORM_WINDOWS', value: 'PLATFORM_WINDOWS', label: 'Windows', kind: 'platform' },
          { id: 'PLATFORM_LINUX', name: 'PLATFORM_LINUX', value: 'PLATFORM_LINUX', label: 'Linux', kind: 'platform' },
        ],
      },
      {
        label: 'Transport',
        options: [
          { id: 'TRANSPORT_GRPC', name: 'TRANSPORT_GRPC', value: 'TRANSPORT_GRPC', label: 'GRPC', kind: 'transport' },
        ],
      },
      {
        label: 'Status',
        options: [
          { id: 'onlineBeacons', name: 'Online beacons', value: 'onlineBeacons', label: 'Online beacons', kind: 'onlineOfflineStatus' },
        ],
      },
      { label: 'Service', options: [] },
      { label: 'Group', options: [] },
      {
        label: 'Principal',
        options: [
          { id: 'root', name: 'root', value: 'root', label: 'root', kind: 'principal' },
          { id: 'admin', name: 'admin', value: 'admin', label: 'admin', kind: 'principal' },
        ],
      },
      { label: 'PrimaryIPs', options: [] },
      { label: 'ExternalIPs', options: [] },
      { label: 'Host', options: [] },
      { label: 'Beacon', options: [] },
    ],
  }),
}));

const platformRow = (overrides: Partial<HostFilterRow> = {}): HostFilterRow => ({
  id: 'row-1',
  field: 'platform',
  predicate: 'is',
  values: [],
  ...overrides,
});

describe('HostFilterBar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders one row per filter with an add button', () => {
    const onChange = vi.fn();
    render(<HostFilterBar value={[platformRow(), platformRow({ id: 'row-2', field: 'transport' })]} onChange={onChange} />);

    expect(screen.getByTestId('host-filter-bar')).toBeInTheDocument();
    expect(screen.getByTestId('host-filter-row-0')).toBeInTheDocument();
    expect(screen.getByTestId('host-filter-row-1')).toBeInTheDocument();
    expect(screen.getByTestId('host-filter-add')).toBeInTheDocument();
  });

  it('adds a default row when Add filter is clicked', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<HostFilterBar value={[platformRow()]} onChange={onChange} />);

    await user.click(screen.getByTestId('host-filter-add'));

    expect(onChange).toHaveBeenCalledTimes(1);
    const rows = onChange.mock.calls[0][0] as HostFilterRow[];
    expect(rows).toHaveLength(2);
    expect(rows[1]).toMatchObject({ field: 'platform', predicate: 'is', values: [] });
    expect(typeof rows[1].id).toBe('string');
  });

  it('removes the row when its remove button is clicked', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <HostFilterBar
        value={[platformRow(), platformRow({ id: 'row-2', field: 'transport' })]}
        onChange={onChange}
      />
    );

    await user.click(screen.getByTestId('host-filter-remove-0'));

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange.mock.calls[0][0]).toEqual([platformRow({ id: 'row-2', field: 'transport' })]);
  });

  it('resets values when the field changes', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<HostFilterBar value={[platformRow({ values: ['PLATFORM_WINDOWS'] })]} onChange={onChange} />);

    const fieldScope = within(screen.getByTestId('host-filter-field-0'));
    await user.click(fieldScope.getByRole('combobox'));
    await user.click(await screen.findByText('Transport'));

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange.mock.calls[0][0]).toEqual([
      { id: 'row-1', field: 'transport', predicate: 'is', values: [] },
    ]);
  });

  it('renders a free text input with suggestions for contains predicates', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<HostFilterBar value={[{ id: 'row-1', field: 'principal', predicate: 'contains', values: [] }]} onChange={onChange} />);

    // Note: an input with a datalist exposes the combobox role, not textbox
    const input = screen.getByRole('combobox', { name: 'Filter value' });
    expect(input).toHaveAttribute('placeholder', 'Type to filter…');

    const datalist = document.getElementById('host-filter-row-1-value-suggestions');
    expect(datalist).not.toBeNull();
    expect(datalist?.querySelectorAll('option')).toHaveLength(2);

    await user.type(input, 'adm');
    await user.tab();

    expect(onChange).toHaveBeenCalledWith([
      { id: 'row-1', field: 'principal', predicate: 'contains', values: ['adm'] },
    ]);
  });

  it('updates values when an enum value is picked', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<HostFilterBar value={[platformRow()]} onChange={onChange} />);

    const valueScope = within(screen.getByTestId('host-filter-value-0'));
    await user.click(valueScope.getByRole('combobox'));
    await user.click(await screen.findByText('Linux'));

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange.mock.calls[0][0]).toEqual([
      { id: 'row-1', field: 'platform', predicate: 'is', values: ['PLATFORM_LINUX'] },
    ]);
  });
});

describe('convertLegacyBeaconFilters', () => {
  const option = (kind: string, name: string, value?: string): FilterBarOption => ({
    id: value ?? name,
    name,
    value: value ?? name,
    label: name,
    kind,
  });

  it('converts each option group into a row with matching values', () => {
    const rows = convertLegacyBeaconFilters([
      option('platform', 'PLATFORM_LINUX'),
      option('transport', 'TRANSPORT_GRPC'),
      option('service', 'web'),
      option('host', 'server1', 'host-1'),
      option('beacon', 'beacon1'),
    ]);

    expect(rows).toEqual([
      { id: 'platform-0', field: 'platform', predicate: 'is', values: ['PLATFORM_LINUX'] },
      { id: 'transport-1', field: 'transport', predicate: 'is', values: ['TRANSPORT_GRPC'] },
      { id: 'service-2', field: 'service', predicate: 'is', values: ['web'] },
      { id: 'host-3', field: 'host', predicate: 'is', values: ['host-1'] },
      { id: 'beacon-4', field: 'beacon', predicate: 'is', values: ['beacon1'] },
    ]);
  });

  it('uses is-in for multi-value groups and skips status options', () => {
    const rows = convertLegacyBeaconFilters([
      option('principal', 'root'),
      option('principal', 'admin'),
      { kind: 'onlineOfflineStatus', id: 'onlineBeacons', name: 'Online beacons', value: 'onlineBeacons' },
    ]);

    expect(rows).toEqual([
      { id: 'principal-0', field: 'principal', predicate: 'is-in', values: ['root', 'admin'] },
    ]);
  });

  it('returns no rows for empty input', () => {
    expect(convertLegacyBeaconFilters([])).toEqual([]);
  });
});
