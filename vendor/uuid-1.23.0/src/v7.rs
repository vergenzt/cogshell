//! The implementation for Version 7 UUIDs.
//!
//! Note that you need to enable the `v7` Cargo feature
//! in order to use this module.

use core::cmp;

use crate::{rng, timestamp::Timestamp, Builder, Uuid};

impl Uuid {
    /// Create a new version 7 UUID using the current time value.
    ///
    /// This method is a convenient alternative to [`Uuid::new_v7`] that uses the current system time
    /// as the source timestamp. All UUIDs generated through this method by the same process are
    /// guaranteed to be ordered by their creation.
    #[cfg(feature = "std")]
    pub fn now_v7() -> Self {
        Self::new_v7(Timestamp::now(
            crate::timestamp::context::shared_context_v7(),
        ))
    }

    /// Create a new version 7 UUID using a time value and random bytes.
    ///
    /// When the `std` feature is enabled, you can also use [`Uuid::now_v7`].
    ///
    /// Note that usage of this method requires the `v7` feature of this crate
    /// to be enabled.
    ///
    /// Also see [`Uuid::now_v7`] for a convenient way to generate version 7
    /// UUIDs using the current system time.
    ///
    /// # Counter treatment
    ///
    /// This method accepts a [`Timestamp`] which may include a counter value.
    /// Only up to 74 bits of the counter value will be used when constructing
    /// the UUID. Any unused counter bits will be filled with random data.
    ///
    /// # Examples
    ///
    /// A v7 UUID can be created from a unix [`Timestamp`] plus a 128 bit
    /// random number. When supplied as such, the data will be combined
    /// to ensure uniqueness and sortability at millisecond granularity.
    ///
    /// ```rust
    /// # use uuid::{Uuid, Timestamp, NoContext};
    /// let ts = Timestamp::from_unix(NoContext, 1497624119, 1234);
    ///
    /// let uuid = Uuid::new_v7(ts);
    ///
    /// assert!(
    ///     uuid.hyphenated().to_string().starts_with("015cb15a-86d8-7")
    /// );
    /// ```
    ///
    /// A v7 UUID can also be created with a counter to ensure batches of
    /// UUIDs created together remain sortable:
    ///
    /// ```rust
    /// # use uuid::{Uuid, Timestamp, ContextV7};
    /// let context = ContextV7::new();
    /// let uuid1 = Uuid::new_v7(Timestamp::from_unix(&context, 1497624119, 1234));
    /// let uuid2 = Uuid::new_v7(Timestamp::from_unix(&context, 1497624119, 1234));
    ///
    /// assert!(uuid1 < uuid2);
    /// ```
    ///
    /// # References
    ///
    /// * [UUID Version 7 in RFC 9562](https://www.ietf.org/rfc/rfc9562.html#section-5.7)
    pub fn new_v7(ts: Timestamp) -> Self {
        let (secs, nanos) = ts.to_unix();
        let millis = secs
            .saturating_mul(1000)
            .saturating_add(nanos as u64 / 1_000_000);

        let (counter, counter_bits) = ts.counter();

        // If the counter intersects the variant field then shift around it.
        // This ensures that any bits set in the counter that would intersect
        // the variant are still preserved
        let shift_counter_over_variant = |mut counter: u128, mut counter_bits: u32| {
            let mask = u128::MAX << (cmp::min(128, counter_bits) - 12);
            counter = (counter & !mask) | ((counter & mask) << 2);
            counter_bits += 2;

            (counter, counter_bits)
        };

        // Mask `counter_bits` of the `counter` into `dst`
        let mask_counter_into_random = |mut dst: u128, counter: u128, counter_bits: u32| {
            dst &= u128::MAX >> counter_bits;
            dst |= counter << (128 - counter_bits);

            dst
        };

        let counter_and_random = match counter_bits {
            // The counter doesn't contribute any bits
            0 => rng::u128(),
            // The counter doesn't intersect the variant field
            // It needs to be merged with random data
            ..12 => {
                let counter_bits = counter_bits as u32;

                mask_counter_into_random(rng::u128(), counter, counter_bits)
            }
            // `rand_a` (12 bits) + `rand_b` (62 bits) + `var` (2 bits)
            // The counter needs to be shifted around the variant and merged with random data
            ..74 => {
                let (counter, counter_bits) =
                    shift_counter_over_variant(counter, counter_bits as u32);

                mask_counter_into_random(rng::u128(), counter, counter_bits)
            }
            // The counter overrides all bits
            74.. => {
                let (counter, _) = shift_counter_over_variant(counter, counter_bits as u32);

                counter
            }
        };

        Builder::from_unix_timestamp_millis(
            millis,
            &counter_and_random.to_be_bytes()[..10].try_into().unwrap(),
        )
        .into_uuid()
    }
}


