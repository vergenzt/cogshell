use crate::{non_nil::NonNilUuid, Builder, Uuid};

use arbitrary::{Arbitrary, Unstructured};

impl Arbitrary<'_> for Uuid {
    fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
        let b = u
            .bytes(16)?
            .try_into()
            .map_err(|_| arbitrary::Error::NotEnoughData)?;

        Ok(Builder::from_random_bytes(b).into_uuid())
    }

    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (16, Some(16))
    }
}
impl arbitrary::Arbitrary<'_> for NonNilUuid {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let uuid = Uuid::arbitrary(u)?;
        Self::try_from(uuid).map_err(|_| arbitrary::Error::IncorrectFormat)
    }

    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (16, Some(16))
    }
}


