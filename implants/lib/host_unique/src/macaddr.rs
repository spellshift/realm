use uuid::Uuid;

use crate::HostIDSelector;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct MacAddr {}

impl MacAddr {
    /// Returns the first non-zero MAC address from a sorted list of network interfaces.
    fn get_mac_bytes(&self) -> Option<[u8; 6]> {
        let mut interfaces = netstat::list_interfaces().ok()?;
        interfaces.sort_by(|a, b| a.iface_name.cmp(&b.iface_name));

        for iface in interfaces {
            if iface.mac_address != [0u8; 6] {
                return Some(iface.mac_address);
            }
        }
        None
    }
}

impl HostIDSelector for MacAddr {
    fn get_name(&self) -> String {
        "macaddr".to_string()
    }

    fn get_host_id(&self) -> Option<Uuid> {
        let mac_bytes = self.get_mac_bytes()?;
        Some(Uuid::new_v5(&Uuid::NAMESPACE_OID, &mac_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mac_addr_deterministic() {
        let selector = MacAddr::default();
        let id_one = selector.get_host_id();
        let id_two = selector.get_host_id();

        assert!(id_one.is_some(), "expected a MAC-based host id");
        assert_eq!(id_one, id_two, "MAC-based host id must be deterministic");
    }

    #[test]
    fn test_mac_addr_is_v5_uuid() {
        let selector = MacAddr::default();
        if let Some(id) = selector.get_host_id() {
            let s = id.to_string();
            // UUID v5 has the version nibble '5' as the 13th hex character
            assert!(s.contains("-5"), "expected a v5 UUID, got: {}", s);
        }
    }
}
