use rkyv::{
    boxed::{ArchivedBox, BoxResolver},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
    Archive, Deserialize, Fallible, Serialize,
};
use std::cell::RefCell;

pub struct InlineRefCell;

impl<F: Archive> ArchiveWith<RefCell<F>> for InlineRefCell {
    type Archived = F::Archived;
    type Resolver = F::Resolver;

    #[inline]
    unsafe fn resolve_with(
        field: &RefCell<F>,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        (*field.borrow()).resolve(pos, resolver, out);
    }
}

impl<F: Serialize<S>, S: Fallible + ?Sized> SerializeWith<RefCell<F>, S> for InlineRefCell {
    #[inline]
    fn serialize_with(field: &RefCell<F>, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        (*field.borrow()).serialize(serializer)
    }
}

impl<F: Archive, D: Fallible + ?Sized> DeserializeWith<F::Archived, RefCell<F>, D> for InlineRefCell
where
    F::Archived: Deserialize<F, D>,
{
    #[inline]
    fn deserialize_with(field: &F::Archived, deserializer: &mut D) -> Result<RefCell<F>, D::Error> {
        match field.deserialize(deserializer) {
            Ok(val) => Ok(RefCell::new(val)),
            Err(a) => Err(a),
        }
    }
}
