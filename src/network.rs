use std::{
    fmt::Debug,
    net::{self, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs},
};

// use bytes::{Buf, BufMut, Bytes};
use epee_encoding::{
    io::{Read, Write},
    read_epee_value, write_field, EpeeObject, EpeeObjectBuilder,
};
use serde::{Deserialize, Serialize};

// use crate::error::MyError;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkAddressTypeEnum {
    INVALID = 0,
    IPV4 = 1,
    IPV6 = 2,
    I2P = 3,
    TOR = 4,
}

impl Default for NetworkAddressTypeEnum {
    fn default() -> Self {
        Self::INVALID
    }
}

impl NetworkAddressTypeEnum {
    pub fn from_u8(in_value: u8) -> NetworkAddressTypeEnum {
        match in_value {
            0 => NetworkAddressTypeEnum::INVALID,
            1 => NetworkAddressTypeEnum::IPV4,
            2 => NetworkAddressTypeEnum::IPV6,
            3 => NetworkAddressTypeEnum::I2P,
            4 => NetworkAddressTypeEnum::TOR,
            _ => NetworkAddressTypeEnum::INVALID,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            NetworkAddressTypeEnum::INVALID => 0,
            NetworkAddressTypeEnum::IPV4 => 1,
            NetworkAddressTypeEnum::IPV6 => 2,
            NetworkAddressTypeEnum::I2P => 3,
            NetworkAddressTypeEnum::TOR => 4,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NetZone {
    Public,
    Tor,
    I2p,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetworkAddress {
    pub addr_type: NetworkAddressTypeEnum,
    pub addr: [u8; 16],
    pub port: u16,
}

#[derive(Default, Debug)]
pub struct __NetworkAddress {
    pub addr_type: NetworkAddressTypeEnum,
    pub addr: Option<[u8; 16]>,
    pub port: Option<u16>,
}

impl EpeeObjectBuilder<NetworkAddress> for __NetworkAddress {
    fn add_field<R: Read>(&mut self, name: &str, r: &mut R) -> epee_encoding::error::Result<bool> {
        match name {
            "addr" => {
                self.addr = Some(read_epee_value(r)?);
            }
            _ => return Ok(false),
        }
        println!(" Add NetAddr. addr = {:x?}", self.addr);
        Ok(true)
    }

    fn finish(self) -> epee_encoding::error::Result<NetworkAddress> {
        println!(" Finishing NetAdd. addr = {:x?}", self.addr);

        Ok(NetworkAddress {
            addr_type: self.addr_type,
            addr: self.addr.ok_or_else(|| {
                println!(" Finishing NetAdd. addr = {:x?}", self.addr);
                epee_encoding::error::Error::Format("Required field was not found!")
            })?,
            port: self.port.unwrap().clone(),
        })
    }
}

impl EpeeObject for NetworkAddress {
    type Builder = TaggedNetworkAddress;

    fn number_of_fields(&self) -> u64 {
        2
    }

    fn write_fields<W: Write>(&self, w: &mut W) -> epee_encoding::error::Result<()> {
        // write the fields
        //write_field(&self.val, "val", w)
        println!(" TaggedNetworkAddress::from(self).write_fields(w) ");
        TaggedNetworkAddress::from(self).write_fields(w)
    }
}

impl NetworkAddress {
    pub fn get_zone(&self) -> NetZone {
        NetZone::Public
    }

    pub fn is_loopback(&self) -> bool {
        // TODO
        false
    }

    pub fn is_local(&self) -> bool {
        // TODO
        false
    }

    pub fn port(&self) -> u16 {
        // match self {
        //     NetworkAddress::Clear(ip) => ip.port(),
        // }
        self.port()
    }
}

impl From<net::SocketAddrV4> for NetworkAddress {
    fn from(value: net::SocketAddrV4) -> Self {
        //NetworkAddress::Clear(value.into())
        // value.into()

        let mut tmp_buffer: [u8; 16] = [0; 16];
        for i in 0..4 {
            tmp_buffer[i] = value.ip().octets()[i];
        }

        println!(" IPv4 value: {:?} ", value);

        NetworkAddress {
            addr_type: NetworkAddressTypeEnum::IPV4,
            addr: tmp_buffer,
            port: value.port(),
        }
    }
}

impl From<net::SocketAddrV6> for NetworkAddress {
    fn from(value: net::SocketAddrV6) -> Self {
        //NetworkAddress::Clear(value.into())
        // value.into()

        let mut tmp_buffer: [u8; 16] = [0; 16];
        for i in 0..16 {
            tmp_buffer[i] = value.ip().octets()[i];
        }

        println!(" IPv6 value: {:?} ", value);

        NetworkAddress {
            addr_type: NetworkAddressTypeEnum::IPV6,
            addr: tmp_buffer,
            port: value.port(),
        }
    }
}

impl From<SocketAddr> for NetworkAddress {
    fn from(value: SocketAddr) -> Self {
        match value {
            SocketAddr::V4(v4) => v4.into(),
            SocketAddr::V6(v6) => v6.into(),
        }
    }
}

#[derive(Default, Debug)]
pub struct TaggedNetworkAddress {
    pub ty: Option<u8>,
    pub addr: Option<AllFieldsNetworkAddress>,
}

#[derive(Default, Debug)]
pub struct __TaggedNetworkAddress {
    pub ty: Option<u8>,
    pub addr: Option<AllFieldsNetworkAddress>,
}

impl EpeeObjectBuilder<NetworkAddress> for TaggedNetworkAddress {
    fn add_field<B: epee_encoding::io::Read>(
        // + epee_encoding::io::Read>(
        &mut self,
        name: &str,
        b: &mut B,
    ) -> Result<bool, epee_encoding::Error> {
        println!(" EpeeObjectBuilder<NetworkAddress> : name {:?} ", name);

        match name {
            "type" => {
                if std::mem::replace(&mut self.ty, Some(epee_encoding::read_epee_value(b)?))
                    .is_some()
                {
                    return Err(epee_encoding::Error::Format(
                        "Duplicate field type in data.",
                    ));
                }
                Ok(true)
            }
            "addr" => {
                if std::mem::replace(&mut self.addr, epee_encoding::read_epee_value(b)?).is_some() {
                    return Err(epee_encoding::Error::Format(
                        "Duplicate field addr in data.",
                    ));
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn finish(self) -> Result<NetworkAddress, epee_encoding::Error> {
        println!(" EpeeObjectBuilder<NetworkAddress> : finish");
        self.try_into()
            .map_err(|_| epee_encoding::Error::Value("Invalid network address"))
    }
}

impl EpeeObjectBuilder<TaggedNetworkAddress> for __TaggedNetworkAddress {
    fn add_field<Buf: epee_encoding::io::Read>(
        &mut self,
        name: &str,
        b: &mut Buf,
    ) -> Result<bool, epee_encoding::Error> {
        println!(
            " EpeeObjectBuilder<TaggedNetworkAddress> : name {:?} ",
            name
        );

        match name {
            "type" => {
                if std::mem::replace(&mut self.ty, Some(epee_encoding::read_epee_value(b)?))
                    .is_some()
                {
                    return Err(epee_encoding::Error::Format("Duplicate field in data."));
                }
                Ok(true)
            }
            "addr" => {
                if std::mem::replace(&mut self.addr, epee_encoding::read_epee_value(b)?).is_some() {
                    return Err(epee_encoding::Error::Format("Duplicate field in data."));
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn finish(self) -> Result<TaggedNetworkAddress, epee_encoding::Error> {
        println!(" EpeeObjectBuilder<TaggedNetworkAddress> finish ");

        self.try_into()
            .map_err(|_| epee_encoding::Error::Value("Invalid network address"))
    }
}

impl EpeeObject for TaggedNetworkAddress {
    type Builder = __TaggedNetworkAddress;

    fn number_of_fields(&self) -> u64 {
        2
    }

    fn write_fields<W: Write>(&self, w: &mut W) -> epee_encoding::error::Result<()> {
        // write the fields
        println!(" EpeeObject for TaggedNetworkAddress  write_fields ");

        write_field(&self.ty, "type", w)?;
        write_field(&self.addr, "addr", w)
    }
}

impl TryFrom<__TaggedNetworkAddress> for TaggedNetworkAddress {
    fn try_from(value: __TaggedNetworkAddress) -> Result<Self, Self::Error> {
        println!(" TryFrom<__TaggedNetworkAddress>  value: {:?} ", value);

        Ok(TaggedNetworkAddress {
            ty: value.ty.clone(),
            addr: value.addr,
        })
    }

    type Error = epee_encoding::Error;
}
impl TryFrom<TaggedNetworkAddress> for NetworkAddress {
    type Error = epee_encoding::Error;

    fn try_from(value: TaggedNetworkAddress) -> Result<Self, Self::Error> {
        println!(
            " TryFrom<TaggedNetworkAddress>   try_from. value: {:?} ",
            value
        );

        value
            .addr
            .ok_or(epee_encoding::Error::Value("Invalid Network Address"))?
            .try_into_network_address(
                value
                    .ty
                    .ok_or(epee_encoding::Error::Value("Invalid Network Address"))?,
            )
            .ok_or(epee_encoding::Error::Value("Invalid Network Address"))
    }
}

impl TryFrom<__TaggedNetworkAddress> for NetworkAddress {
    type Error = epee_encoding::Error;

    fn try_from(value: __TaggedNetworkAddress) -> Result<Self, Self::Error> {
        println!(
            " TryFrom<__TaggedNetworkAddress>  try_from. value: {:?} ",
            value
        );

        value
            .addr
            .ok_or(epee_encoding::Error::Value("Invalid Network Address"))?
            .try_into_network_address(
                value
                    .ty
                    .ok_or(epee_encoding::Error::Value("Invalid Network Address"))?,
            )
            .ok_or(epee_encoding::Error::Value("Invalid Network Address"))
    }
}
impl From<&NetworkAddress> for TaggedNetworkAddress {
    fn from(value: &NetworkAddress) -> Self {
        println!(" From<&NetworkAddress> from. value: {:?} ", value);

        let mut tmp_ipv4: [u8; 4] = [0; 4];
        for i in 0..4 {
            tmp_ipv4[i] = value.addr[i];
        }
        match value.addr_type {
            // NetworkAddress { addr) => match addr {
            NetworkAddressTypeEnum::IPV4 => TaggedNetworkAddress {
                ty: Some(1),
                addr: Some(AllFieldsNetworkAddress {
                    m_ip: Some(u32::from_be_bytes(tmp_ipv4)),
                    m_port: Some(value.port()),
                    addr: None,
                }),
            },
            NetworkAddressTypeEnum::IPV6 => TaggedNetworkAddress {
                ty: Some(2),
                addr: Some(AllFieldsNetworkAddress {
                    addr: Some(value.addr),
                    m_port: Some(value.port()),
                    m_ip: None,
                }),
            },
            NetworkAddressTypeEnum::INVALID => todo!(),
            NetworkAddressTypeEnum::I2P => todo!(),
            NetworkAddressTypeEnum::TOR => todo!(),
            // },
        }
    }
}

impl From<NetworkAddress> for __TaggedNetworkAddress {
    fn from(value: NetworkAddress) -> Self {
        println!(" From<NetworkAddress> from. value: {:?} ", value);

        let mut tmp_ipv4: [u8; 4] = [0; 4];
        for i in 0..4 {
            tmp_ipv4[i] = value.addr[i];
        }

        match value.addr_type {
            // NetworkAddress { addr) => match addr {
            NetworkAddressTypeEnum::IPV4 => __TaggedNetworkAddress {
                ty: Some(1),
                addr: Some(AllFieldsNetworkAddress {
                    m_ip: Some(u32::from_be_bytes(tmp_ipv4)),
                    m_port: Some(value.port()),
                    addr: None,
                }),
            },
            NetworkAddressTypeEnum::IPV6 => __TaggedNetworkAddress {
                ty: Some(2),
                addr: Some(AllFieldsNetworkAddress {
                    addr: Some(value.addr),
                    m_port: Some(value.port()),
                    m_ip: None,
                }),
            },
            NetworkAddressTypeEnum::INVALID => todo!(),
            NetworkAddressTypeEnum::I2P => todo!(),
            NetworkAddressTypeEnum::TOR => todo!(),
            // },
        }
    }
}

#[derive(Default, Debug, EpeeObject)]
pub struct AllFieldsNetworkAddress {
    pub m_ip: Option<u32>,
    pub m_port: Option<u16>,
    pub addr: Option<[u8; 16]>,
}

impl AllFieldsNetworkAddress {
    fn try_into_network_address(self, ty: u8) -> Option<NetworkAddress> {
        println!(
            "  AllFieldsNetworkAddress  try_into_network_address: {:?} ",
            ty
        );

        Some(match ty {
            1 => NetworkAddress::from(SocketAddrV4::new(Ipv4Addr::from(self.m_ip?), self.m_port?)),
            2 => NetworkAddress::from(SocketAddrV6::new(
                Ipv6Addr::from(self.addr?),
                self.m_port?,
                0,
                0,
            )),
            _ => return None,
        })
    }
}

/*

// #[derive(MyError, Debug)]
// #[error("Invalid network address")]
pub struct InvalidNetworkAddress;

impl TryFrom<TaggedNetworkAddress> for NetworkAddress {
    type Error = InvalidNetworkAddress;

    fn try_from(value: TaggedNetworkAddress) -> Result<Self, Self::Error> {
        value
            .addr
            .ok_or(InvalidNetworkAddress)?
            .try_into_network_address(value.ty.ok_or(InvalidNetworkAddress)?)
            .ok_or(InvalidNetworkAddress)
    }
}

// #[derive(Debug, PartialEq, Serialize, Deserialize, EpeeObject)]
// pub struct IPv4NetworkAddress {
//     pub addr: u32,

//     pub m_ip: u32,
//     pub m_port: u16,
// }

#[derive(Default, EpeeObject)]
pub struct TaggedNetworkAddress {
    ty: Option<u8>,
    addr: Option<AllFieldsNetworkAddress>,
}

// epee_object!(
//     TaggedNetworkAddress,
//     ty("type"): Option<u8>,
//     addr: Option<AllFieldsNetworkAddress>,
// );

impl EpeeObjectBuilder<NetworkAddress> for TaggedNetworkAddress {
    fn add_field<B: Buf + epee_encoding::io::Read>(
        &mut self,
        name: &str,
        b: &mut B,
    ) -> Result<bool, epee_encoding::Error> {
        match name {
            "type" => {
                if std::mem::replace(&mut self.ty, Some(epee_encoding::read_epee_value(b)?))
                    .is_some()
                {
                    return Err(epee_encoding::Error::Format("Duplicate field in data."));
                }
                Ok(true)
            }
            "addr" => {
                if std::mem::replace(&mut self.addr, epee_encoding::read_epee_value(b)?).is_some() {
                    return Err(epee_encoding::Error::Format("Duplicate field in data."));
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn finish(self) -> Result<NetworkAddress, epee_encoding::Error> {
        self.try_into()
            .map_err(|_| epee_encoding::Error::Value("Invalid network address"))
    }
}

impl From<NetworkAddress> for TaggedNetworkAddress {
    fn from(value: NetworkAddress) -> Self {
        match value {
            NetworkAddress::Clear(addr) => match addr {
                SocketAddr::V4(addr) => TaggedNetworkAddress {
                    ty: Some(1),
                    addr: Some(AllFieldsNetworkAddress {
                        m_ip: Some(u32::from_be_bytes(addr.ip().octets())),
                        m_port: Some(addr.port()),
                        addr: None,
                    }),
                },
                SocketAddr::V6(addr) => TaggedNetworkAddress {
                    ty: Some(2),
                    addr: Some(AllFieldsNetworkAddress {
                        addr: Some(addr.ip().octets()),
                        m_port: Some(addr.port()),
                        m_ip: None,
                    }),
                },
            },
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, EpeeObject)]
struct AllFieldsNetworkAddress {
    pub m_ip: Option<u32>,
    pub m_port: Option<u16>,
    pub addr: Option<[u8; 16]>,
}

impl AllFieldsNetworkAddress {
    fn try_into_network_address(self, ty: u8) -> Option<NetworkAddress> {
        Some(match ty {
            1 => NetworkAddress::from(SocketAddrV4::new(Ipv4Addr::from(self.m_ip?), self.m_port?)),
            2 => NetworkAddress::from(SocketAddrV6::new(
                Ipv6Addr::from(self.addr?),
                self.m_port?,
                0,
                0,
            )),
            _ => return None,
        })
    }
}

pub enum NetZone {
    Public,
    Tor,
    I2p,
}

#[derive(Debug, PartialEq)]
pub enum NetworkAddress {
    Clear(SocketAddr),
}

impl EpeeObject for NetworkAddress {
    type Builder = TaggedNetworkAddress;

    fn number_of_fields(&self) -> u64 {
        2
    }

    fn write_fields<B: BufMut>(&self, w: &mut B) -> Result<(), epee_encoding::Error> {
        TaggedNetworkAddress::from(&self).write_fields(w)
    }
}

impl NetworkAddress {
    pub fn get_zone(&self) -> NetZone {
        match self {
            NetworkAddress::Clear(_) => NetZone::Public,
        }
    }

    pub fn is_loopback(&self) -> bool {
        // TODO
        false
    }

    pub fn is_local(&self) -> bool {
        // TODO
        false
    }

    pub fn port(&self) -> u16 {
        match self {
            NetworkAddress::Clear(ip) => ip.port(),
        }
    }
}

impl From<net::SocketAddrV4> for NetworkAddress {
    fn from(value: net::SocketAddrV4) -> Self {
        NetworkAddress::Clear(value.into())
    }
}

impl From<net::SocketAddrV6> for NetworkAddress {
    fn from(value: net::SocketAddrV6) -> Self {
        NetworkAddress::Clear(value.into())
    }
}

impl From<SocketAddr> for NetworkAddress {
    fn from(value: SocketAddr) -> Self {
        match value {
            SocketAddr::V4(v4) => v4.into(),
            SocketAddr::V6(v6) => v6.into(),
        }
    }
}
*/
