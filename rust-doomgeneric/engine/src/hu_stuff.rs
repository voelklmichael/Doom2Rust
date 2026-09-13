use crate::src::d_event::event_t;
use crate::src::d_event::EvType;
use crate::src::d_mode::GameVersion;
use crate::src::d_mode::GameMode_t;
use crate::src::d_mode::GameMission_t;
use crate::src::d_player::PlayerId;
use crate::src::fixed_cstr::FixedCStr;
use crate::src::doomdef::MAXPLAYERS;
use crate::src::doomdef::TICRATE;
use crate::src::game_state::GameState;
use crate::src::hu_lib::{
    hu_itext_t, hu_stext_t, hu_textline_t, patch_t, HUlib_addCharToTextLine,
    HUlib_addMessageToSText, HUlib_drawIText, HUlib_drawSText, HUlib_drawTextLine,
    HUlib_eraseIText, HUlib_eraseSText, HUlib_eraseTextLine, HUlib_initIText, HUlib_initSText,
    HUlib_initTextLine, HUlib_keyInIText, HUlib_resetIText,
};
use crate::src::m_controls::KEY_ENTER;
use crate::src::m_controls::KEY_ESCAPE;
use crate::src::m_controls::KEY_RALT;
use crate::src::m_controls::KEY_RSHIFT;
use crate::src::s_sound::S_StartSound;
use crate::src::s_sound::SoundOrigin;
use crate::src::sounds::{sfx_radio, sfx_tink};
use crate::src::stdint_types::byte;
use crate::src::w_wad::W_CacheLumpName;
use crate::src::z_zone::PU_STATIC;

pub struct HuStuffState {
    pub plr: PlayerId,
    pub hu_font: [*mut patch_t; 63],
    pub w_title: hu_textline_t,
    pub chat_on: bool,
    pub w_chat: hu_itext_t,
    pub always_off: bool,
    pub chat_dest: [u8; 4],
    pub w_inputbuffer: [hu_itext_t; 4],
    pub message_on: bool,
    pub message_dontfuckwithme: bool,
    pub message_nottobefuckedwith: bool,
    pub w_message: hu_stext_t,
    pub message_counter: i32,
    pub headsupactive: bool,
    pub chatchars: [u8; 128],
    pub head: i32,
    pub tail: i32,
    pub chat_macros: [&'static str; 10],
    pub hu_responder_altdown: bool,
    pub hu_responder_num_nobrainers: i32,
    pub player_names: [*mut ::core::ffi::c_char; 4],
}

impl HuStuffState {
    pub const fn new() -> Self {
        HuStuffState {
            plr: PlayerId(0),
            hu_font: [::core::ptr::null::<patch_t>() as *mut patch_t; 63],
            w_title: hu_textline_t {
                x: 0,
                y: 0,
                f: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
                sc: 0,
                l: String::new(),
                needsupdate: 0,
            },
            chat_on: false,
            w_chat: hu_itext_t {
                l: hu_textline_t {
                    x: 0,
                    y: 0,
                    f: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
                    sc: 0,
                    l: String::new(),
                    needsupdate: 0,
                },
                lm: 0,
                on: ::core::ptr::null::<bool>() as *mut bool,
                laston: false,
            },
            always_off: false,
            chat_dest: [0; 4],
            w_inputbuffer: [
                new_hu_itext_t(),
                new_hu_itext_t(),
                new_hu_itext_t(),
                new_hu_itext_t(),
            ],
            message_on: false,
            message_dontfuckwithme: false,
            message_nottobefuckedwith: false,
            w_message: hu_stext_t {
                l: [
                    new_hu_textline_t(),
                    new_hu_textline_t(),
                    new_hu_textline_t(),
                    new_hu_textline_t(),
                ],
                h: 0,
                cl: 0,
                on: ::core::ptr::null::<bool>() as *mut bool,
                laston: false,
            },
            message_counter: 0,
            headsupactive: false,
            chatchars: [0; 128],
            head: 0,
            tail: 0,
            chat_macros: [
                "No",
                "I'm ready to kick butt!",
                "I'm OK.",
                "I'm not looking too good!",
                "Help!",
                "You suck!",
                "Next time, scumbag...",
                "Come here!",
                "I'll take care of it.",
                "Yes",
            ],
            hu_responder_altdown: false,
            hu_responder_num_nobrainers: 0,
            player_names: [
                HUSTR_PLRGREEN.as_ptr() as *mut ::core::ffi::c_char,
                HUSTR_PLRINDIGO.as_ptr() as *mut ::core::ffi::c_char,
                HUSTR_PLRBROWN.as_ptr() as *mut ::core::ffi::c_char,
                HUSTR_PLRRED.as_ptr() as *mut ::core::ffi::c_char,
            ],
        }
    }
}

pub const KEY_LALT: i32 = KEY_RALT;
pub const HU_FONTSTART: i32 = '!' as i32;
pub const HU_FONTEND: i32 = '_' as i32;
pub const HU_FONTSIZE: i32 = HU_FONTEND - HU_FONTSTART + 1 as i32;
pub const HU_BROADCAST: i32 = 5;
pub const HU_MSGX: i32 = 0;
pub const HU_MSGY: i32 = 0;
pub const HU_MSGHEIGHT: i32 = 1;
pub const HU_MSGTIMEOUT: i32 = 4 * TICRATE;
pub const HUSTR_E1M1: &str = "E1M1: Hangar";
pub const HUSTR_E1M2: &str = "E1M2: Nuclear Plant";
pub const HUSTR_E1M3: &str = "E1M3: Toxin Refinery";
pub const HUSTR_E1M4: &str = "E1M4: Command Control";
pub const HUSTR_E1M5: &str = "E1M5: Phobos Lab";
pub const HUSTR_E1M6: &str = "E1M6: Central Processing";
pub const HUSTR_E1M7: &str = "E1M7: Computer Station";
pub const HUSTR_E1M8: &str = "E1M8: Phobos Anomaly";
pub const HUSTR_E1M9: &str = "E1M9: Military Base";
pub const HUSTR_E2M1: &str = "E2M1: Deimos Anomaly";
pub const HUSTR_E2M2: &str = "E2M2: Containment Area";
pub const HUSTR_E2M3: &str = "E2M3: Refinery";
pub const HUSTR_E2M4: &str = "E2M4: Deimos Lab";
pub const HUSTR_E2M5: &str = "E2M5: Command Center";
pub const HUSTR_E2M6: &str = "E2M6: Halls of the Damned";
pub const HUSTR_E2M7: &str = "E2M7: Spawning Vats";
pub const HUSTR_E2M8: &str = "E2M8: Tower of Babel";
pub const HUSTR_E2M9: &str = "E2M9: Fortress of Mystery";
pub const HUSTR_E3M1: &str = "E3M1: Hell Keep";
pub const HUSTR_E3M2: &str = "E3M2: Slough of Despair";
pub const HUSTR_E3M3: &str = "E3M3: Pandemonium";
pub const HUSTR_E3M4: &str = "E3M4: House of Pain";
pub const HUSTR_E3M5: &str = "E3M5: Unholy Cathedral";
pub const HUSTR_E3M6: &str = "E3M6: Mt. Erebus";
pub const HUSTR_E3M7: &str = "E3M7: Limbo";
pub const HUSTR_E3M8: &str = "E3M8: Dis";
pub const HUSTR_E3M9: &str = "E3M9: Warrens";
pub const HUSTR_E4M1: &str = "E4M1: Hell Beneath";
pub const HUSTR_E4M2: &str = "E4M2: Perfect Hatred";
pub const HUSTR_E4M3: &str = "E4M3: Sever The Wicked";
pub const HUSTR_E4M4: &str = "E4M4: Unruly Evil";
pub const HUSTR_E4M5: &str = "E4M5: They Will Repent";
pub const HUSTR_E4M6: &str = "E4M6: Against Thee Wickedly";
pub const HUSTR_E4M7: &str = "E4M7: And Hell Followed";
pub const HUSTR_E4M8: &str = "E4M8: Unto The Cruel";
pub const HUSTR_E4M9: &str = "E4M9: Fear";
pub const HUSTR_1: &str = "level 1: entryway";
pub const HUSTR_2: &str = "level 2: underhalls";
pub const HUSTR_3: &str = "level 3: the gantlet";
pub const HUSTR_4: &str = "level 4: the focus";
pub const HUSTR_5: &str = "level 5: the waste tunnels";
pub const HUSTR_6: &str = "level 6: the crusher";
pub const HUSTR_7: &str = "level 7: dead simple";
pub const HUSTR_8: &str = "level 8: tricks and traps";
pub const HUSTR_9: &str = "level 9: the pit";
pub const HUSTR_10: &str = "level 10: refueling base";
pub const HUSTR_11: &str = "level 11: 'o' of destruction!";
pub const HUSTR_12: &str = "level 12: the factory";
pub const HUSTR_13: &str = "level 13: downtown";
pub const HUSTR_14: &str = "level 14: the inmost dens";
pub const HUSTR_15: &str = "level 15: industrial zone";
pub const HUSTR_16: &str = "level 16: suburbs";
pub const HUSTR_17: &str = "level 17: tenements";
pub const HUSTR_18: &str = "level 18: the courtyard";
pub const HUSTR_19: &str = "level 19: the citadel";
pub const HUSTR_20: &str = "level 20: gotcha!";
pub const HUSTR_21: &str = "level 21: nirvana";
pub const HUSTR_22: &str = "level 22: the catacombs";
pub const HUSTR_23: &str = "level 23: barrels o' fun";
pub const HUSTR_24: &str = "level 24: the chasm";
pub const HUSTR_25: &str = "level 25: bloodfalls";
pub const HUSTR_26: &str = "level 26: the abandoned mines";
pub const HUSTR_27: &str = "level 27: monster condo";
pub const HUSTR_28: &str = "level 28: the spirit world";
pub const HUSTR_29: &str = "level 29: the living end";
pub const HUSTR_30: &str = "level 30: icon of sin";
pub const HUSTR_31: &str = "level 31: wolfenstein";
pub const HUSTR_32: &str = "level 32: grosse";
pub const PHUSTR_1: &str = "level 1: congo";
pub const PHUSTR_2: &str = "level 2: well of souls";
pub const PHUSTR_3: &str = "level 3: aztec";
pub const PHUSTR_4: &str = "level 4: caged";
pub const PHUSTR_5: &str = "level 5: ghost town";
pub const PHUSTR_6: &str = "level 6: baron's lair";
pub const PHUSTR_7: &str = "level 7: caughtyard";
pub const PHUSTR_8: &str = "level 8: realm";
pub const PHUSTR_9: &str = "level 9: abattoire";
pub const PHUSTR_10: &str = "level 10: onslaught";
pub const PHUSTR_11: &str = "level 11: hunted";
pub const PHUSTR_12: &str = "level 12: speed";
pub const PHUSTR_13: &str = "level 13: the crypt";
pub const PHUSTR_14: &str = "level 14: genesis";
pub const PHUSTR_15: &str = "level 15: the twilight";
pub const PHUSTR_16: &str = "level 16: the omen";
pub const PHUSTR_17: &str = "level 17: compound";
pub const PHUSTR_18: &str = "level 18: neurosphere";
pub const PHUSTR_19: &str = "level 19: nme";
pub const PHUSTR_20: &str = "level 20: the death domain";
pub const PHUSTR_21: &str = "level 21: slayer";
pub const PHUSTR_22: &str = "level 22: impossible mission";
pub const PHUSTR_23: &str = "level 23: tombstone";
pub const PHUSTR_24: &str = "level 24: the final frontier";
pub const PHUSTR_25: &str = "level 25: the temple of darkness";
pub const PHUSTR_26: &str = "level 26: bunker";
pub const PHUSTR_27: &str = "level 27: anti-christ";
pub const PHUSTR_28: &str = "level 28: the sewers";
pub const PHUSTR_29: &str = "level 29: odyssey of noises";
pub const PHUSTR_30: &str = "level 30: the gateway of hell";
pub const PHUSTR_31: &str = "level 31: cyberden";
pub const PHUSTR_32: &str = "level 32: go 2 it";
pub const THUSTR_1: &str = "level 1: system control";
pub const THUSTR_2: &str = "level 2: human bbq";
pub const THUSTR_3: &str = "level 3: power control";
pub const THUSTR_4: &str = "level 4: wormhole";
pub const THUSTR_5: &str = "level 5: hanger";
pub const THUSTR_6: &str = "level 6: open season";
pub const THUSTR_7: &str = "level 7: prison";
pub const THUSTR_8: &str = "level 8: metal";
pub const THUSTR_9: &str = "level 9: stronghold";
pub const THUSTR_10: &str = "level 10: redemption";
pub const THUSTR_11: &str = "level 11: storage facility";
pub const THUSTR_12: &str = "level 12: crater";
pub const THUSTR_13: &str = "level 13: nukage processing";
pub const THUSTR_14: &str = "level 14: steel works";
pub const THUSTR_15: &str = "level 15: dead zone";
pub const THUSTR_16: &str = "level 16: deepest reaches";
pub const THUSTR_17: &str = "level 17: processing area";
pub const THUSTR_18: &str = "level 18: mill";
pub const THUSTR_19: &str = "level 19: shipping/respawning";
pub const THUSTR_20: &str = "level 20: central processing";
pub const THUSTR_21: &str = "level 21: administration center";
pub const THUSTR_22: &str = "level 22: habitat";
pub const THUSTR_23: &str = "level 23: lunar mining project";
pub const THUSTR_24: &str = "level 24: quarry";
pub const THUSTR_25: &str = "level 25: baron's den";
pub const THUSTR_26: &str = "level 26: ballistyx";
pub const THUSTR_27: &str = "level 27: mount pain";
pub const THUSTR_28: &str = "level 28: heck";
pub const THUSTR_29: &str = "level 29: river styx";
pub const THUSTR_30: &str = "level 30: last call";
pub const THUSTR_31: &str = "level 31: pharaoh";
pub const THUSTR_32: &str = "level 32: caribbean";
pub const HUSTR_PLRGREEN: FixedCStr<8> = FixedCStr(*b"Green: \0");
pub const HUSTR_PLRINDIGO: FixedCStr<9> = FixedCStr(*b"Indigo: \0");
pub const HUSTR_PLRBROWN: FixedCStr<8> = FixedCStr(*b"Brown: \0");
pub const HUSTR_PLRRED: FixedCStr<6> = FixedCStr(*b"Red: \0");
pub const HU_TITLEX: i32 = 0;
pub const HU_INPUTX: i32 = HU_MSGX;
#[no_mangle]
pub static chat_char: u8 = 0;
const fn new_hu_itext_t() -> hu_itext_t {
    hu_itext_t {
        l: hu_textline_t {
            x: 0,
            y: 0,
            f: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
            sc: 0,
            l: String::new(),
            needsupdate: 0,
        },
        lm: 0,
        on: ::core::ptr::null::<bool>() as *mut bool,
        laston: false,
    }
}
const fn new_hu_textline_t() -> hu_textline_t {
    hu_textline_t {
        x: 0,
        y: 0,
        f: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
        sc: 0,
        l: String::new(),
        needsupdate: 0,
    }
}
pub static mapnames: [&str; 45] = [
    HUSTR_E1M1, HUSTR_E1M2, HUSTR_E1M3, HUSTR_E1M4, HUSTR_E1M5, HUSTR_E1M6, HUSTR_E1M7, HUSTR_E1M8,
    HUSTR_E1M9, HUSTR_E2M1, HUSTR_E2M2, HUSTR_E2M3, HUSTR_E2M4, HUSTR_E2M5, HUSTR_E2M6, HUSTR_E2M7,
    HUSTR_E2M8, HUSTR_E2M9, HUSTR_E3M1, HUSTR_E3M2, HUSTR_E3M3, HUSTR_E3M4, HUSTR_E3M5, HUSTR_E3M6,
    HUSTR_E3M7, HUSTR_E3M8, HUSTR_E3M9, HUSTR_E4M1, HUSTR_E4M2, HUSTR_E4M3, HUSTR_E4M4, HUSTR_E4M5,
    HUSTR_E4M6, HUSTR_E4M7, HUSTR_E4M8, HUSTR_E4M9, "NEWLEVEL", "NEWLEVEL", "NEWLEVEL", "NEWLEVEL",
    "NEWLEVEL", "NEWLEVEL", "NEWLEVEL", "NEWLEVEL", "NEWLEVEL",
];
pub static mapnames_commercial: [&str; 96] = [
    HUSTR_1, HUSTR_2, HUSTR_3, HUSTR_4, HUSTR_5, HUSTR_6, HUSTR_7, HUSTR_8, HUSTR_9, HUSTR_10,
    HUSTR_11, HUSTR_12, HUSTR_13, HUSTR_14, HUSTR_15, HUSTR_16, HUSTR_17, HUSTR_18, HUSTR_19,
    HUSTR_20, HUSTR_21, HUSTR_22, HUSTR_23, HUSTR_24, HUSTR_25, HUSTR_26, HUSTR_27, HUSTR_28,
    HUSTR_29, HUSTR_30, HUSTR_31, HUSTR_32, PHUSTR_1, PHUSTR_2, PHUSTR_3, PHUSTR_4, PHUSTR_5,
    PHUSTR_6, PHUSTR_7, PHUSTR_8, PHUSTR_9, PHUSTR_10, PHUSTR_11, PHUSTR_12, PHUSTR_13, PHUSTR_14,
    PHUSTR_15, PHUSTR_16, PHUSTR_17, PHUSTR_18, PHUSTR_19, PHUSTR_20, PHUSTR_21, PHUSTR_22,
    PHUSTR_23, PHUSTR_24, PHUSTR_25, PHUSTR_26, PHUSTR_27, PHUSTR_28, PHUSTR_29, PHUSTR_30,
    PHUSTR_31, PHUSTR_32, THUSTR_1, THUSTR_2, THUSTR_3, THUSTR_4, THUSTR_5, THUSTR_6, THUSTR_7,
    THUSTR_8, THUSTR_9, THUSTR_10, THUSTR_11, THUSTR_12, THUSTR_13, THUSTR_14, THUSTR_15,
    THUSTR_16, THUSTR_17, THUSTR_18, THUSTR_19, THUSTR_20, THUSTR_21, THUSTR_22, THUSTR_23,
    THUSTR_24, THUSTR_25, THUSTR_26, THUSTR_27, THUSTR_28, THUSTR_29, THUSTR_30, THUSTR_31,
    THUSTR_32,
];
pub unsafe fn HU_Init(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    j = HU_FONTSTART;
    i = 0 as i32;
    while i < HU_FONTSIZE {
        let fresh0 = j;
        j = j + 1;
        let buffer = format!("STCFN{:03}", fresh0);
        state.hu_stuff.hu_font[i as usize] =
            W_CacheLumpName(state, &buffer, PU_STATIC as i32) as *mut patch_t;
        i += 1;
    }
}
pub fn HU_Stop(state: &mut GameState) {
    state.hu_stuff.headsupactive = false;
}
pub unsafe fn HU_Start(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut s: &str = "";
    if state.hu_stuff.headsupactive {
        HU_Stop(state);
    }
    state.hu_stuff.plr = PlayerId(state.g_game.consoleplayer as u8);
    state.hu_stuff.message_on = false;
    state.hu_stuff.message_dontfuckwithme = false;
    state.hu_stuff.message_nottobefuckedwith = false;
    state.hu_stuff.chat_on = false;
    HUlib_initSText(
        &raw mut state.hu_stuff.w_message,
        HU_MSGX,
        HU_MSGY,
        HU_MSGHEIGHT,
        &raw mut state.hu_stuff.hu_font as *mut *mut patch_t,
        HU_FONTSTART,
        &raw mut state.hu_stuff.message_on,
    );
    HUlib_initTextLine(
        &raw mut state.hu_stuff.w_title,
        HU_TITLEX,
        167 as i32 - (*state.hu_stuff.hu_font[0]).height as i32,
        &raw mut state.hu_stuff.hu_font as *mut *mut patch_t,
        HU_FONTSTART,
    );
    match if state.doomstat.gamemission as u32 == GameMission_t::pack_chex as i32 as u32 {
        GameMission_t::doom as i32 as u32
    } else if state.doomstat.gamemission as u32 == GameMission_t::pack_hacx as i32 as u32 {
        GameMission_t::doom2 as i32 as u32
    } else {
        state.doomstat.gamemission as u32
    } {
        0 => {
            s = mapnames[((state.g_game.gameepisode - 1 as i32) * 9 as i32 + state.g_game.gamemap
                - 1 as i32) as usize];
        }
        1 => {
            s = mapnames_commercial[(state.g_game.gamemap - 1 as i32) as usize];
        }
        3 => {
            s = mapnames_commercial[(state.g_game.gamemap - 1 as i32 + 32 as i32) as usize];
        }
        2 => {
            s = mapnames_commercial[(state.g_game.gamemap - 1 as i32 + 64 as i32) as usize];
        }
        _ => {
            s = "Unknown level";
        }
    }
    if state.doomstat.gameversion  == GameVersion::chex {
        s = mapnames[(state.g_game.gamemap - 1 as i32) as usize];
    }
    for b in s.bytes() {
        HUlib_addCharToTextLine(&raw mut state.hu_stuff.w_title, b);
    }
    HUlib_initIText(
        &raw mut state.hu_stuff.w_chat,
        HU_INPUTX,
        HU_MSGY
            + HU_MSGHEIGHT
                * ((*state.hu_stuff.hu_font[0]).height as i32 + 1 as i32),
        &raw mut state.hu_stuff.hu_font as *mut *mut patch_t,
        HU_FONTSTART,
        &raw mut state.hu_stuff.chat_on,
    );
    i = 0 as i32;
    while i < MAXPLAYERS {
        HUlib_initIText(
            (&raw mut state.hu_stuff.w_inputbuffer as *mut hu_itext_t).offset(i as isize)
                as *mut hu_itext_t,
            0 as i32,
            0 as i32,
            ::core::ptr::null_mut::<*mut patch_t>(),
            0 as i32,
            &raw mut state.hu_stuff.always_off,
        );
        i += 1;
    }
    state.hu_stuff.headsupactive = true;
}
pub unsafe fn HU_Drawer(state: &mut GameState) {
    let w_message = &raw mut state.hu_stuff.w_message;
    HUlib_drawSText(state, w_message);
    let w_chat = &raw mut state.hu_stuff.w_chat;
    HUlib_drawIText(state, w_chat);
    if state.am_map.automapactive {
        let w_title = &raw mut state.hu_stuff.w_title;
        HUlib_drawTextLine(state, w_title, false);
    }
}
pub unsafe fn HU_Erase(state: &mut GameState) {
    let w_message = &raw mut state.hu_stuff.w_message;
    HUlib_eraseSText(state, w_message);
    let w_chat = &raw mut state.hu_stuff.w_chat;
    HUlib_eraseIText(state, w_chat);
    let w_title = &raw mut state.hu_stuff.w_title;
    HUlib_eraseTextLine(state, w_title);
}
pub unsafe fn HU_Ticker(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut rc: i32 = 0;
    let mut c: u8 = 0;
    if state.hu_stuff.message_counter != 0 && {
        state.hu_stuff.message_counter -= 1;
        state.hu_stuff.message_counter == 0
    } {
        state.hu_stuff.message_on = false;
        state.hu_stuff.message_nottobefuckedwith = false;
    }
    if state.m_menu.showMessages != 0 || state.hu_stuff.message_dontfuckwithme {
        if (*state.g_game.player_mut(state.hu_stuff.plr)).message.is_some() && !state.hu_stuff.message_nottobefuckedwith
            || (*state.g_game.player_mut(state.hu_stuff.plr)).message.is_some() && state.hu_stuff.message_dontfuckwithme
        {
            HUlib_addMessageToSText(
                &raw mut state.hu_stuff.w_message,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                (*state.g_game.player_mut(state.hu_stuff.plr)).message.as_deref().unwrap(),
            );
            (*state.g_game.player_mut(state.hu_stuff.plr)).message = None;
            state.hu_stuff.message_on = true;
            state.hu_stuff.message_counter = HU_MSGTIMEOUT;
            state.hu_stuff.message_nottobefuckedwith = state.hu_stuff.message_dontfuckwithme;
            state.hu_stuff.message_dontfuckwithme = false;
        }
    }
    if state.g_game.netgame {
        i = 0 as i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                if i != state.g_game.consoleplayer && {
                    c = state.g_game.players[i as usize].cmd.chatchar;
                    c as i32 != 0
                } {
                    if c as i32 <= HU_BROADCAST {
                        state.hu_stuff.chat_dest[i as usize] = c;
                    } else {
                        rc = HUlib_keyInIText(
                            (&raw mut state.hu_stuff.w_inputbuffer as *mut hu_itext_t)
                                .offset(i as isize) as *mut hu_itext_t,
                            c as u8,
                        ) as i32;
                        if rc != 0 && c as i32 == KEY_ENTER {
                            if !state.hu_stuff.w_inputbuffer[i as usize].l.l.is_empty()
                                && (state.hu_stuff.chat_dest[i as usize] as i32
                                    == state.g_game.consoleplayer + 1 as i32
                                    || state.hu_stuff.chat_dest[i as usize] as i32 == HU_BROADCAST)
                            {
                                HUlib_addMessageToSText(
                                    &raw mut state.hu_stuff.w_message,
                                    state.hu_stuff.player_names[i as usize],
                                    &state.hu_stuff.w_inputbuffer[i as usize].l.l,
                                );
                                state.hu_stuff.message_nottobefuckedwith = true;
                                state.hu_stuff.message_on = true;
                                state.hu_stuff.message_counter = HU_MSGTIMEOUT;
                                if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
                                    S_StartSound(state, SoundOrigin::None, sfx_radio as i32);
                                } else {
                                    S_StartSound(state, SoundOrigin::None, sfx_tink as i32);
                                }
                            }
                            HUlib_resetIText(
                                (&raw mut state.hu_stuff.w_inputbuffer as *mut hu_itext_t)
                                    .offset(i as isize)
                                    as *mut hu_itext_t,
                            );
                        }
                    }
                    state.g_game.players[i as usize].cmd.chatchar = 0 as byte;
                }
            }
            i += 1;
        }
    }
}
pub const QUEUESIZE: i32 = 128;
pub unsafe fn HU_queueChatChar(state: &mut GameState, c: u8) {
    if state.hu_stuff.head + 1 as i32 & QUEUESIZE - 1 as i32 == state.hu_stuff.tail {
        (*state.g_game.player_mut(state.hu_stuff.plr)).message = Some("[Message unsent]".to_string());
    } else {
        state.hu_stuff.chatchars[state.hu_stuff.head as usize] = c;
        state.hu_stuff.head = state.hu_stuff.head + 1 as i32 & QUEUESIZE - 1 as i32;
    };
}
pub fn HU_dequeueChatChar(state: &mut HuStuffState) -> u8 {
    let mut c: u8 = 0;
    if state.head != state.tail {
        c = state.chatchars[state.tail as usize];
        state.tail = state.tail + 1 as i32 & QUEUESIZE - 1 as i32;
    } else {
        c = 0 as u8;
    }
    return c;
}
pub unsafe fn HU_Responder(state: &mut GameState, mut ev: &event_t) -> bool {
    let mut eatkey: bool = false;
    let mut c: u8 = 0;
    let mut i: i32 = 0;
    let mut numplayers: i32 = 0;
    numplayers = 0 as i32;
    i = 0 as i32;
    while i < MAXPLAYERS {
        numplayers += state.g_game.playeringame[i as usize] as i32;
        i += 1;
    }
    if (*ev).data1 == KEY_RSHIFT {
        return false;
    } else if (*ev).data1 == KEY_RALT || (*ev).data1 == KEY_LALT {
        state.hu_stuff.hu_responder_altdown = (*ev).type_0 == EvType::ev_keydown;
        return false;
    }
    if (*ev).type_0 != EvType::ev_keydown {
        return false;
    }
    if !state.hu_stuff.chat_on {
        if (*ev).data1 == state.m_controls.key_message_refresh {
            state.hu_stuff.message_on = true;
            state.hu_stuff.message_counter = HU_MSGTIMEOUT;
            eatkey = true;
        } else if state.g_game.netgame && (*ev).data2 == state.m_controls.key_multi_msg {
            state.hu_stuff.chat_on = true;
            eatkey = state.hu_stuff.chat_on;
            HUlib_resetIText(&raw mut state.hu_stuff.w_chat);
            HU_queueChatChar(state, HU_BROADCAST as u8);
        } else if state.g_game.netgame && numplayers > 2 as i32 {
            i = 0 as i32;
            while i < MAXPLAYERS {
                if (*ev).data2 == state.m_controls.key_multi_msgplayer[i as usize] {
                    if state.g_game.playeringame[i as usize] && i != state.g_game.consoleplayer
                    {
                        state.hu_stuff.chat_on = true;
                        eatkey = state.hu_stuff.chat_on;
                        HUlib_resetIText(&raw mut state.hu_stuff.w_chat);
                        HU_queueChatChar(state, (i + 1 as i32) as u8);
                        break;
                    } else if i == state.g_game.consoleplayer {
                        state.hu_stuff.hu_responder_num_nobrainers += 1;
                        if state.hu_stuff.hu_responder_num_nobrainers < 3 as i32 {
                            (*state.g_game.player_mut(state.hu_stuff.plr)).message =
                                Some("You mumble to yourself".to_string());
                        } else if state.hu_stuff.hu_responder_num_nobrainers < 6 as i32 {
                            (*state.g_game.player_mut(state.hu_stuff.plr)).message = Some("Who's there?".to_string());
                        } else if state.hu_stuff.hu_responder_num_nobrainers < 9 as i32 {
                            (*state.g_game.player_mut(state.hu_stuff.plr)).message =
                                Some("You scare yourself".to_string());
                        } else if state.hu_stuff.hu_responder_num_nobrainers < 32 as i32 {
                            (*state.g_game.player_mut(state.hu_stuff.plr)).message = Some("You start to rave".to_string());
                        } else {
                            (*state.g_game.player_mut(state.hu_stuff.plr)).message =
                                Some("You've lost it...".to_string());
                        }
                    }
                }
                i += 1;
            }
        }
    } else if state.hu_stuff.hu_responder_altdown {
        c = ((*ev).data1 - '0' as i32) as u8;
        if c as i32 > 9 as i32 {
            return false;
        }
        let macromessage = state.hu_stuff.chat_macros[c as usize];
        HU_queueChatChar(state, KEY_ENTER as u8);
        for b in macromessage.bytes() {
            HU_queueChatChar(state, b);
        }
        HU_queueChatChar(state, KEY_ENTER as u8);
        state.hu_stuff.chat_on = false;
        (*state.g_game.player_mut(state.hu_stuff.plr)).message = Some(macromessage.to_string());
        eatkey = true;
    } else {
        c = (*ev).data2 as u8;
        eatkey = HUlib_keyInIText(&raw mut state.hu_stuff.w_chat, c);
        if eatkey {
            HU_queueChatChar(state, c);
        }
        if c as i32 == KEY_ENTER {
            state.hu_stuff.chat_on = false;
            if !state.hu_stuff.w_chat.l.l.is_empty() {
                (*state.g_game.player_mut(state.hu_stuff.plr)).message = Some(state.hu_stuff.w_chat.l.l.clone());
            }
        } else if c as i32 == KEY_ESCAPE {
            state.hu_stuff.chat_on = false;
        }
    }
    return eatkey;
}
