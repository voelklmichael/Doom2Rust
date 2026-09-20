pub const BTS_SAVESHIFT: u32 = 2;
pub const BTS_SAVEMASK: u8 = 28;
pub const BTS_SAVEGAME: u8 = 2;
pub const BTS_PAUSE: u8 = 1;
pub const BT_WEAPONSHIFT: u32 = 3;
pub const BT_WEAPONMASK: u8 = 56;
pub const BT_CHANGE: u8 = 4;
pub const BT_SPECIALMASK: u8 = 3;
pub const BT_SPECIAL: u8 = 128;
pub const BT_USE: u8 = 2;
pub const BT_ATTACK: u8 = 1;

#[derive(Copy, Clone)]
pub struct TicCmd {
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
