use crate::d_event::EvType;
use crate::d_event::Event;
use crate::d_mode::GameMission;
use crate::d_mode::GameMode;
use crate::d_mode::GameVersion;
use crate::d_player::PlayerId;
use crate::doomdef::MAXPLAYERS;
use crate::doomdef::TICRATE;
use crate::filesystem::DoomFileSystem;
use crate::g_game::GGameState;
use crate::game_state::GameState;
use crate::hu_lib::{
    hulib_add_char_to_text_line, hulib_add_message_to_stext, hulib_draw_itext, hulib_draw_stext,
    hulib_draw_text_line, hulib_erase_itext, hulib_erase_stext, hulib_erase_text_line,
    hulib_init_itext, hulib_init_stext, hulib_init_text_line, hulib_key_in_itext,
    hulib_reset_itext, HuIText, HuSText, HuTextLine,
};
use crate::m_controls::MControlsState;
use crate::m_controls::KEY_ENTER;
use crate::m_controls::KEY_ESCAPE;
use crate::m_controls::KEY_RALT;
use crate::m_controls::KEY_RSHIFT;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::v_video::cache_patch_num;
use crate::w_wad::WWadState;
use crate::w_wad::{get_num_for_name, lump_bytes};
use alloc::string::String;
use alloc::string::ToString;

pub struct HuStuffState {
    pub plr: PlayerId,
    pub hu_font: [i32; 63],
    pub w_title: HuTextLine,
    pub chat_on: bool,
    pub w_chat: HuIText,
    pub chat_dest: [u8; 4],
    pub w_inputbuffer: [HuIText; 4],
    pub message_on: bool,
    pub message_dontfuckwithme: bool,
    pub message_nottobefuckedwith: bool,
    pub w_message: HuSText,
    pub message_counter: i32,
    pub headsupactive: bool,
    pub chatchars: [u8; 128],
    pub head: i32,
    pub tail: i32,
    pub chat_macros: [Option<&'static str>; 10],
    pub hu_responder_altdown: bool,
    pub hu_responder_num_nobrainers: i32,
}

impl Default for HuStuffState {
    fn default() -> Self {
        Self::new()
    }
}

impl HuStuffState {
    pub const fn new() -> Self {
        Self {
            plr: PlayerId(0),
            hu_font: [-1; 63],
            w_title: HuTextLine {
                x: 0,
                y: 0,
                sc: 0,
                l: String::new(),
                needsupdate: 0,
            },
            chat_on: false,
            w_chat: HuIText {
                l: HuTextLine {
                    x: 0,
                    y: 0,
                    sc: 0,
                    l: String::new(),
                    needsupdate: 0,
                },
                lm: 0,
                laston: false,
            },
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
            w_message: HuSText {
                l: [
                    new_hu_textline_t(),
                    new_hu_textline_t(),
                    new_hu_textline_t(),
                    new_hu_textline_t(),
                ],
                h: 0,
                cl: 0,
                laston: false,
            },
            message_counter: 0,
            headsupactive: false,
            chatchars: [0; 128],
            head: 0,
            tail: 0,
            chat_macros: [
                Some("No"),
                Some("I'm ready to kick butt!"),
                Some("I'm OK."),
                Some("I'm not looking too good!"),
                Some("Help!"),
                Some("You suck!"),
                Some("Next time, scumbag..."),
                Some("Come here!"),
                Some("I'll take care of it."),
                Some("Yes"),
            ],
            hu_responder_altdown: false,
            hu_responder_num_nobrainers: 0,
        }
    }
}

pub const KEY_LALT: i32 = KEY_RALT;
pub const HU_FONTSTART: i32 = '!' as i32;
pub const HU_FONTEND: i32 = '_' as i32;
pub const HU_FONTSIZE: i32 = HU_FONTEND - HU_FONTSTART + 1;
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
pub const PLAYER_NAMES: [&str; 4] = ["Green: ", "Indigo: ", "Brown: ", "Red: "];
pub const HU_TITLEX: i32 = 0;
pub const HU_INPUTX: i32 = HU_MSGX;
const fn new_hu_itext_t() -> HuIText {
    HuIText {
        l: HuTextLine {
            x: 0,
            y: 0,
            sc: 0,
            l: String::new(),
            needsupdate: 0,
        },
        lm: 0,
        laston: false,
    }
}
const fn new_hu_textline_t() -> HuTextLine {
    HuTextLine {
        x: 0,
        y: 0,
        sc: 0,
        l: String::new(),
        needsupdate: 0,
    }
}
pub static MAPNAMES: [&str; 45] = [
    HUSTR_E1M1, HUSTR_E1M2, HUSTR_E1M3, HUSTR_E1M4, HUSTR_E1M5, HUSTR_E1M6, HUSTR_E1M7, HUSTR_E1M8,
    HUSTR_E1M9, HUSTR_E2M1, HUSTR_E2M2, HUSTR_E2M3, HUSTR_E2M4, HUSTR_E2M5, HUSTR_E2M6, HUSTR_E2M7,
    HUSTR_E2M8, HUSTR_E2M9, HUSTR_E3M1, HUSTR_E3M2, HUSTR_E3M3, HUSTR_E3M4, HUSTR_E3M5, HUSTR_E3M6,
    HUSTR_E3M7, HUSTR_E3M8, HUSTR_E3M9, HUSTR_E4M1, HUSTR_E4M2, HUSTR_E4M3, HUSTR_E4M4, HUSTR_E4M5,
    HUSTR_E4M6, HUSTR_E4M7, HUSTR_E4M8, HUSTR_E4M9, "NEWLEVEL", "NEWLEVEL", "NEWLEVEL", "NEWLEVEL",
    "NEWLEVEL", "NEWLEVEL", "NEWLEVEL", "NEWLEVEL", "NEWLEVEL",
];
pub static MAPNAMES_COMMERCIAL: [&str; 96] = [
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
pub fn hu_init(fs: &dyn DoomFileSystem, hu_stuff: &mut HuStuffState, w_wad: &mut WWadState) {
    for (i, code) in (HU_FONTSTART..HU_FONTSTART + HU_FONTSIZE).enumerate() {
        let buffer = format!("STCFN{code:03}");
        let lumpnum = get_num_for_name(w_wad, &buffer);
        lump_bytes(fs, w_wad, lumpnum);
        hu_stuff.hu_font[i] = lumpnum;
    }
}
pub fn hu_stop(hu_stuff: &mut HuStuffState) {
    hu_stuff.headsupactive = false;
}
pub fn hu_start(state: &mut GameState) {
    let mut _i: i32 = 0;
    let mut s: &str;
    if state.ui.hu_stuff.headsupactive {
        hu_stop(&mut state.ui.hu_stuff);
    }
    state.ui.hu_stuff.plr = PlayerId(state.game.g_game.consoleplayer as u8);
    state.ui.hu_stuff.message_on = false;
    state.ui.hu_stuff.message_dontfuckwithme = false;
    state.ui.hu_stuff.message_nottobefuckedwith = false;
    state.ui.hu_stuff.chat_on = false;
    let hu_font0 = state.ui.hu_stuff.hu_font[0];
    let hu_font0_height =
        cache_patch_num(&*state.assets.fs, &mut state.assets.w_wad, hu_font0).height();
    hulib_init_stext(
        &mut state.ui.hu_stuff.w_message,
        HU_MSGX,
        HU_MSGY,
        HU_MSGHEIGHT,
        HU_FONTSTART,
        hu_font0_height,
    );
    hulib_init_text_line(
        &mut state.ui.hu_stuff.w_title,
        HU_TITLEX,
        167 - hu_font0_height,
        HU_FONTSTART,
    );
    match if state.game.doomstat.gamemission as u32 == GameMission::PackChex as i32 as u32 {
        GameMission::Doom as i32 as u32
    } else if state.game.doomstat.gamemission as u32 == GameMission::PackHacx as i32 as u32 {
        GameMission::Doom2 as i32 as u32
    } else {
        state.game.doomstat.gamemission as u32
    } {
        0 => {
            s = MAPNAMES[((state.game.g_game.gameepisode - 1) * 9 + state.game.g_game.gamemap - 1)
                as usize];
        }
        1 => {
            s = MAPNAMES_COMMERCIAL[(state.game.g_game.gamemap - 1) as usize];
        }
        3 => {
            s = MAPNAMES_COMMERCIAL[(state.game.g_game.gamemap - 1 + 32) as usize];
        }
        2 => {
            s = MAPNAMES_COMMERCIAL[(state.game.g_game.gamemap - 1 + 64) as usize];
        }
        _ => {
            s = "Unknown level";
        }
    }
    if state.game.doomstat.gameversion == GameVersion::Chex {
        s = MAPNAMES[(state.game.g_game.gamemap - 1) as usize];
    }
    for b in s.bytes() {
        hulib_add_char_to_text_line(&mut state.ui.hu_stuff.w_title, b);
    }
    hulib_init_itext(
        &mut state.ui.hu_stuff.w_chat,
        HU_INPUTX,
        HU_MSGY + HU_MSGHEIGHT * (hu_font0_height + 1),
        HU_FONTSTART,
    );
    for i in 0..(MAXPLAYERS as usize) {
        hulib_init_itext(&mut state.ui.hu_stuff.w_inputbuffer[i], 0, 0, 0);
    }
    state.ui.hu_stuff.headsupactive = true;
}
pub fn hu_drawer(state: &mut GameState) {
    let message_on = state.ui.hu_stuff.message_on;
    let w_message = state.ui.hu_stuff.w_message.clone();
    hulib_draw_stext(state, &w_message, message_on);
    let chat_on = state.ui.hu_stuff.chat_on;
    let w_chat = state.ui.hu_stuff.w_chat.clone();
    hulib_draw_itext(state, &w_chat, chat_on);
    if state.ui.am_map.automapactive {
        let w_title = state.ui.hu_stuff.w_title.clone();
        hulib_draw_text_line(state, &w_title, false);
    }
}
pub fn erase(state: &mut GameState) {
    let message_on = state.ui.hu_stuff.message_on;
    let mut w_message = state.ui.hu_stuff.w_message.clone();
    hulib_erase_stext(state, &mut w_message, message_on);
    state.ui.hu_stuff.w_message = w_message;
    let chat_on = state.ui.hu_stuff.chat_on;
    let mut w_chat = state.ui.hu_stuff.w_chat.clone();
    hulib_erase_itext(state, &mut w_chat, chat_on);
    state.ui.hu_stuff.w_chat = w_chat;
    let mut w_title = state.ui.hu_stuff.w_title.clone();
    hulib_erase_text_line(state, &mut w_title);
    state.ui.hu_stuff.w_title = w_title;
}
pub fn hu_ticker(state: &mut GameState) {
    let mut rc: i32;
    let mut c: u8;
    if state.ui.hu_stuff.message_counter != 0 && {
        state.ui.hu_stuff.message_counter -= 1;
        state.ui.hu_stuff.message_counter == 0
    } {
        state.ui.hu_stuff.message_on = false;
        state.ui.hu_stuff.message_nottobefuckedwith = false;
    }
    if (state.ui.m_menu.show_messages != 0 || state.ui.hu_stuff.message_dontfuckwithme)
        && (state
            .game
            .g_game
            .player_mut(state.ui.hu_stuff.plr)
            .message
            .is_some()
            && !state.ui.hu_stuff.message_nottobefuckedwith
            || state
                .game
                .g_game
                .player_mut(state.ui.hu_stuff.plr)
                .message
                .is_some()
                && state.ui.hu_stuff.message_dontfuckwithme)
    {
        hulib_add_message_to_stext(
            &mut state.ui.hu_stuff.w_message,
            None,
            state
                .game
                .g_game
                .player_mut(state.ui.hu_stuff.plr)
                .message
                .as_deref()
                .unwrap(),
        );
        state.game.g_game.player_mut(state.ui.hu_stuff.plr).message = None;
        state.ui.hu_stuff.message_on = true;
        state.ui.hu_stuff.message_counter = HU_MSGTIMEOUT;
        state.ui.hu_stuff.message_nottobefuckedwith = state.ui.hu_stuff.message_dontfuckwithme;
        state.ui.hu_stuff.message_dontfuckwithme = false;
    }
    if state.game.g_game.netgame {
        for i in 0..MAXPLAYERS {
            if state.game.g_game.playeringame[i as usize]
                && i != state.game.g_game.consoleplayer
                && {
                    c = state.game.g_game.players[i as usize].cmd.chatchar;
                    c as i32 != 0
                }
            {
                if c as i32 <= HU_BROADCAST {
                    state.ui.hu_stuff.chat_dest[i as usize] = c;
                } else {
                    rc = hulib_key_in_itext(&mut state.ui.hu_stuff.w_inputbuffer[i as usize], c)
                        as i32;
                    if rc != 0 && c as i32 == KEY_ENTER {
                        if !state.ui.hu_stuff.w_inputbuffer[i as usize].l.l.is_empty()
                            && (state.ui.hu_stuff.chat_dest[i as usize] as i32
                                == state.game.g_game.consoleplayer + 1
                                || state.ui.hu_stuff.chat_dest[i as usize] as i32 == HU_BROADCAST)
                        {
                            hulib_add_message_to_stext(
                                &mut state.ui.hu_stuff.w_message,
                                Some(PLAYER_NAMES[i as usize]),
                                &state.ui.hu_stuff.w_inputbuffer[i as usize].l.l,
                            );
                            state.ui.hu_stuff.message_nottobefuckedwith = true;
                            state.ui.hu_stuff.message_on = true;
                            state.ui.hu_stuff.message_counter = HU_MSGTIMEOUT;
                            if state.game.doomstat.gamemode as u32
                                == GameMode::Commercial as i32 as u32
                            {
                                s_start_sound(state, SoundOrigin::None, SfxName::Radio as i32);
                            } else {
                                s_start_sound(state, SoundOrigin::None, SfxName::Tink as i32);
                            }
                        }
                        hulib_reset_itext(&mut state.ui.hu_stuff.w_inputbuffer[i as usize]);
                    }
                }
                state.game.g_game.players[i as usize].cmd.chatchar = 0_u8;
            }
        }
    }
}
pub const QUEUESIZE: i32 = 128;
pub fn queue_chat_char(g_game: &mut GGameState, hu_stuff: &mut HuStuffState, c: u8) {
    if (hu_stuff.head + 1) & (QUEUESIZE - 1) == hu_stuff.tail {
        g_game.player_mut(hu_stuff.plr).message = Some("[Message unsent]".to_string());
    } else {
        hu_stuff.chatchars[hu_stuff.head as usize] = c;
        hu_stuff.head = (hu_stuff.head + 1) & (QUEUESIZE - 1);
    }
}
pub fn dequeue_chat_char(state: &mut HuStuffState) -> u8 {
    let c: u8;
    if state.head == state.tail {
        c = 0;
    } else {
        c = state.chatchars[state.tail as usize];
        state.tail = (state.tail + 1) & (QUEUESIZE - 1);
    }
    c
}
pub fn hu_responder(
    g_game: &mut GGameState,
    hu_stuff: &mut HuStuffState,
    m_controls: &MControlsState,
    ev: &Event,
) -> bool {
    let mut eatkey: bool = false;
    let c: u8;
    let mut numplayers: i32;
    numplayers = 0;
    for i in 0..(MAXPLAYERS as usize) {
        numplayers += g_game.playeringame[i] as i32;
    }
    if ev.data1 == KEY_RSHIFT {
        return false;
    } else if ev.data1 == KEY_RALT || ev.data1 == KEY_LALT {
        hu_stuff.hu_responder_altdown = ev.kind == EvType::Keydown;
        return false;
    }
    if ev.kind != EvType::Keydown {
        return false;
    }
    if !hu_stuff.chat_on {
        if ev.data1 == m_controls.key_message_refresh {
            hu_stuff.message_on = true;
            hu_stuff.message_counter = HU_MSGTIMEOUT;
            eatkey = true;
        } else if g_game.netgame && ev.data2 == m_controls.key_multi_msg {
            hu_stuff.chat_on = true;
            eatkey = hu_stuff.chat_on;
            hulib_reset_itext(&mut hu_stuff.w_chat);
            queue_chat_char(g_game, hu_stuff, HU_BROADCAST as u8);
        } else if g_game.netgame && numplayers > 2 {
            for i in 0..MAXPLAYERS {
                if ev.data2 == m_controls.key_multi_msgplayer[i as usize] {
                    if g_game.playeringame[i as usize] && i != g_game.consoleplayer {
                        hu_stuff.chat_on = true;
                        eatkey = hu_stuff.chat_on;
                        hulib_reset_itext(&mut hu_stuff.w_chat);
                        queue_chat_char(g_game, hu_stuff, (i + 1) as u8);
                        break;
                    } else if i == g_game.consoleplayer {
                        hu_stuff.hu_responder_num_nobrainers += 1;
                        if hu_stuff.hu_responder_num_nobrainers < 3 {
                            g_game.player_mut(hu_stuff.plr).message =
                                Some("You mumble to yourself".to_string());
                        } else if hu_stuff.hu_responder_num_nobrainers < 6 {
                            g_game.player_mut(hu_stuff.plr).message =
                                Some("Who's there?".to_string());
                        } else if hu_stuff.hu_responder_num_nobrainers < 9 {
                            g_game.player_mut(hu_stuff.plr).message =
                                Some("You scare yourself".to_string());
                        } else if hu_stuff.hu_responder_num_nobrainers < 32 {
                            g_game.player_mut(hu_stuff.plr).message =
                                Some("You start to rave".to_string());
                        } else {
                            g_game.player_mut(hu_stuff.plr).message =
                                Some("You've lost it...".to_string());
                        }
                    }
                }
            }
        }
    } else if hu_stuff.hu_responder_altdown {
        c = (ev.data1 - '0' as i32) as u8;
        if c as i32 > 9 {
            return false;
        }
        let macromessage = hu_stuff.chat_macros[c as usize].unwrap_or("");
        queue_chat_char(g_game, hu_stuff, KEY_ENTER as u8);
        for b in macromessage.bytes() {
            queue_chat_char(g_game, hu_stuff, b);
        }
        queue_chat_char(g_game, hu_stuff, KEY_ENTER as u8);
        hu_stuff.chat_on = false;
        g_game.player_mut(hu_stuff.plr).message = Some(macromessage.to_string());
        eatkey = true;
    } else {
        c = ev.data2 as u8;
        eatkey = hulib_key_in_itext(&mut hu_stuff.w_chat, c);
        if eatkey {
            queue_chat_char(g_game, hu_stuff, c);
        }
        if c as i32 == KEY_ENTER {
            hu_stuff.chat_on = false;
            if !hu_stuff.w_chat.l.l.is_empty() {
                g_game.player_mut(hu_stuff.plr).message = Some(hu_stuff.w_chat.l.l.clone());
            }
        } else if c as i32 == KEY_ESCAPE {
            hu_stuff.chat_on = false;
        }
    }
    eatkey
}
