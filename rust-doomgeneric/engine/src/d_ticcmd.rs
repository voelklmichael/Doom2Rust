use crate::stdint_types::byte;
pub type C2RustUnnamed = u32;
pub const BTS_SAVESHIFT: C2RustUnnamed = 2;
pub const BTS_SAVEMASK: C2RustUnnamed = 28;
pub const BTS_SAVEGAME: C2RustUnnamed = 2;
pub const BTS_PAUSE: C2RustUnnamed = 1;
pub const BT_WEAPONSHIFT: C2RustUnnamed = 3;
pub const BT_WEAPONMASK: C2RustUnnamed = 56;
pub const BT_CHANGE: C2RustUnnamed = 4;
pub const BT_SPECIALMASK: C2RustUnnamed = 3;
pub const BT_SPECIAL: C2RustUnnamed = 128;
pub const BT_USE: C2RustUnnamed = 2;
pub const BT_ATTACK: C2RustUnnamed = 1;

#[derive(Copy, Clone)]
pub struct ticcmd_t {
    pub forwardmove: i8,
    pub sidemove: i8,
    pub angleturn: i16,
    pub chatchar: byte,
    pub buttons: byte,
    pub consistancy: byte,
    pub buttons2: byte,
    pub inventory: i32,
    pub lookfly: byte,
    pub arti: byte,
}
