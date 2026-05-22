macro_rules! deref_field {
    // https://stackoverflow.com/a/61189128
    (
      impl
      $(<
        $($lt:lifetime $(: $ltb:lifetime)?),*
        $($gt:ident $(: $gtb:tt)?),*
      >)?
      *
      $from:ty = . $field:tt $(($($fsfx:tt)?))? : $to:ty
    ) => {
      impl$(< $($lt $(: $ltb)?),* $($gt $(: $gtb)?),* >)? std::ops::Deref for $from {
            type Target = $to;
            fn deref(&self) -> &Self::Target {
                &self.$field $(($($fsfx)?))?
            }
        }
    };
}

pub(crate) use deref_field;
