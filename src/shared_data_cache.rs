// Internal library to handle caching stuff that is fine to lose
use bytecheck::CheckBytes;
use rkyv::{
    archived_value,
    ser::{serializers::AllocSerializer, Serializer},
    Archive, Archived, Deserialize, Infallible, Serialize,
};


#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[archive_attr(repr(u16))]
#[repr(u16)]
pub enum SharedDataCache {
    A(u32),
}