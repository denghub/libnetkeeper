use std::net::Ipv4Addr;

use rustc_serialize::hex::ToHex;
use openssl::crypto::hash::{Hasher, Type};

use utils::{current_timestamp, integer_to_bytes};

#[derive(Debug)]
pub struct Attribute {
    typename: String,
    parent_id: u8,
    type_id: u8,
    value_type_id: u8,
    data: Vec<u8>,
}

pub trait AttributeFactory {
    fn username(username: &str) -> Attribute;
    fn client_ip_address(ipaddress: Ipv4Addr) -> Attribute;
    fn client_type(client_type: &str) -> Attribute;
    fn client_version(client_version: &str) -> Attribute;
    fn os_version(version: &str) -> Attribute;
    fn os_language(language: &str) -> Attribute;
    fn cpu_info(cpu_info: &str) -> Attribute;
    fn mac_address(mac_address: &[u8; 4]) -> Attribute;
    fn memory_size(size: u32) -> Attribute;
    fn default_explorer(explorer: &str) -> Attribute;
    fn keepalive_data(data: &str) -> Attribute;
    fn keepalive_time(timestamp: u32) -> Attribute;

    fn calc_keepalive_data(timestamp: Option<u32>, last_data: Option<&str>) -> String;
}

pub trait AttributeVec {
    fn as_bytes(&self) -> Vec<u8>;
}

impl Attribute {
    pub fn new(typename: &str,
               parent_id: u8,
               type_id: u8,
               value_type_id: u8,
               data: Vec<u8>)
               -> Self {
        Attribute {
            typename: typename.to_string(),
            parent_id: parent_id,
            type_id: type_id,
            value_type_id: value_type_id,
            data: data,
        }
    }

    fn data_length(&self) -> u16 {
        self.data.len() as u16
    }

    pub fn length(&self) -> u16 {
        self.data_length() + 3
    }

    pub fn as_bytes(&self) -> Box<Vec<u8>> {
        let mut attribute_bytes: Box<Vec<u8>> = Box::new(Vec::new());
        {
            let length_be = self.length().to_be();
            let length_bytes = integer_to_bytes(&length_be);
            attribute_bytes.push(self.parent_id);
            attribute_bytes.extend(length_bytes);
            attribute_bytes.extend(&self.data);
        }
        attribute_bytes
    }
}

impl AttributeFactory for Attribute {
    fn username(username: &str) -> Attribute {
        Attribute::new("User-Name", 0x1, 0x0, 0x2, username.as_bytes().to_vec())
    }

    fn client_ip_address(ipaddress: Ipv4Addr) -> Attribute {
        Attribute::new("Client-IP-Address",
                       0x2,
                       0x0,
                       0x1,
                       ipaddress.octets().to_vec())
    }

    fn client_type(client_type: &str) -> Attribute {
        Attribute::new("Client-Type",
                       0x4,
                       0x0,
                       0x2,
                       client_type.as_bytes().to_vec())
    }

    fn client_version(client_version: &str) -> Attribute {
        Attribute::new("Client-Version",
                       0x3,
                       0x0,
                       0x2,
                       client_version.as_bytes().to_vec())
    }

    fn os_version(version: &str) -> Attribute {
        Attribute::new("OS-Version", 0x5, 0x0, 0x2, version.as_bytes().to_vec())
    }

    fn os_language(language: &str) -> Attribute {
        Attribute::new("OS-Lang", 0x6, 0x0, 0x2, language.as_bytes().to_vec())
    }

    fn cpu_info(cpu_info: &str) -> Attribute {
        Attribute::new("CPU-Info", 0x8, 0x0, 0x2, cpu_info.as_bytes().to_vec())
    }

    fn mac_address(mac_address: &[u8; 4]) -> Attribute {
        Attribute::new("MAC-Address", 0x9, 0x0, 0x2, mac_address.to_vec())
    }

    fn memory_size(size: u32) -> Attribute {
        let size_be = size.to_be();
        let size_bytes = integer_to_bytes(&size_be);
        Attribute::new("Memory-Size", 0xa, 0x0, 0x0, size_bytes.to_vec())
    }

    fn default_explorer(explorer: &str) -> Attribute {
        Attribute::new("Default-Explorer",
                       0xb,
                       0x0,
                       0x2,
                       explorer.as_bytes().to_vec())
    }

    fn keepalive_data(data: &str) -> Attribute {
        Attribute::new("KeepAlive-Data", 0x14, 0x0, 0x2, data.as_bytes().to_vec())
    }

    fn keepalive_time(timestamp: u32) -> Attribute {
        let timestamp_be = timestamp.to_be();
        let timestamp_bytes = integer_to_bytes(&timestamp_be);
        Attribute::new("KeepAlive-Time", 0x12, 0x0, 0x0, timestamp_bytes.to_vec())
    }

    fn calc_keepalive_data(timestamp: Option<u32>, last_data: Option<&str>) -> String {
        let timenow = match timestamp {
            Some(timestamp) => timestamp,
            None => current_timestamp(),
        };

        let salt = match last_data {
            Some(data) => data,
            None => "llwl",
        };

        let keepalive_data;
        {
            let mut md5 = Hasher::new(Type::MD5).unwrap();
            let timenow_be = timenow.to_be();
            let timenow_bytes = integer_to_bytes(&timenow_be);

            md5.update(timenow_bytes).unwrap();
            md5.update(salt.as_bytes()).unwrap();

            let hashed_bytes = md5.finish().unwrap();
            keepalive_data = hashed_bytes[..].to_hex();
        }
        keepalive_data
    }
}

impl AttributeVec for Vec<Attribute> {
    fn as_bytes(&self) -> Vec<u8> {
        let mut attributes_bytes: Vec<u8> = Vec::new();
        for attr in self {
            attributes_bytes.extend(*attr.as_bytes());
        }
        attributes_bytes
    }
}

#[test]
fn test_attribute_gen_bytes() {
    let un = Attribute::username("05802278989@HYXY.XY");
    let assert_data: &[u8] = &[1, 0, 22, 48, 53, 56, 48, 50, 50, 55, 56, 57, 56, 57, 64, 72, 89,
                               88, 89, 46, 88, 89];
    assert_eq!(&un.as_bytes()[..], assert_data);
}

#[test]
fn test_keepalive_data() {
    let kp_data1 = Attribute::calc_keepalive_data(Some(1472483020), None);
    let kp_data2 = Attribute::calc_keepalive_data(Some(1472483020),
                                                  Some("ffb0b2af94693fd1ba4c93e6b9aebd3f"));
    assert_eq!(kp_data1, "ffb0b2af94693fd1ba4c93e6b9aebd3f");
    assert_eq!(kp_data2, "d0dce2b013c8adfac646a2917fdab802");
}