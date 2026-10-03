//! Windows TCP listener discovery using the documented IP Helper API.

use std::mem::{offset_of, size_of};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::num::NonZeroU16;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{
    ERROR_ACCESS_DENIED, ERROR_INSUFFICIENT_BUFFER, ERROR_NOT_SUPPORTED, FALSE, NO_ERROR,
};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCP6TABLE_OWNER_PID, MIB_TCPROW_OWNER_PID,
    MIB_TCPTABLE_OWNER_PID, MIB_TCP_STATE_LISTEN, TCP_TABLE_OWNER_PID_LISTENER,
};
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};

use crate::domain::network::{NetworkListener, NetworkProtocol};
use crate::domain::port_provider::{
    PortProvider, PortProviderError, PortProviderErrorKind, PortScanCompleteness, PortScanResult,
};
use crate::domain::process::ProcessId;

// 16 MiB holds hundreds of thousands of rows at current MIB row sizes, far
// beyond a normal workstation table, while bounding native and output memory.
const MAX_TABLE_BYTES: usize = 16 * 1024 * 1024;
const MAX_RETRIEVAL_ATTEMPTS: usize = 3;
const WORD_BYTES: usize = size_of::<u64>();

#[derive(Debug, Clone)]
struct FamilyScan {
    listeners: Vec<NetworkListener>,
    partial: Option<PortProviderErrorKind>,
}

/// Windows implementation of the shared TCP listener provider contract.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsPortProvider;

impl PortProvider for WindowsPortProvider {
    fn listeners(&self) -> Result<PortScanResult, PortProviderError> {
        combine_family_scans(query_ipv4(), query_ipv6())
    }
}

fn combine_family_scans(
    ipv4: Result<FamilyScan, PortProviderErrorKind>,
    ipv6: Result<FamilyScan, PortProviderErrorKind>,
) -> Result<PortScanResult, PortProviderError> {
    match (ipv4, ipv6) {
        (Err(ipv4_error), Err(_)) => Err(PortProviderError::new(ipv4_error)),
        (Err(ipv4_error), Ok(ipv6)) => Ok(PortScanResult {
            listeners: ipv6.listeners,
            completeness: PortScanCompleteness::Partial(ipv4_error),
        }),
        (Ok(ipv4), Err(ipv6_error)) => Ok(PortScanResult {
            listeners: ipv4.listeners,
            completeness: PortScanCompleteness::Partial(ipv6_error),
        }),
        (Ok(ipv4), Ok(ipv6)) => {
            let completeness = ipv4.partial.or(ipv6.partial).map_or(
                PortScanCompleteness::Complete,
                PortScanCompleteness::Partial,
            );
            let mut listeners = ipv4.listeners;
            listeners.extend(ipv6.listeners);
            Ok(PortScanResult {
                listeners,
                completeness,
            })
        }
    }
}

fn query_ipv4() -> Result<FamilyScan, PortProviderErrorKind> {
    query_table(u32::from(AF_INET), parse_ipv4_table)
}

fn query_ipv6() -> Result<FamilyScan, PortProviderErrorKind> {
    query_table(u32::from(AF_INET6), parse_ipv6_table)
}

fn query_table<T>(
    address_family: u32,
    parse: impl Fn(&[u8]) -> Result<T, PortProviderErrorKind>,
) -> Result<T, PortProviderErrorKind> {
    let mut required_size = 0_u32;

    // SAFETY: a null output pointer is the documented sizing query. The size
    // pointer is valid for the duration of the call, and all other arguments
    // are fixed provider-controlled values.
    let sizing_status = unsafe {
        GetExtendedTcpTable(
            null_mut(),
            &mut required_size,
            FALSE,
            address_family,
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        )
    };

    if sizing_status != ERROR_INSUFFICIENT_BUFFER {
        return Err(map_api_error(sizing_status));
    }
    validate_required_size(required_size)?;

    for _ in 0..MAX_RETRIEVAL_ATTEMPTS {
        let mut storage = allocate_aligned_table(required_size)?;
        let capacity_bytes = storage
            .len()
            .checked_mul(WORD_BYTES)
            .ok_or(PortProviderErrorKind::ProviderFailure)?;
        let mut returned_size =
            u32::try_from(capacity_bytes).map_err(|_| PortProviderErrorKind::ProviderFailure)?;

        // SAFETY: `storage` is initialized, has u64 alignment (the returned
        // MIB structures require at most DWORD alignment), and `returned_size`
        // is exactly its byte length. The API receives only that writable
        // region and a live size pointer. The output is parsed only after a
        // successful status and after its reported size is checked below.
        let status = unsafe {
            GetExtendedTcpTable(
                storage.as_mut_ptr().cast(),
                &mut returned_size,
                FALSE,
                address_family,
                TCP_TABLE_OWNER_PID_LISTENER,
                0,
            )
        };

        if status == NO_ERROR {
            let used_bytes =
                usize::try_from(returned_size).map_err(|_| PortProviderErrorKind::ParseFailure)?;
            if used_bytes > capacity_bytes || used_bytes < size_of::<u32>() {
                return Err(PortProviderErrorKind::ParseFailure);
            }

            // SAFETY: the vector owns `capacity_bytes` initialized bytes and
            // `used_bytes` was validated to be within that allocation. The
            // resulting view cannot outlive `storage`; `parse` is called
            // synchronously before the allocation is dropped.
            let bytes =
                unsafe { std::slice::from_raw_parts(storage.as_ptr().cast::<u8>(), used_bytes) };
            return parse(bytes);
        }

        if status == ERROR_INSUFFICIENT_BUFFER {
            validate_required_size(returned_size)?;
            required_size = returned_size;
            continue;
        }

        return Err(map_api_error(status));
    }

    Err(PortProviderErrorKind::OperatingSystemFailure)
}

fn validate_required_size(size: u32) -> Result<usize, PortProviderErrorKind> {
    let size = usize::try_from(size).map_err(|_| PortProviderErrorKind::ProviderFailure)?;
    if size < size_of::<u32>() || size > MAX_TABLE_BYTES {
        return Err(PortProviderErrorKind::ProviderFailure);
    }
    Ok(size)
}

fn allocate_aligned_table(required_bytes: u32) -> Result<Vec<u64>, PortProviderErrorKind> {
    let required_bytes = validate_required_size(required_bytes)?;
    let word_count = required_bytes
        .checked_add(WORD_BYTES - 1)
        .ok_or(PortProviderErrorKind::ProviderFailure)?
        / WORD_BYTES;
    let rounded_bytes = word_count
        .checked_mul(WORD_BYTES)
        .ok_or(PortProviderErrorKind::ProviderFailure)?;
    if rounded_bytes > MAX_TABLE_BYTES {
        return Err(PortProviderErrorKind::ProviderFailure);
    }

    let word_count = rounded_bytes / WORD_BYTES;
    let mut storage = Vec::new();
    storage
        .try_reserve_exact(word_count)
        .map_err(|_| PortProviderErrorKind::ProviderFailure)?;
    storage.resize(word_count, 0);
    Ok(storage)
}

fn map_api_error(status: u32) -> PortProviderErrorKind {
    match status {
        ERROR_ACCESS_DENIED => PortProviderErrorKind::PermissionDenied,
        ERROR_NOT_SUPPORTED => PortProviderErrorKind::Unsupported,
        _ => PortProviderErrorKind::OperatingSystemFailure,
    }
}

fn parse_ipv4_table(bytes: &[u8]) -> Result<FamilyScan, PortProviderErrorKind> {
    let layout = checked_table_layout(
        bytes.len(),
        offset_of!(MIB_TCPTABLE_OWNER_PID, table),
        size_of::<MIB_TCPROW_OWNER_PID>(),
        read_u32_ne(bytes, offset_of!(MIB_TCPTABLE_OWNER_PID, dwNumEntries))? as usize,
    )?;

    let mut listeners = reserve_listener_rows(layout.entry_count)?;
    for index in 0..layout.entry_count {
        let row = layout.row(bytes, index)?;
        let state = read_u32_ne(row, offset_of!(MIB_TCPROW_OWNER_PID, dwState))?;
        if state != MIB_TCP_STATE_LISTEN as u32 {
            return Err(PortProviderErrorKind::ParseFailure);
        }

        let raw_address = read_u32_ne(row, offset_of!(MIB_TCPROW_OWNER_PID, dwLocalAddr))?;
        let address = Ipv4Addr::from(raw_address.to_ne_bytes());
        let raw_port = read_u32_ne(row, offset_of!(MIB_TCPROW_OWNER_PID, dwLocalPort))?;
        let port = decode_network_port(raw_port)?;
        let owner_pid = read_u32_ne(row, offset_of!(MIB_TCPROW_OWNER_PID, dwOwningPid))?;

        listeners.push(NetworkListener {
            protocol: NetworkProtocol::Tcp,
            local_address: Some(IpAddr::V4(address)),
            local_port: port,
            owner_pid: Some(ProcessId::new(owner_pid)),
        });
    }

    Ok(FamilyScan {
        listeners,
        partial: None,
    })
}

fn parse_ipv6_table(bytes: &[u8]) -> Result<FamilyScan, PortProviderErrorKind> {
    let layout = checked_table_layout(
        bytes.len(),
        offset_of!(MIB_TCP6TABLE_OWNER_PID, table),
        size_of::<MIB_TCP6ROW_OWNER_PID>(),
        read_u32_ne(bytes, offset_of!(MIB_TCP6TABLE_OWNER_PID, dwNumEntries))? as usize,
    )?;

    let mut listeners = reserve_listener_rows(layout.entry_count)?;
    let mut partial = None;
    for index in 0..layout.entry_count {
        let row = layout.row(bytes, index)?;
        let state = read_u32_ne(row, offset_of!(MIB_TCP6ROW_OWNER_PID, dwState))?;
        if state != MIB_TCP_STATE_LISTEN as u32 {
            return Err(PortProviderErrorKind::ParseFailure);
        }

        let address_offset = offset_of!(MIB_TCP6ROW_OWNER_PID, ucLocalAddr);
        let address_end = address_offset
            .checked_add(16)
            .ok_or(PortProviderErrorKind::ParseFailure)?;
        let address_bytes: [u8; 16] = row
            .get(address_offset..address_end)
            .ok_or(PortProviderErrorKind::ParseFailure)?
            .try_into()
            .map_err(|_| PortProviderErrorKind::ParseFailure)?;
        let scope_id = read_u32_ne(row, offset_of!(MIB_TCP6ROW_OWNER_PID, dwLocalScopeId))?;
        let address = if scope_id == 0 {
            Some(IpAddr::V6(Ipv6Addr::from(address_bytes)))
        } else {
            partial = Some(PortProviderErrorKind::Unsupported);
            None
        };

        let raw_port = read_u32_ne(row, offset_of!(MIB_TCP6ROW_OWNER_PID, dwLocalPort))?;
        let port = decode_network_port(raw_port)?;
        let owner_pid = read_u32_ne(row, offset_of!(MIB_TCP6ROW_OWNER_PID, dwOwningPid))?;

        listeners.push(NetworkListener {
            protocol: NetworkProtocol::Tcp,
            local_address: address,
            local_port: port,
            owner_pid: Some(ProcessId::new(owner_pid)),
        });
    }

    Ok(FamilyScan { listeners, partial })
}

#[derive(Debug, Clone, Copy)]
struct TableLayout {
    rows_offset: usize,
    row_size: usize,
    entry_count: usize,
}

impl TableLayout {
    fn row(self, bytes: &[u8], index: usize) -> Result<&[u8], PortProviderErrorKind> {
        let start = index
            .checked_mul(self.row_size)
            .and_then(|offset| self.rows_offset.checked_add(offset))
            .ok_or(PortProviderErrorKind::ParseFailure)?;
        let end = start
            .checked_add(self.row_size)
            .ok_or(PortProviderErrorKind::ParseFailure)?;
        bytes
            .get(start..end)
            .ok_or(PortProviderErrorKind::ParseFailure)
    }
}

fn checked_table_layout(
    data_len: usize,
    rows_offset: usize,
    row_size: usize,
    entry_count: usize,
) -> Result<TableLayout, PortProviderErrorKind> {
    let rows_bytes = entry_count
        .checked_mul(row_size)
        .ok_or(PortProviderErrorKind::ParseFailure)?;
    let required_len = rows_offset
        .checked_add(rows_bytes)
        .ok_or(PortProviderErrorKind::ParseFailure)?;
    if row_size == 0 || rows_offset > data_len || required_len > data_len {
        return Err(PortProviderErrorKind::ParseFailure);
    }
    Ok(TableLayout {
        rows_offset,
        row_size,
        entry_count,
    })
}

fn reserve_listener_rows(count: usize) -> Result<Vec<NetworkListener>, PortProviderErrorKind> {
    let mut listeners = Vec::new();
    listeners
        .try_reserve_exact(count)
        .map_err(|_| PortProviderErrorKind::ProviderFailure)?;
    Ok(listeners)
}

fn read_u32_ne(bytes: &[u8], offset: usize) -> Result<u32, PortProviderErrorKind> {
    let end = offset
        .checked_add(size_of::<u32>())
        .ok_or(PortProviderErrorKind::ParseFailure)?;
    let field: [u8; 4] = bytes
        .get(offset..end)
        .ok_or(PortProviderErrorKind::ParseFailure)?
        .try_into()
        .map_err(|_| PortProviderErrorKind::ParseFailure)?;
    Ok(u32::from_ne_bytes(field))
}

fn decode_network_port(raw_port: u32) -> Result<NonZeroU16, PortProviderErrorKind> {
    let memory_bytes = raw_port.to_ne_bytes();
    let port = u16::from_be_bytes([memory_bytes[0], memory_bytes[1]]);
    NonZeroU16::new(port).ok_or(PortProviderErrorKind::ParseFailure)
}

#[cfg(test)]
mod tests {
    use super::{
        allocate_aligned_table, checked_table_layout, combine_family_scans, decode_network_port,
        map_api_error, parse_ipv4_table, parse_ipv6_table, read_u32_ne, MAX_TABLE_BYTES,
    };
    use std::mem::{offset_of, size_of};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::num::NonZeroU16;
    use windows_sys::Win32::Foundation::{
        ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER, ERROR_NOT_SUPPORTED,
    };
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        MIB_TCP6ROW_OWNER_PID, MIB_TCP6TABLE_OWNER_PID, MIB_TCPROW_OWNER_PID,
        MIB_TCPTABLE_OWNER_PID, MIB_TCP_STATE_LISTEN,
    };

    fn set_u32_ne(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
    }

    fn ipv4_fixture(address: Ipv4Addr, port: u16, pid: u32) -> Vec<u8> {
        let rows_offset = offset_of!(MIB_TCPTABLE_OWNER_PID, table);
        let row_size = size_of::<MIB_TCPROW_OWNER_PID>();
        let mut bytes = vec![0_u8; rows_offset + row_size];
        set_u32_ne(
            &mut bytes,
            offset_of!(MIB_TCPTABLE_OWNER_PID, dwNumEntries),
            1,
        );
        let row = rows_offset;
        set_u32_ne(
            &mut bytes,
            row + offset_of!(MIB_TCPROW_OWNER_PID, dwState),
            MIB_TCP_STATE_LISTEN as u32,
        );
        set_u32_ne(
            &mut bytes,
            row + offset_of!(MIB_TCPROW_OWNER_PID, dwLocalAddr),
            u32::from_ne_bytes(address.octets()),
        );
        let mut port_memory = [0_u8; 4];
        port_memory[..2].copy_from_slice(&port.to_be_bytes());
        set_u32_ne(
            &mut bytes,
            row + offset_of!(MIB_TCPROW_OWNER_PID, dwLocalPort),
            u32::from_ne_bytes(port_memory),
        );
        set_u32_ne(
            &mut bytes,
            row + offset_of!(MIB_TCPROW_OWNER_PID, dwOwningPid),
            pid,
        );
        bytes
    }

    fn ipv6_fixture(address: Ipv6Addr, port: u16, pid: u32, scope: u32) -> Vec<u8> {
        let rows_offset = offset_of!(MIB_TCP6TABLE_OWNER_PID, table);
        let row_size = size_of::<MIB_TCP6ROW_OWNER_PID>();
        let mut bytes = vec![0_u8; rows_offset + row_size];
        set_u32_ne(
            &mut bytes,
            offset_of!(MIB_TCP6TABLE_OWNER_PID, dwNumEntries),
            1,
        );
        let row = rows_offset;
        let addr_offset = row + offset_of!(MIB_TCP6ROW_OWNER_PID, ucLocalAddr);
        bytes[addr_offset..addr_offset + 16].copy_from_slice(&address.octets());
        set_u32_ne(
            &mut bytes,
            row + offset_of!(MIB_TCP6ROW_OWNER_PID, dwLocalScopeId),
            scope,
        );
        set_u32_ne(
            &mut bytes,
            row + offset_of!(MIB_TCP6ROW_OWNER_PID, dwLocalPort),
            u32::from_ne_bytes({
                let mut port_memory = [0_u8; 4];
                port_memory[..2].copy_from_slice(&port.to_be_bytes());
                port_memory
            }),
        );
        set_u32_ne(
            &mut bytes,
            row + offset_of!(MIB_TCP6ROW_OWNER_PID, dwState),
            MIB_TCP_STATE_LISTEN as u32,
        );
        set_u32_ne(
            &mut bytes,
            row + offset_of!(MIB_TCP6ROW_OWNER_PID, dwOwningPid),
            pid,
        );
        bytes
    }

    #[test]
    fn decodes_network_order_port_from_native_dword_memory() {
        let raw = u32::from_ne_bytes([0x12, 0x34, 0, 0]);
        assert_eq!(
            decode_network_port(raw),
            Ok(NonZeroU16::new(0x1234).unwrap())
        );
        assert_eq!(
            decode_network_port(0),
            Err(crate::domain::port_provider::PortProviderErrorKind::ParseFailure)
        );
    }

    #[test]
    fn converts_ipv4_loopback_and_wildcard_and_preserves_pid() {
        let loopback = parse_ipv4_table(&ipv4_fixture(Ipv4Addr::LOCALHOST, 43210, 77))
            .expect("valid IPv4 table");
        assert_eq!(
            loopback.listeners[0].local_address,
            Some(IpAddr::V4(Ipv4Addr::LOCALHOST))
        );
        assert_eq!(loopback.listeners[0].local_port.get(), 43210);
        assert_eq!(
            loopback.listeners[0].owner_pid.map(|pid| pid.get()),
            Some(77)
        );

        let wildcard = parse_ipv4_table(&ipv4_fixture(Ipv4Addr::UNSPECIFIED, 8080, 0))
            .expect("valid wildcard IPv4 table");
        assert_eq!(
            wildcard.listeners[0].local_address,
            Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
        );
        assert_eq!(
            wildcard.listeners[0].owner_pid.map(|pid| pid.get()),
            Some(0)
        );

        let specific = "192.0.2.14".parse().expect("valid IPv4 fixture");
        let result = parse_ipv4_table(&ipv4_fixture(specific, 8081, u32::MAX))
            .expect("valid specific IPv4 table");
        assert_eq!(
            result.listeners[0].local_address,
            Some(IpAddr::V4(specific))
        );
        assert_eq!(
            result.listeners[0].owner_pid.map(|pid| pid.get()),
            Some(u32::MAX)
        );
    }

    #[test]
    fn converts_ipv6_loopback_and_wildcard_binary_addresses() {
        let specific = "2001:db8::42".parse().expect("valid IPv6 fixture");
        for address in [Ipv6Addr::LOCALHOST, Ipv6Addr::UNSPECIFIED, specific] {
            let result =
                parse_ipv6_table(&ipv6_fixture(address, 12345, 9, 0)).expect("valid IPv6 table");
            assert_eq!(result.listeners[0].local_address, Some(IpAddr::V6(address)));
            assert_eq!(result.listeners[0].local_port.get(), 12345);
        }
    }

    #[test]
    fn scoped_ipv6_listener_is_retained_as_partial_unknown_address() {
        let result = parse_ipv6_table(&ipv6_fixture("fe80::1".parse().unwrap(), 80, 10, 7))
            .expect("scoped address remains a listener");
        assert_eq!(result.listeners[0].local_address, None);
        assert_eq!(result.listeners[0].owner_pid.map(|pid| pid.get()), Some(10));
        assert_eq!(
            result.partial,
            Some(crate::domain::port_provider::PortProviderErrorKind::Unsupported)
        );
    }

    #[test]
    fn rejects_non_listening_state_and_truncated_or_overflowing_tables() {
        let mut fixture = ipv4_fixture(Ipv4Addr::LOCALHOST, 80, 10);
        fixture[0..4].copy_from_slice(&u32::MAX.to_ne_bytes());
        assert!(parse_ipv4_table(&fixture).is_err());
        assert!(parse_ipv4_table(&ipv4_fixture(Ipv4Addr::LOCALHOST, 80, 10)[..4]).is_err());
        assert!(checked_table_layout(usize::MAX, 4, 8, usize::MAX).is_err());
        assert!(checked_table_layout(8, 4, 8, 1).is_err());
    }

    #[test]
    fn preserves_multiple_rows_with_shared_port_and_distinct_addresses() {
        let first = ipv4_fixture(Ipv4Addr::LOCALHOST, 8080, 11);
        let second = ipv4_fixture(Ipv4Addr::UNSPECIFIED, 8080, 12);
        let rows_offset = offset_of!(MIB_TCPTABLE_OWNER_PID, table);
        let mut table = first.clone();
        table.extend_from_slice(&second[rows_offset..]);
        set_u32_ne(
            &mut table,
            offset_of!(MIB_TCPTABLE_OWNER_PID, dwNumEntries),
            2,
        );

        let scan = parse_ipv4_table(&table).expect("both listener rows are valid");
        assert_eq!(scan.listeners.len(), 2);
        assert_eq!(scan.listeners[0].local_port, scan.listeners[1].local_port);
        assert_ne!(
            scan.listeners[0].local_address,
            scan.listeners[1].local_address
        );
        assert_ne!(scan.listeners[0].owner_pid, scan.listeners[1].owner_pid);
    }

    #[test]
    fn empty_family_tables_are_valid_and_family_failures_are_explicit() {
        let empty = vec![0_u8; offset_of!(MIB_TCPTABLE_OWNER_PID, table)];
        let v4 = parse_ipv4_table(&empty).expect("zero-entry table is valid");
        let v6 = parse_ipv6_table(&vec![0_u8; offset_of!(MIB_TCP6TABLE_OWNER_PID, table)])
            .expect("zero-entry table is valid");
        let result = combine_family_scans(Ok(v4), Ok(v6)).expect("both families succeeded");
        assert!(result.listeners.is_empty());
        assert_eq!(
            result.completeness,
            crate::domain::port_provider::PortScanCompleteness::Complete
        );

        let v4_failure = crate::domain::port_provider::PortProviderErrorKind::PermissionDenied;
        let v6_failure = crate::domain::port_provider::PortProviderErrorKind::Unsupported;
        assert_eq!(
            combine_family_scans(Err(v4_failure), Err(v6_failure))
                .expect_err("two failed family queries have no usable scan")
                .kind(),
            v4_failure
        );
        let available = parse_ipv6_table(&ipv6_fixture(Ipv6Addr::LOCALHOST, 3000, 42, 0))
            .expect("one successful family contains a valid controlled row");
        let partial = combine_family_scans(Err(v4_failure), Ok(available))
            .expect("one successful family is a partial scan");
        assert_eq!(
            partial.completeness,
            crate::domain::port_provider::PortScanCompleteness::Partial(v4_failure)
        );
        assert_eq!(partial.listeners.len(), 1);
        assert_eq!(
            partial.listeners[0].owner_pid.map(|pid| pid.get()),
            Some(42)
        );
    }

    #[test]
    fn bounded_aligned_allocation_rejects_pathological_sizes() {
        let storage = allocate_aligned_table(13).expect("small aligned allocation");
        assert_eq!(storage.as_ptr() as usize % size_of::<u64>(), 0);
        assert!(storage.len() * size_of::<u64>() >= 13);
        assert!(allocate_aligned_table((MAX_TABLE_BYTES + 1) as u32).is_err());
    }

    #[test]
    fn rejects_zero_entry_port_and_non_listener_rows() {
        let mut zero_port = ipv4_fixture(Ipv4Addr::LOCALHOST, 80, 1);
        let port_offset = offset_of!(MIB_TCPTABLE_OWNER_PID, table)
            + offset_of!(MIB_TCPROW_OWNER_PID, dwLocalPort);
        set_u32_ne(&mut zero_port, port_offset, 0);
        assert!(parse_ipv4_table(&zero_port).is_err());

        let mut non_listener = ipv4_fixture(Ipv4Addr::LOCALHOST, 80, 1);
        let state_offset =
            offset_of!(MIB_TCPTABLE_OWNER_PID, table) + offset_of!(MIB_TCPROW_OWNER_PID, dwState);
        set_u32_ne(&mut non_listener, state_offset, 5);
        assert!(parse_ipv4_table(&non_listener).is_err());
    }

    #[test]
    fn reads_fields_only_when_the_entire_dword_is_present() {
        assert_eq!(
            read_u32_ne(&[1, 2, 3], 0),
            Err(crate::domain::port_provider::PortProviderErrorKind::ParseFailure)
        );
    }

    #[test]
    fn maps_win32_errors_to_stable_provider_categories() {
        assert_eq!(
            map_api_error(ERROR_ACCESS_DENIED),
            crate::domain::port_provider::PortProviderErrorKind::PermissionDenied
        );
        assert_eq!(
            map_api_error(ERROR_NOT_SUPPORTED),
            crate::domain::port_provider::PortProviderErrorKind::Unsupported
        );
        assert_eq!(
            map_api_error(ERROR_INVALID_PARAMETER),
            crate::domain::port_provider::PortProviderErrorKind::OperatingSystemFailure
        );
    }
}
