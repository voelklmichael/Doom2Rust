use crate::fixed_cstr::FixedCStr;
use crate::w_wad::LumpNum;
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; variant order must stay
pub enum MusicName {
    MusNone,
    E1m1,
    E1m2,
    E1m3,
    E1m4,
    E1m5,
    E1m6,
    E1m7,
    E1m8,
    E1m9,
    E2m1,
    E2m2,
    E2m3,
    E2m4,
    E2m5,
    E2m6,
    E2m7,
    E2m8,
    E2m9,
    E3m1,
    E3m2,
    E3m3,
    E3m4,
    E3m5,
    E3m6,
    E3m7,
    E3m8,
    E3m9,
    Inter,
    Intro,
    Bunny,
    Victor,
    Introa,
    Runnin,
    Stalks,
    Countd,
    Betwee,
    Doom,
    TheDa,
    Shawn,
    Ddtblu,
    InCit,
    Dead,
    Stlks2,
    Theda2,
    Doom2,
    Ddtbl2,
    Runni2,
    Dead2,
    Stlks3,
    Romero,
    Shawn2,
    Messag,
    Count2,
    Ddtbl3,
    Ampie,
    Theda3,
    Adrian,
    Messg2,
    Romer2,
    Tense,
    Shawn3,
    Openin,
    Evil,
    Ultima,
    ReadM,
    Dm2ttl,
    Dm2int,
}
pub const NUMMUSIC: i32 = 68;
#[derive(Copy, Clone)]
pub struct SfxInfo {
    pub tagname: Option<&'static str>,
    pub name: FixedCStr<9>,
    pub priority: i32,
    pub link: Option<SfxId>,
    pub pitch: i32,
    pub volume: i32,
    pub usefulness: i32,
    pub lumpnum: Option<LumpNum>,
    pub numchannels: i32,
}
#[derive(Copy, Clone)]
pub struct MusicInfo {
    pub name: FixedCStr<7>,
    pub lumpnum: Option<LumpNum>,
    pub handle: usize,
}
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; variant order must stay
pub enum SfxName {
    SfxNone,
    Pistol,
    Shotgn,
    Sgcock,
    Dshtgn,
    Dbopn,
    Dbcls,
    Dbload,
    Plasma,
    Bfg,
    Sawup,
    Sawidl,
    Sawful,
    Sawhit,
    Rlaunc,
    Rxplod,
    Firsht,
    Firxpl,
    Pstart,
    Pstop,
    Doropn,
    Dorcls,
    Stnmov,
    Swtchn,
    Swtchx,
    Plpain,
    Dmpain,
    Popain,
    Vipain,
    Mnpain,
    Pepain,
    Slop,
    Itemup,
    Wpnup,
    Oof,
    Telept,
    Posit1,
    Posit2,
    Posit3,
    Bgsit1,
    Bgsit2,
    Sgtsit,
    Cacsit,
    Brssit,
    Cybsit,
    Spisit,
    Bspsit,
    Kntsit,
    Vilsit,
    Mansit,
    Pesit,
    Sklatk,
    Sgtatk,
    Skepch,
    Vilatk,
    Claw,
    Skeswg,
    Pldeth,
    Pdiehi,
    Podth1,
    Podth2,
    Podth3,
    Bgdth1,
    Bgdth2,
    Sgtdth,
    Cacdth,
    Skldth,
    Brsdth,
    Cybdth,
    Spidth,
    Bspdth,
    Vildth,
    Kntdth,
    Pedth,
    Skedth,
    Posact,
    Bgact,
    Dmact,
    Bspact,
    Bspwlk,
    Vilact,
    Noway,
    Barexp,
    Punch,
    Hoof,
    Metal,
    Chgun,
    Tink,
    Bdopn,
    Bdcls,
    Itmbk,
    Flame,
    Flamst,
    Getpow,
    Bospit,
    Boscub,
    Bossit,
    Bospn,
    Bosdth,
    Manatk,
    Mandth,
    Sssit,
    Ssdth,
    Keenpn,
    Keendt,
    Skeact,
    Skesit,
    Skeatk,
    Radio,
}
pub const NUMSFX: usize = 109;
/// The lump name of each piece of music (without the `d_` prefix), in `MusicName` order.
const MUSIC_NAMES: [&str; 68] = [
    "", "e1m1", "e1m2", "e1m3", "e1m4", "e1m5", "e1m6", "e1m7", "e1m8", "e1m9", "e2m1", "e2m2",
    "e2m3", "e2m4", "e2m5", "e2m6", "e2m7", "e2m8", "e2m9", "e3m1", "e3m2", "e3m3", "e3m4", "e3m5",
    "e3m6", "e3m7", "e3m8", "e3m9", "inter", "intro", "bunny", "victor", "introa", "runnin",
    "stalks", "countd", "betwee", "doom", "the_da", "shawn", "ddtblu", "in_cit", "dead", "stlks2",
    "theda2", "doom2", "ddtbl2", "runni2", "dead2", "stlks3", "romero", "shawn2", "messag",
    "count2", "ddtbl3", "ampie", "theda3", "adrian", "messg2", "romer2", "tense", "shawn3",
    "openin", "evil", "ultima", "read_m", "dm2ttl", "dm2int",
];

/// The lump name (without the `ds` prefix) and priority of each sound effect, in `SfxName` order.
const SFX_TABLE: [(&str, i32); NUMSFX] = [
    ("none", 0),
    ("pistol", 64),
    ("shotgn", 64),
    ("sgcock", 64),
    ("dshtgn", 64),
    ("dbopn", 64),
    ("dbcls", 64),
    ("dbload", 64),
    ("plasma", 64),
    ("bfg", 64),
    ("sawup", 64),
    ("sawidl", 118),
    ("sawful", 64),
    ("sawhit", 64),
    ("rlaunc", 64),
    ("rxplod", 70),
    ("firsht", 70),
    ("firxpl", 70),
    ("pstart", 100),
    ("pstop", 100),
    ("doropn", 100),
    ("dorcls", 100),
    ("stnmov", 119),
    ("swtchn", 78),
    ("swtchx", 78),
    ("plpain", 96),
    ("dmpain", 96),
    ("popain", 96),
    ("vipain", 96),
    ("mnpain", 96),
    ("pepain", 96),
    ("slop", 78),
    ("itemup", 78),
    ("wpnup", 78),
    ("oof", 96),
    ("telept", 32),
    ("posit1", 98),
    ("posit2", 98),
    ("posit3", 98),
    ("bgsit1", 98),
    ("bgsit2", 98),
    ("sgtsit", 98),
    ("cacsit", 98),
    ("brssit", 94),
    ("cybsit", 92),
    ("spisit", 90),
    ("bspsit", 90),
    ("kntsit", 90),
    ("vilsit", 90),
    ("mansit", 90),
    ("pesit", 90),
    ("sklatk", 70),
    ("sgtatk", 70),
    ("skepch", 70),
    ("vilatk", 70),
    ("claw", 70),
    ("skeswg", 70),
    ("pldeth", 32),
    ("pdiehi", 32),
    ("podth1", 70),
    ("podth2", 70),
    ("podth3", 70),
    ("bgdth1", 70),
    ("bgdth2", 70),
    ("sgtdth", 70),
    ("cacdth", 70),
    ("skldth", 70),
    ("brsdth", 32),
    ("cybdth", 32),
    ("spidth", 32),
    ("bspdth", 32),
    ("vildth", 32),
    ("kntdth", 32),
    ("pedth", 32),
    ("skedth", 32),
    ("posact", 120),
    ("bgact", 120),
    ("dmact", 120),
    ("bspact", 100),
    ("bspwlk", 100),
    ("vilact", 100),
    ("noway", 78),
    ("barexp", 60),
    ("punch", 64),
    ("hoof", 70),
    ("metal", 70),
    ("chgun", 64),
    ("tink", 60),
    ("bdopn", 100),
    ("bdcls", 100),
    ("itmbk", 100),
    ("flame", 32),
    ("flamst", 32),
    ("getpow", 60),
    ("bospit", 70),
    ("boscub", 70),
    ("bossit", 70),
    ("bospn", 70),
    ("bosdth", 70),
    ("manatk", 70),
    ("mandth", 70),
    ("sssit", 70),
    ("ssdth", 70),
    ("keenpn", 70),
    ("keendt", 70),
    ("skeact", 70),
    ("skesit", 70),
    ("skeatk", 70),
    ("radio", 60),
];

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SfxId(pub u32);

pub struct SoundsState {
    pub s_music: [MusicInfo; 68],
    pub s_sfx: [SfxInfo; 109],
}

impl Default for SoundsState {
    fn default() -> Self {
        Self::new()
    }
}

impl SoundsState {
    pub fn sfx_mut(&mut self, id: SfxId) -> &mut SfxInfo {
        &mut self.s_sfx[id.0 as usize]
    }

    // Kept out of line: `GameState::new` inlines every state constructor, and once
    // `init_game_state` passes 256 KB the Xtensa linker fails ("dangerous relocation:
    // l32r: literal target out of range") building the firmware. (They were the biggest
    // constructors before their tables became compact rows; not re-measured on the firmware.)
    #[inline(never)]
    pub fn new() -> Self {
        let mut s_sfx: [SfxInfo; NUMSFX] = core::array::from_fn(|i| {
            let (name, priority) = SFX_TABLE[i];
            SfxInfo {
                tagname: None,
                name: FixedCStr::new(name),
                priority,
                link: None,
                pitch: -1,
                volume: -1,
                usefulness: 0,
                lumpnum: None,
                numchannels: -1,
            }
        });
        // The chaingun is the only sound with a fixed pitch and volume; it also links to the
        // pistol (see `fixup_self_links`).
        let chaingun = &mut s_sfx[SfxName::Chgun as usize];
        chaingun.pitch = 150;
        chaingun.volume = 0;
        Self {
            s_music: core::array::from_fn(|i| MusicInfo {
                name: FixedCStr::new(MUSIC_NAMES[i]),
                lumpnum: None,
                handle: 0,
            }),
            s_sfx,
        }
    }

    // Must run only after `self` is at its final, permanently-stable address
    // (i.e. once already moved into GameState's 'static storage via
    // `Box::leak`) -- link's target address is computed from `self`'s own
    // location, which would be invalidated by any subsequent move. See
    // init_game_state().
    pub fn fixup_self_links(&mut self) {
        self.s_sfx[SfxName::Chgun as usize].link = Some(SfxId(SfxName::Pistol as u32));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_are_built_from_the_rows() {
        let sounds = SoundsState::new();
        let pistol = &sounds.s_sfx[SfxName::Pistol as usize];
        assert_eq!(pistol.name.as_str(), "pistol");
        assert_eq!(pistol.priority, 64);
        assert_eq!(
            (pistol.pitch, pistol.volume, pistol.numchannels),
            (-1, -1, -1)
        );
        assert!(pistol.lumpnum.is_none() && pistol.link.is_none());
        assert_eq!(sounds.s_music[1].name.as_str(), "e1m1");
        assert_eq!(sounds.s_music[0].name.as_str(), "");
        assert_eq!(sounds.s_music[67].name.as_str(), "dm2int");
    }

    #[test]
    fn only_the_chaingun_has_a_fixed_pitch_and_volume() {
        let sounds = SoundsState::new();
        for (i, sfx) in sounds.s_sfx.iter().enumerate() {
            let chaingun = i == SfxName::Chgun as usize;
            assert_eq!(sfx.pitch == 150 && sfx.volume == 0, chaingun, "sfx {i}");
            assert_eq!(sfx.pitch == -1 && sfx.volume == -1, !chaingun, "sfx {i}");
        }
    }
}
