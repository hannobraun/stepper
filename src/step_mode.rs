use core::convert::TryFrom;

use paste::paste;

/// Implemented for all step mode enums
///
/// Required as a trait bound by `SetStepMode::StepMode`.
pub trait StepMode:
    Into<u16> + TryFrom<u16, Error = InvalidStepModeError>
{
    /// The type of the iterator returned by [`StepMode::iter`]
    type Iter: Iterator<Item = Self>;

    /// Returns an iterator over all supported modes
    ///
    /// Starts at the mode for configuring full steps and ends at the highest
    /// supported number of microsteps per step.
    fn iter() -> Self::Iter;
}

macro_rules! generate_step_mode_enums {
    (
        $(
            $max:expr => $($variant:expr),*;
        )*
    ) => {
        $(
            generate_step_mode_enums!(@gen_enum,
                (),
                (),
                (),
                $max => $($variant,)*
            );
        )*
    };

    // This is a trick to work around a limitation of Rust macros: We can't
    // generate a part of something, like an enum variant. We can only generate
    // complete things, like the whole enum.
    //
    // This first rules gets matched on the first call, when there is not output
    // yet. It is generates the full step variant, then passes the input on to
    // the next macro call.
    (
        @gen_enum,
        (),
        (),
        (),
        $max:expr => $($input:expr,)*
    ) => {
        generate_step_mode_enums!(
            @gen_enum,
            (
                #[doc = "Full steps"]
                Full = 1,
            ),
            (
                1 => Ok(Self::Full),
            ),
            (
                [<StepMode $max>]::Full,
            ),
            $max => $($input,)*
        );
    };
    // This next rule gets matched as long as there are still enum variants to
    // be generated. It creates the tokens that make up a variant, then passes
    // them and the rest of the input on to the next recursive macro call.
    (
        @gen_enum,
        (
            $($variant_output:tt)*
        ),
        (
            $($try_from_output:tt)*
        ),
        (
            $($iter_output:tt)*
        ),
        $max:expr => $variant:expr, $($input:expr,)*
    ) => {
        generate_step_mode_enums!(
            @gen_enum,
            (
                $($variant_output)*

                #[doc = $variant " microsteps per full step"]
                [<M $variant>] = $variant,
            ),
            (
                $($try_from_output)*

                $variant => Ok(Self::[<M $variant>]),
            ),
            (
                $($iter_output)*

                [<StepMode $max>]::[<M $variant>],
            ),
            $max => $($input,)*
        );
    };
    // This last rule gets matched when there is no more input left and all
    // variants have been generated. It takes all the tokens generated in
    // previous macro calls and uses them to generate the complete enum.
    (
        @gen_enum,
        (
            $($variant_output:tt)*
        ),
        (
            $($try_from_output:tt)*
        ),
        (
            $($iter_output:tt)*
        ),
        $max:expr =>
    ) => {
        paste! {
            #[doc =
                "Defines the step mode with a resolution of up to " $max " \
                microsteps\n\
                \n\
                Can be used by drivers for the `StepMode` associated type of \
                `SetStepMode`."
            ]
            #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
            pub enum [<StepMode $max>] {
                $($variant_output)*
            }

            impl From<[<StepMode $max>]> for u16 {
                fn from(step_mode: [<StepMode $max>]) -> Self {
                    step_mode as Self
                }
            }

            impl TryFrom<u16> for [<StepMode $max>] {
                type Error = InvalidStepModeError;

                fn try_from(val: u16) -> Result<Self, Self::Error> {
                    match val {
                        $($try_from_output)*

                        _ => Err(InvalidStepModeError),
                    }
                }
            }

            impl StepMode for [<StepMode $max>] {
                // It would be nice to avoid the custom iterator and use
                // `iter::from_fn` instead. That would require `impl Iterator`
                // here, which is not supported yet. Tracking issue:
                // https://github.com/rust-lang/rust/issues/63063
                type Iter = [<Iter $max>];

                fn iter() -> Self::Iter {
                    [<Iter $max>] {
                        i: 0,
                    }
                }
            }

            #[doc =
                "An iterator over the variants of [`StepMode" $max "`]"
            ]
            pub struct [<Iter $max>] {
                i: usize,
            }

            impl Iterator for [<Iter $max>] {
                type Item = [<StepMode $max>];

                fn next(&mut self) -> Option<Self::Item> {
                    let modes = [$($iter_output)*];

                    if self.i < modes.len() {
                        let mode = modes[self.i];
                        self.i += 1;
                        Some(mode)
                    }
                    else {
                        None
                    }
                }
            }
        }
    };
}

generate_step_mode_enums! {
    2   => 2;
    4   => 2, 4;
    8   => 2, 4, 8;
    16  => 2, 4, 8, 16;
    32  => 2, 4, 8, 16, 32;
    64  => 2, 4, 8, 16, 32, 64;
    128 => 2, 4, 8, 16, 32, 64, 128;
    256 => 2, 4, 8, 16, 32, 64, 128, 256;
}

/// Indicates that a given step mode value did not represent a valid step mode
///
/// Returned by the `TryFrom` implementations of the various step mode enums.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidStepModeError;

#[cfg(test)]
mod tests {
    // Only tests `StepMode256`. This should be fine, since all other step mode
    // enums are generated by the same code.

    use core::convert::TryFrom;

    use super::{StepMode as _, StepMode256};

    #[test]
    fn step_mode_should_convert_into_microsteps_per_step() {
        use StepMode256::*;

        assert_eq!(<StepMode256 as Into<u16>>::into(Full), 1);
        assert_eq!(<StepMode256 as Into<u16>>::into(M2), 2);
        assert_eq!(<StepMode256 as Into<u16>>::into(M4), 4);
        assert_eq!(<StepMode256 as Into<u16>>::into(M8), 8);
        assert_eq!(<StepMode256 as Into<u16>>::into(M16), 16);
        assert_eq!(<StepMode256 as Into<u16>>::into(M32), 32);
        assert_eq!(<StepMode256 as Into<u16>>::into(M64), 64);
        assert_eq!(<StepMode256 as Into<u16>>::into(M128), 128);
        assert_eq!(<StepMode256 as Into<u16>>::into(M256), 256);
    }

    #[test]
    fn step_mode_should_convert_from_microsteps_per_step() {
        use StepMode256::*;

        assert_eq!(<StepMode256 as TryFrom<u16>>::try_from(1), Ok(Full));
        assert_eq!(<StepMode256 as TryFrom<u16>>::try_from(2), Ok(M2));
        assert_eq!(<StepMode256 as TryFrom<u16>>::try_from(4), Ok(M4));
        assert_eq!(<StepMode256 as TryFrom<u16>>::try_from(8), Ok(M8));
        assert_eq!(<StepMode256 as TryFrom<u16>>::try_from(16), Ok(M16));
        assert_eq!(<StepMode256 as TryFrom<u16>>::try_from(32), Ok(M32));
        assert_eq!(<StepMode256 as TryFrom<u16>>::try_from(64), Ok(M64));
        assert_eq!(<StepMode256 as TryFrom<u16>>::try_from(128), Ok(M128));
        assert_eq!(<StepMode256 as TryFrom<u16>>::try_from(256), Ok(M256));
    }

    #[test]
    fn step_mode_should_provide_iterator_over_modes() {
        use StepMode256::*;

        let modes: Vec<_> = StepMode256::iter().collect();
        assert_eq!(modes, [Full, M2, M4, M8, M16, M32, M64, M128, M256]);
    }
}
