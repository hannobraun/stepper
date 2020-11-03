use core::convert::TryFrom;

use paste::paste;

macro_rules! generate_step_mode_enums {
    (
        $(
            $max:expr => $($variant:expr),*;
        )*
    ) => {
        $(
            generate_step_mode_enums!(@gen_enum, (), (), $max => $($variant,)*);
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
        $max:expr =>
    ) => {
        paste! {
            #[doc =
                "Defines the step mode with a resolution of up to " $max " \
                microsteps"
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
/// Valid values are 1, 2, 4, 8, 16, 32, 64, 128, and 256.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidStepModeError;
