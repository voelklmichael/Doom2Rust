use crate::src::doomdef::NULL;
use crate::src::fixed_cstr::FixedCStr;
pub type C2RustUnnamed_0 = u32;
pub const NUMMUSIC: C2RustUnnamed_0 = 68;
pub const mus_dm2int: C2RustUnnamed_0 = 67;
pub const mus_dm2ttl: C2RustUnnamed_0 = 66;
pub const mus_read_m: C2RustUnnamed_0 = 65;
pub const mus_ultima: C2RustUnnamed_0 = 64;
pub const mus_evil: C2RustUnnamed_0 = 63;
pub const mus_openin: C2RustUnnamed_0 = 62;
pub const mus_shawn3: C2RustUnnamed_0 = 61;
pub const mus_tense: C2RustUnnamed_0 = 60;
pub const mus_romer2: C2RustUnnamed_0 = 59;
pub const mus_messg2: C2RustUnnamed_0 = 58;
pub const mus_adrian: C2RustUnnamed_0 = 57;
pub const mus_theda3: C2RustUnnamed_0 = 56;
pub const mus_ampie: C2RustUnnamed_0 = 55;
pub const mus_ddtbl3: C2RustUnnamed_0 = 54;
pub const mus_count2: C2RustUnnamed_0 = 53;
pub const mus_messag: C2RustUnnamed_0 = 52;
pub const mus_shawn2: C2RustUnnamed_0 = 51;
pub const mus_romero: C2RustUnnamed_0 = 50;
pub const mus_stlks3: C2RustUnnamed_0 = 49;
pub const mus_dead2: C2RustUnnamed_0 = 48;
pub const mus_runni2: C2RustUnnamed_0 = 47;
pub const mus_ddtbl2: C2RustUnnamed_0 = 46;
pub const mus_doom2: C2RustUnnamed_0 = 45;
pub const mus_theda2: C2RustUnnamed_0 = 44;
pub const mus_stlks2: C2RustUnnamed_0 = 43;
pub const mus_dead: C2RustUnnamed_0 = 42;
pub const mus_in_cit: C2RustUnnamed_0 = 41;
pub const mus_ddtblu: C2RustUnnamed_0 = 40;
pub const mus_shawn: C2RustUnnamed_0 = 39;
pub const mus_the_da: C2RustUnnamed_0 = 38;
pub const mus_doom: C2RustUnnamed_0 = 37;
pub const mus_betwee: C2RustUnnamed_0 = 36;
pub const mus_countd: C2RustUnnamed_0 = 35;
pub const mus_stalks: C2RustUnnamed_0 = 34;
pub const mus_runnin: C2RustUnnamed_0 = 33;
pub const mus_introa: C2RustUnnamed_0 = 32;
pub const mus_victor: C2RustUnnamed_0 = 31;
pub const mus_bunny: C2RustUnnamed_0 = 30;
pub const mus_intro: C2RustUnnamed_0 = 29;
pub const mus_inter: C2RustUnnamed_0 = 28;
pub const mus_e3m9: C2RustUnnamed_0 = 27;
pub const mus_e3m8: C2RustUnnamed_0 = 26;
pub const mus_e3m7: C2RustUnnamed_0 = 25;
pub const mus_e3m6: C2RustUnnamed_0 = 24;
pub const mus_e3m5: C2RustUnnamed_0 = 23;
pub const mus_e3m4: C2RustUnnamed_0 = 22;
pub const mus_e3m3: C2RustUnnamed_0 = 21;
pub const mus_e3m2: C2RustUnnamed_0 = 20;
pub const mus_e3m1: C2RustUnnamed_0 = 19;
pub const mus_e2m9: C2RustUnnamed_0 = 18;
pub const mus_e2m8: C2RustUnnamed_0 = 17;
pub const mus_e2m7: C2RustUnnamed_0 = 16;
pub const mus_e2m6: C2RustUnnamed_0 = 15;
pub const mus_e2m5: C2RustUnnamed_0 = 14;
pub const mus_e2m4: C2RustUnnamed_0 = 13;
pub const mus_e2m3: C2RustUnnamed_0 = 12;
pub const mus_e2m2: C2RustUnnamed_0 = 11;
pub const mus_e2m1: C2RustUnnamed_0 = 10;
pub const mus_e1m9: C2RustUnnamed_0 = 9;
pub const mus_e1m8: C2RustUnnamed_0 = 8;
pub const mus_e1m7: C2RustUnnamed_0 = 7;
pub const mus_e1m6: C2RustUnnamed_0 = 6;
pub const mus_e1m5: C2RustUnnamed_0 = 5;
pub const mus_e1m4: C2RustUnnamed_0 = 4;
pub const mus_e1m3: C2RustUnnamed_0 = 3;
pub const mus_e1m2: C2RustUnnamed_0 = 2;
pub const mus_e1m1: C2RustUnnamed_0 = 1;
pub const mus_None: C2RustUnnamed_0 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sfxinfo_struct {
    pub tagname: Option<&'static str>,
    pub name: FixedCStr<9>,
    pub priority: i32,
    pub link: Option<SfxId>,
    pub pitch: i32,
    pub volume: i32,
    pub usefulness: i32,
    pub lumpnum: i32,
    pub numchannels: i32,
}
pub type sfxinfo_t = sfxinfo_struct;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct musicinfo_t {
    pub name: FixedCStr<7>,
    pub lumpnum: i32,
    pub data: *mut ::core::ffi::c_void,
    pub handle: *mut ::core::ffi::c_void,
}
pub const sfx_pistol: C2RustUnnamed = 1;
pub type C2RustUnnamed = u32;
pub const NUMSFX: C2RustUnnamed = 109;
pub const sfx_radio: C2RustUnnamed = 108;
pub const sfx_skeatk: C2RustUnnamed = 107;
pub const sfx_skesit: C2RustUnnamed = 106;
pub const sfx_skeact: C2RustUnnamed = 105;
pub const sfx_keendt: C2RustUnnamed = 104;
pub const sfx_keenpn: C2RustUnnamed = 103;
pub const sfx_ssdth: C2RustUnnamed = 102;
pub const sfx_sssit: C2RustUnnamed = 101;
pub const sfx_mandth: C2RustUnnamed = 100;
pub const sfx_manatk: C2RustUnnamed = 99;
pub const sfx_bosdth: C2RustUnnamed = 98;
pub const sfx_bospn: C2RustUnnamed = 97;
pub const sfx_bossit: C2RustUnnamed = 96;
pub const sfx_boscub: C2RustUnnamed = 95;
pub const sfx_bospit: C2RustUnnamed = 94;
pub const sfx_getpow: C2RustUnnamed = 93;
pub const sfx_flamst: C2RustUnnamed = 92;
pub const sfx_flame: C2RustUnnamed = 91;
pub const sfx_itmbk: C2RustUnnamed = 90;
pub const sfx_bdcls: C2RustUnnamed = 89;
pub const sfx_bdopn: C2RustUnnamed = 88;
pub const sfx_tink: C2RustUnnamed = 87;
pub const sfx_chgun: C2RustUnnamed = 86;
pub const sfx_metal: C2RustUnnamed = 85;
pub const sfx_hoof: C2RustUnnamed = 84;
pub const sfx_punch: C2RustUnnamed = 83;
pub const sfx_barexp: C2RustUnnamed = 82;
pub const sfx_noway: C2RustUnnamed = 81;
pub const sfx_vilact: C2RustUnnamed = 80;
pub const sfx_bspwlk: C2RustUnnamed = 79;
pub const sfx_bspact: C2RustUnnamed = 78;
pub const sfx_dmact: C2RustUnnamed = 77;
pub const sfx_bgact: C2RustUnnamed = 76;
pub const sfx_posact: C2RustUnnamed = 75;
pub const sfx_skedth: C2RustUnnamed = 74;
pub const sfx_pedth: C2RustUnnamed = 73;
pub const sfx_kntdth: C2RustUnnamed = 72;
pub const sfx_vildth: C2RustUnnamed = 71;
pub const sfx_bspdth: C2RustUnnamed = 70;
pub const sfx_spidth: C2RustUnnamed = 69;
pub const sfx_cybdth: C2RustUnnamed = 68;
pub const sfx_brsdth: C2RustUnnamed = 67;
pub const sfx_skldth: C2RustUnnamed = 66;
pub const sfx_cacdth: C2RustUnnamed = 65;
pub const sfx_sgtdth: C2RustUnnamed = 64;
pub const sfx_bgdth2: C2RustUnnamed = 63;
pub const sfx_bgdth1: C2RustUnnamed = 62;
pub const sfx_podth3: C2RustUnnamed = 61;
pub const sfx_podth2: C2RustUnnamed = 60;
pub const sfx_podth1: C2RustUnnamed = 59;
pub const sfx_pdiehi: C2RustUnnamed = 58;
pub const sfx_pldeth: C2RustUnnamed = 57;
pub const sfx_skeswg: C2RustUnnamed = 56;
pub const sfx_claw: C2RustUnnamed = 55;
pub const sfx_vilatk: C2RustUnnamed = 54;
pub const sfx_skepch: C2RustUnnamed = 53;
pub const sfx_sgtatk: C2RustUnnamed = 52;
pub const sfx_sklatk: C2RustUnnamed = 51;
pub const sfx_pesit: C2RustUnnamed = 50;
pub const sfx_mansit: C2RustUnnamed = 49;
pub const sfx_vilsit: C2RustUnnamed = 48;
pub const sfx_kntsit: C2RustUnnamed = 47;
pub const sfx_bspsit: C2RustUnnamed = 46;
pub const sfx_spisit: C2RustUnnamed = 45;
pub const sfx_cybsit: C2RustUnnamed = 44;
pub const sfx_brssit: C2RustUnnamed = 43;
pub const sfx_cacsit: C2RustUnnamed = 42;
pub const sfx_sgtsit: C2RustUnnamed = 41;
pub const sfx_bgsit2: C2RustUnnamed = 40;
pub const sfx_bgsit1: C2RustUnnamed = 39;
pub const sfx_posit3: C2RustUnnamed = 38;
pub const sfx_posit2: C2RustUnnamed = 37;
pub const sfx_posit1: C2RustUnnamed = 36;
pub const sfx_telept: C2RustUnnamed = 35;
pub const sfx_oof: C2RustUnnamed = 34;
pub const sfx_wpnup: C2RustUnnamed = 33;
pub const sfx_itemup: C2RustUnnamed = 32;
pub const sfx_slop: C2RustUnnamed = 31;
pub const sfx_pepain: C2RustUnnamed = 30;
pub const sfx_mnpain: C2RustUnnamed = 29;
pub const sfx_vipain: C2RustUnnamed = 28;
pub const sfx_popain: C2RustUnnamed = 27;
pub const sfx_dmpain: C2RustUnnamed = 26;
pub const sfx_plpain: C2RustUnnamed = 25;
pub const sfx_swtchx: C2RustUnnamed = 24;
pub const sfx_swtchn: C2RustUnnamed = 23;
pub const sfx_stnmov: C2RustUnnamed = 22;
pub const sfx_dorcls: C2RustUnnamed = 21;
pub const sfx_doropn: C2RustUnnamed = 20;
pub const sfx_pstop: C2RustUnnamed = 19;
pub const sfx_pstart: C2RustUnnamed = 18;
pub const sfx_firxpl: C2RustUnnamed = 17;
pub const sfx_firsht: C2RustUnnamed = 16;
pub const sfx_rxplod: C2RustUnnamed = 15;
pub const sfx_rlaunc: C2RustUnnamed = 14;
pub const sfx_sawhit: C2RustUnnamed = 13;
pub const sfx_sawful: C2RustUnnamed = 12;
pub const sfx_sawidl: C2RustUnnamed = 11;
pub const sfx_sawup: C2RustUnnamed = 10;
pub const sfx_bfg: C2RustUnnamed = 9;
pub const sfx_plasma: C2RustUnnamed = 8;
pub const sfx_dbload: C2RustUnnamed = 7;
pub const sfx_dbcls: C2RustUnnamed = 6;
pub const sfx_dbopn: C2RustUnnamed = 5;
pub const sfx_dshtgn: C2RustUnnamed = 4;
pub const sfx_sgcock: C2RustUnnamed = 3;
pub const sfx_shotgn: C2RustUnnamed = 2;
pub const sfx_None: C2RustUnnamed = 0;
const INITIAL_S_MUSIC: [musicinfo_t; 68] = [
    musicinfo_t {
        name: FixedCStr([0u8; 7]),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e1m1\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e1m2\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e1m3\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e1m4\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e1m5\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e1m6\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e1m7\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e1m8\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e1m9\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e2m1\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e2m2\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e2m3\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e2m4\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e2m5\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e2m6\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e2m7\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e2m8\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e2m9\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e3m1\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e3m2\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e3m3\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e3m4\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e3m5\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e3m6\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e3m7\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e3m8\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"e3m9\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"inter\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"intro\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"bunny\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"victor\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"introa\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"runnin\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"stalks\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"countd\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"betwee\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"doom\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"the_da\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"shawn\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"ddtblu\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"in_cit\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"dead\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"stlks2\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"theda2\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"doom2\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"ddtbl2\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"runni2\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"dead2\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"stlks3\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"romero\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"shawn2\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"messag\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"count2\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"ddtbl3\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"ampie\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"theda3\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"adrian\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"messg2\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"romer2\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"tense\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"shawn3\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"openin\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"evil\x00\x00\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"ultima\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"read_m\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"dm2ttl\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
    musicinfo_t {
        name: FixedCStr(*b"dm2int\x00"),
        lumpnum: 0 as i32,
        data: NULL,
        handle: NULL,
    },
];

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SfxId(pub u32);

pub struct SoundsState {
    pub S_music: [musicinfo_t; 68],
    pub S_sfx: [sfxinfo_t; 109],
}

impl SoundsState {
    pub fn sfx_mut(&mut self, id: SfxId) -> *mut sfxinfo_t {
        &mut self.S_sfx[id.0 as usize] as *mut sfxinfo_t
    }

    pub fn new() -> Self {
        SoundsState {
            S_music: INITIAL_S_MUSIC,
            S_sfx: [
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"none\0\0\0\0\0"),
                    priority: 0 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"pistol\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"shotgn\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"sgcock\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"dshtgn\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"dbopn\0\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"dbcls\0\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"dbload\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"plasma\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bfg\0\0\0\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"sawup\0\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"sawidl\0\0\0"),
                    priority: 118 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"sawful\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"sawhit\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"rlaunc\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"rxplod\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"firsht\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"firxpl\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"pstart\0\0\0"),
                    priority: 100 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"pstop\0\0\0\0"),
                    priority: 100 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"doropn\0\0\0"),
                    priority: 100 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"dorcls\0\0\0"),
                    priority: 100 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"stnmov\0\0\0"),
                    priority: 119 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"swtchn\0\0\0"),
                    priority: 78 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"swtchx\0\0\0"),
                    priority: 78 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"plpain\0\0\0"),
                    priority: 96 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"dmpain\0\0\0"),
                    priority: 96 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"popain\0\0\0"),
                    priority: 96 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"vipain\0\0\0"),
                    priority: 96 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"mnpain\0\0\0"),
                    priority: 96 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"pepain\0\0\0"),
                    priority: 96 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"slop\0\0\0\0\0"),
                    priority: 78 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"itemup\0\0\0"),
                    priority: 78 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"wpnup\0\0\0\0"),
                    priority: 78 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"oof\0\0\0\0\0\0"),
                    priority: 96 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"telept\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"posit1\0\0\0"),
                    priority: 98 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"posit2\0\0\0"),
                    priority: 98 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"posit3\0\0\0"),
                    priority: 98 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bgsit1\0\0\0"),
                    priority: 98 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bgsit2\0\0\0"),
                    priority: 98 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"sgtsit\0\0\0"),
                    priority: 98 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"cacsit\0\0\0"),
                    priority: 98 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"brssit\0\0\0"),
                    priority: 94 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"cybsit\0\0\0"),
                    priority: 92 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"spisit\0\0\0"),
                    priority: 90 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bspsit\0\0\0"),
                    priority: 90 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"kntsit\0\0\0"),
                    priority: 90 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"vilsit\0\0\0"),
                    priority: 90 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"mansit\0\0\0"),
                    priority: 90 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"pesit\0\0\0\0"),
                    priority: 90 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"sklatk\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"sgtatk\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"skepch\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"vilatk\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"claw\0\0\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"skeswg\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"pldeth\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"pdiehi\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"podth1\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"podth2\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"podth3\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bgdth1\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bgdth2\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"sgtdth\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"cacdth\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"skldth\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"brsdth\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"cybdth\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"spidth\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bspdth\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"vildth\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"kntdth\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"pedth\0\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"skedth\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"posact\0\0\0"),
                    priority: 120 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bgact\0\0\0\0"),
                    priority: 120 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"dmact\0\0\0\0"),
                    priority: 120 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bspact\0\0\0"),
                    priority: 100 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bspwlk\0\0\0"),
                    priority: 100 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"vilact\0\0\0"),
                    priority: 100 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"noway\0\0\0\0"),
                    priority: 78 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"barexp\0\0\0"),
                    priority: 60 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"punch\0\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"hoof\0\0\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"metal\0\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"chgun\0\0\0\0"),
                    priority: 64 as i32,
                    link: None,
                    pitch: 150 as i32,
                    volume: 0 as i32,
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"tink\0\0\0\0\0"),
                    priority: 60 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bdopn\0\0\0\0"),
                    priority: 100 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bdcls\0\0\0\0"),
                    priority: 100 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"itmbk\0\0\0\0"),
                    priority: 100 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"flame\0\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"flamst\0\0\0"),
                    priority: 32 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"getpow\0\0\0"),
                    priority: 60 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bospit\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"boscub\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bossit\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bospn\0\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"bosdth\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"manatk\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"mandth\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"sssit\0\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"ssdth\0\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"keenpn\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"keendt\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"skeact\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"skesit\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"skeatk\0\0\0"),
                    priority: 70 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
                sfxinfo_struct {
                    tagname: None,
                    name: FixedCStr(*b"radio\0\0\0\0"),
                    priority: 60 as i32,
                    link: None,
                    pitch: -(1 as i32),
                    volume: -(1 as i32),
                    usefulness: 0 as i32,
                    lumpnum: 0 as i32,
                    numchannels: -(1 as i32),
                },
            ],
        }
    }

    // Must run only after `self` is at its final, permanently-stable address
    // (i.e. once already moved into GameState's 'static storage via
    // `Box::leak`) -- link's target address is computed from `self`'s own
    // location, which would be invalidated by any subsequent move. See
    // init_game_state().
    pub fn fixup_self_links(&mut self) {
        self.S_sfx[sfx_chgun as usize].link = Some(SfxId(sfx_pistol));
    }
}
