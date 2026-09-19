pub const BTS_SAVESHIFT: i32 = 2;
pub const BTS_SAVEMASK: i32 = 28;
pub const BTS_SAVEGAME: i32 = 2;
pub const BTS_PAUSE: i32 = 1;
pub const BT_WEAPONSHIFT: i32 = 3;
pub const BT_WEAPONMASK: i32 = 56;
pub const BT_CHANGE: i32 = 4;
pub const BT_SPECIALMASK: i32 = 3;
pub const BT_SPECIAL: i32 = 128;
pub const BT_USE: i32 = 2;
pub const BT_ATTACK: i32 = 1;

#[derive(Copy, Clone)]
pub struct ticcmd_t {
    pub forwardmove: i8,
    pub sidemove: i8,
    pub angleturn: i16,
    pub chatchar: u8,
    pub buttons: u8,
    pub consistancy: u8,
    pub buttons2: u8,
    pub inventory: i32,
    pub lookfly: u8,
    pub arti: u8,
}
