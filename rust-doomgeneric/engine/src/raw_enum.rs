//! Enums that are also tables of the original game: the variants are numbered `0, 1, 2, ...` in
//! declaration order, and the numbers appear in savegames and in the state/sprite/mobj tables.

/// Declares a field-less enum together with a function that maps a raw index back to the variant.
///
/// The variant list is written once, so the enum and its lookup cannot drift apart. The lookup
/// panics on an index that names no variant (a corrupt savegame or table).
macro_rules! raw_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident, $from_raw:ident, $what:literal {
            $($variant:ident),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($variant),*
        }

        $vis fn $from_raw(v: i32) -> $name {
            const ALL: &[$name] = &[$($name::$variant),*];
            usize::try_from(v)
                .ok()
                .and_then(|i| ALL.get(i))
                .copied()
                .unwrap_or_else(|| panic!(concat!("invalid ", $what, " {}"), v))
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::p_mobj::{
        mobjtype_from_raw, spritenum_from_raw, statenum_from_raw, MobjType, SpriteNum, StateNum,
    };

    #[test]
    fn every_index_maps_to_the_variant_with_that_number() {
        for i in 0..=StateNum::Tech2lamp4 as i32 {
            assert_eq!(statenum_from_raw(i) as i32, i);
        }
        for i in 0..=MobjType::Misc86 as i32 {
            assert_eq!(mobjtype_from_raw(i) as i32, i);
        }
        for i in 0..=SpriteNum::Tlp2 as i32 {
            assert_eq!(spritenum_from_raw(i) as i32, i);
        }
        assert_eq!(StateNum::Tech2lamp4 as i32, 966);
    }

    #[test]
    #[should_panic(expected = "invalid statenum 967")]
    fn an_index_past_the_end_is_rejected() {
        statenum_from_raw(967);
    }

    #[test]
    #[should_panic(expected = "invalid mobjtype -1")]
    fn a_negative_index_is_rejected() {
        mobjtype_from_raw(-1);
    }
}
