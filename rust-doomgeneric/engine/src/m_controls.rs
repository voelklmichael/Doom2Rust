use crate::game_state::GameState;
use crate::m_config::M_BindVariable_int;

pub struct MControlsState {
    pub key_right: i32,
    pub key_left: i32,
    pub key_up: i32,
    pub key_down: i32,
    pub key_strafeleft: i32,
    pub key_straferight: i32,
    pub key_fire: i32,
    pub key_use: i32,
    pub key_strafe: i32,
    pub key_speed: i32,
    pub key_flyup: i32,
    pub key_flydown: i32,
    pub key_flycenter: i32,
    pub key_lookup: i32,
    pub key_lookdown: i32,
    pub key_lookcenter: i32,
    pub key_invleft: i32,
    pub key_invright: i32,
    pub key_useartifact: i32,
    pub key_jump: i32,
    pub key_arti_all: i32,
    pub key_arti_health: i32,
    pub key_arti_poisonbag: i32,
    pub key_arti_blastradius: i32,
    pub key_arti_teleport: i32,
    pub key_arti_teleportother: i32,
    pub key_arti_egg: i32,
    pub key_arti_invulnerability: i32,
    pub key_usehealth: i32,
    pub key_invquery: i32,
    pub key_mission: i32,
    pub key_invpop: i32,
    pub key_invkey: i32,
    pub key_invhome: i32,
    pub key_invend: i32,
    pub key_invuse: i32,
    pub key_invdrop: i32,
    pub mousebfire: i32,
    pub mousebstrafe: i32,
    pub mousebforward: i32,
    pub mousebjump: i32,
    pub mousebstrafeleft: i32,
    pub mousebstraferight: i32,
    pub mousebbackward: i32,
    pub mousebuse: i32,
    pub mousebprevweapon: i32,
    pub mousebnextweapon: i32,
    pub key_message_refresh: i32,
    pub key_pause: i32,
    pub key_demo_quit: i32,
    pub key_spy: i32,
    pub key_multi_msg: i32,
    pub key_multi_msgplayer: [i32; 8],
    pub key_weapon1: i32,
    pub key_weapon2: i32,
    pub key_weapon3: i32,
    pub key_weapon4: i32,
    pub key_weapon5: i32,
    pub key_weapon6: i32,
    pub key_weapon7: i32,
    pub key_weapon8: i32,
    pub key_prevweapon: i32,
    pub key_nextweapon: i32,
    pub key_map_north: i32,
    pub key_map_south: i32,
    pub key_map_east: i32,
    pub key_map_west: i32,
    pub key_map_zoomin: i32,
    pub key_map_zoomout: i32,
    pub key_map_toggle: i32,
    pub key_map_maxzoom: i32,
    pub key_map_follow: i32,
    pub key_map_grid: i32,
    pub key_map_mark: i32,
    pub key_map_clearmark: i32,
    pub key_menu_activate: i32,
    pub key_menu_up: i32,
    pub key_menu_down: i32,
    pub key_menu_left: i32,
    pub key_menu_right: i32,
    pub key_menu_back: i32,
    pub key_menu_forward: i32,
    pub key_menu_confirm: i32,
    pub key_menu_abort: i32,
    pub key_menu_help: i32,
    pub key_menu_save: i32,
    pub key_menu_load: i32,
    pub key_menu_volume: i32,
    pub key_menu_detail: i32,
    pub key_menu_qsave: i32,
    pub key_menu_endgame: i32,
    pub key_menu_messages: i32,
    pub key_menu_qload: i32,
    pub key_menu_quit: i32,
    pub key_menu_gamma: i32,
    pub key_menu_incscreen: i32,
    pub key_menu_decscreen: i32,
    pub key_menu_screenshot: i32,
    pub joybfire: i32,
    pub joybstrafe: i32,
    pub joybuse: i32,
    pub joybspeed: i32,
    pub joybstrafeleft: i32,
    pub joybstraferight: i32,
    pub joybjump: i32,
    pub joybprevweapon: i32,
    pub joybnextweapon: i32,
    pub joybmenu: i32,
    pub dclick_use: i32,
}

impl MControlsState {
    pub const fn new() -> Self {
        MControlsState {
            key_right: KEY_RIGHTARROW,
            key_left: KEY_LEFTARROW,
            key_up: KEY_UPARROW,
            key_down: KEY_DOWNARROW,
            key_strafeleft: KEY_STRAFE_L,
            key_straferight: KEY_STRAFE_R,
            key_fire: KEY_FIRE,
            key_use: KEY_USE,
            key_strafe: KEY_RALT,
            key_speed: KEY_RSHIFT,
            key_flyup: KEY_PGUP,
            key_flydown: KEY_INS,
            key_flycenter: KEY_HOME,
            key_lookup: KEY_PGDN,
            key_lookdown: KEY_DEL,
            key_lookcenter: KEY_END,
            key_invleft: '[' as i32,
            key_invright: ']' as i32,
            key_useartifact: KEY_ENTER,
            key_jump: '/' as i32,
            key_arti_all: KEY_BACKSPACE,
            key_arti_health: '\\' as i32,
            key_arti_poisonbag: '0' as i32,
            key_arti_blastradius: '9' as i32,
            key_arti_teleport: '8' as i32,
            key_arti_teleportother: '7' as i32,
            key_arti_egg: '6' as i32,
            key_arti_invulnerability: '5' as i32,
            key_usehealth: 'h' as i32,
            key_invquery: 'q' as i32,
            key_mission: 'w' as i32,
            key_invpop: 'z' as i32,
            key_invkey: 'k' as i32,
            key_invhome: KEY_HOME,
            key_invend: KEY_END,
            key_invuse: KEY_ENTER,
            key_invdrop: KEY_BACKSPACE,
            mousebfire: 0,
            mousebstrafe: 1,
            mousebforward: 2,
            mousebjump: -1,
            mousebstrafeleft: -1,
            mousebstraferight: -1,
            mousebbackward: -1,
            mousebuse: -1,
            mousebprevweapon: -1,
            mousebnextweapon: -1,
            key_message_refresh: KEY_ENTER,
            key_pause: KEY_PAUSE,
            key_demo_quit: 'q' as i32,
            key_spy: KEY_F12,
            key_multi_msg: 't' as i32,
            key_multi_msgplayer: [0; 8],
            key_weapon1: '1' as i32,
            key_weapon2: '2' as i32,
            key_weapon3: '3' as i32,
            key_weapon4: '4' as i32,
            key_weapon5: '5' as i32,
            key_weapon6: '6' as i32,
            key_weapon7: '7' as i32,
            key_weapon8: '8' as i32,
            key_prevweapon: 0,
            key_nextweapon: 0,
            key_map_north: KEY_UPARROW,
            key_map_south: KEY_DOWNARROW,
            key_map_east: KEY_RIGHTARROW,
            key_map_west: KEY_LEFTARROW,
            key_map_zoomin: '=' as i32,
            key_map_zoomout: '-' as i32,
            key_map_toggle: KEY_TAB,
            key_map_maxzoom: '0' as i32,
            key_map_follow: 'f' as i32,
            key_map_grid: 'g' as i32,
            key_map_mark: 'm' as i32,
            key_map_clearmark: 'c' as i32,
            key_menu_activate: KEY_ESCAPE,
            key_menu_up: KEY_UPARROW,
            key_menu_down: KEY_DOWNARROW,
            key_menu_left: KEY_LEFTARROW,
            key_menu_right: KEY_RIGHTARROW,
            key_menu_back: KEY_BACKSPACE,
            key_menu_forward: KEY_ENTER,
            key_menu_confirm: 'y' as i32,
            key_menu_abort: 'n' as i32,
            key_menu_help: KEY_F1,
            key_menu_save: KEY_F2,
            key_menu_load: KEY_F3,
            key_menu_volume: KEY_F4,
            key_menu_detail: KEY_F5,
            key_menu_qsave: KEY_F6,
            key_menu_endgame: KEY_F7,
            key_menu_messages: KEY_F8,
            key_menu_qload: KEY_F9,
            key_menu_quit: KEY_F10,
            key_menu_gamma: KEY_F11,
            key_menu_incscreen: KEY_EQUALS,
            key_menu_decscreen: KEY_MINUS,
            key_menu_screenshot: 0,
            joybfire: 0,
            joybstrafe: 1,
            joybuse: 3,
            joybspeed: 2,
            joybstrafeleft: -1,
            joybstraferight: -1,
            joybjump: -1,
            joybprevweapon: -1,
            joybnextweapon: -1,
            joybmenu: -1,
            dclick_use: 1,
        }
    }

    pub const fn weapon_keys(&self) -> [i32; 8] {
        [
            self.key_weapon1,
            self.key_weapon2,
            self.key_weapon3,
            self.key_weapon4,
            self.key_weapon5,
            self.key_weapon6,
            self.key_weapon7,
            self.key_weapon8,
        ]
    }
}

pub const KEY_RIGHTARROW: i32 = 0xae;
pub const KEY_LEFTARROW: i32 = 0xac;
pub const KEY_UPARROW: i32 = 0xad;
pub const KEY_DOWNARROW: i32 = 0xaf;
pub const KEY_STRAFE_L: i32 = 0xa0;
pub const KEY_STRAFE_R: i32 = 0xa1;
pub const KEY_USE: i32 = 0xa2;
pub const KEY_FIRE: i32 = 0xa3;
pub const KEY_ESCAPE: i32 = 27;
pub const KEY_ENTER: i32 = 13;
pub const KEY_TAB: i32 = 9;
pub const KEY_F1: i32 = 0x80 + 0x3b_i32;
pub const KEY_F2: i32 = 0x80 + 0x3c_i32;
pub const KEY_F3: i32 = 0x80 + 0x3d_i32;
pub const KEY_F4: i32 = 0x80 + 0x3e_i32;
pub const KEY_F5: i32 = 0x80 + 0x3f_i32;
pub const KEY_F6: i32 = 0x80 + 0x40_i32;
pub const KEY_F7: i32 = 0x80 + 0x41_i32;
pub const KEY_F8: i32 = 0x80 + 0x42_i32;
pub const KEY_F9: i32 = 0x80 + 0x43_i32;
pub const KEY_F10: i32 = 0x80 + 0x44_i32;
pub const KEY_F11: i32 = 0x80 + 0x57_i32;
pub const KEY_F12: i32 = 0x80 + 0x58_i32;
pub const KEY_BACKSPACE: i32 = 0x7f;
pub const KEY_PAUSE: i32 = 0xff;
pub const KEY_EQUALS: i32 = 0x3d;
pub const KEY_MINUS: i32 = 0x2d;
pub const KEY_RSHIFT: i32 = 0x80 + 0x36_i32;
pub const KEY_RALT: i32 = 0x80 + 0x38_i32;
pub const KEY_HOME: i32 = 0x80 + 0x47_i32;
pub const KEY_END: i32 = 0x80 + 0x4f_i32;
pub const KEY_PGUP: i32 = 0x80 + 0x49_i32;
pub const KEY_PGDN: i32 = 0x80 + 0x51_i32;
pub const KEY_INS: i32 = 0x80 + 0x52_i32;
pub const KEY_DEL: i32 = 0x80 + 0x53_i32;
pub fn M_BindBaseControls(state: &mut GameState) {
    M_BindVariable_int(&mut state.m_config, "key_right", &mut state.m_controls.key_right);
    M_BindVariable_int(&mut state.m_config, "key_left", &mut state.m_controls.key_left);
    M_BindVariable_int(&mut state.m_config, "key_up", &mut state.m_controls.key_up);
    M_BindVariable_int(&mut state.m_config, "key_down", &mut state.m_controls.key_down);
    M_BindVariable_int(&mut state.m_config, "key_strafeleft", &mut state.m_controls.key_strafeleft);
    M_BindVariable_int(&mut state.m_config, "key_straferight", &mut state.m_controls.key_straferight);
    M_BindVariable_int(&mut state.m_config, "key_fire", &mut state.m_controls.key_fire);
    M_BindVariable_int(&mut state.m_config, "key_use", &mut state.m_controls.key_use);
    M_BindVariable_int(&mut state.m_config, "key_strafe", &mut state.m_controls.key_strafe);
    M_BindVariable_int(&mut state.m_config, "key_speed", &mut state.m_controls.key_speed);
    M_BindVariable_int(&mut state.m_config, "mouseb_fire", &mut state.m_controls.mousebfire);
    M_BindVariable_int(&mut state.m_config, "mouseb_strafe", &mut state.m_controls.mousebstrafe);
    M_BindVariable_int(&mut state.m_config, "mouseb_forward", &mut state.m_controls.mousebforward);
    M_BindVariable_int(&mut state.m_config, "joyb_fire", &mut state.m_controls.joybfire);
    M_BindVariable_int(&mut state.m_config, "joyb_strafe", &mut state.m_controls.joybstrafe);
    M_BindVariable_int(&mut state.m_config, "joyb_use", &mut state.m_controls.joybuse);
    M_BindVariable_int(&mut state.m_config, "joyb_speed", &mut state.m_controls.joybspeed);
    M_BindVariable_int(&mut state.m_config, "joyb_menu_activate", &mut state.m_controls.joybmenu);
    M_BindVariable_int(&mut state.m_config, "joyb_strafeleft", &mut state.m_controls.joybstrafeleft);
    M_BindVariable_int(&mut state.m_config, "joyb_straferight", &mut state.m_controls.joybstraferight);
    M_BindVariable_int(&mut state.m_config, "mouseb_strafeleft", &mut state.m_controls.mousebstrafeleft);
    M_BindVariable_int(&mut state.m_config, "mouseb_straferight", &mut state.m_controls.mousebstraferight);
    M_BindVariable_int(&mut state.m_config, "mouseb_use", &mut state.m_controls.mousebuse);
    M_BindVariable_int(&mut state.m_config, "mouseb_backward", &mut state.m_controls.mousebbackward);
    M_BindVariable_int(&mut state.m_config, "dclick_use", &mut state.m_controls.dclick_use);
    M_BindVariable_int(&mut state.m_config, "key_pause", &mut state.m_controls.key_pause);
    M_BindVariable_int(&mut state.m_config, "key_message_refresh", &mut state.m_controls.key_message_refresh);
}
pub fn M_BindHereticControls(state: &mut GameState) {
    M_BindVariable_int(&mut state.m_config, "key_flyup", &mut state.m_controls.key_flyup);
    M_BindVariable_int(&mut state.m_config, "key_flydown", &mut state.m_controls.key_flydown);
    M_BindVariable_int(&mut state.m_config, "key_flycenter", &mut state.m_controls.key_flycenter);
    M_BindVariable_int(&mut state.m_config, "key_lookup", &mut state.m_controls.key_lookup);
    M_BindVariable_int(&mut state.m_config, "key_lookdown", &mut state.m_controls.key_lookdown);
    M_BindVariable_int(&mut state.m_config, "key_lookcenter", &mut state.m_controls.key_lookcenter);
    M_BindVariable_int(&mut state.m_config, "key_invleft", &mut state.m_controls.key_invleft);
    M_BindVariable_int(&mut state.m_config, "key_invright", &mut state.m_controls.key_invright);
    M_BindVariable_int(&mut state.m_config, "key_useartifact", &mut state.m_controls.key_useartifact);
}
pub fn M_BindHexenControls(state: &mut GameState) {
    M_BindVariable_int(&mut state.m_config, "key_jump", &mut state.m_controls.key_jump);
    M_BindVariable_int(&mut state.m_config, "mouseb_jump", &mut state.m_controls.mousebjump);
    M_BindVariable_int(&mut state.m_config, "joyb_jump", &mut state.m_controls.joybjump);
    M_BindVariable_int(&mut state.m_config, "key_arti_all", &mut state.m_controls.key_arti_all);
    M_BindVariable_int(&mut state.m_config, "key_arti_health", &mut state.m_controls.key_arti_health);
    M_BindVariable_int(&mut state.m_config, "key_arti_poisonbag", &mut state.m_controls.key_arti_poisonbag);
    M_BindVariable_int(
        &mut state.m_config,
        "key_arti_blastradius",
        &mut state.m_controls.key_arti_blastradius,
    );
    M_BindVariable_int(&mut state.m_config, "key_arti_teleport", &mut state.m_controls.key_arti_teleport);
    M_BindVariable_int(
        &mut state.m_config,
        "key_arti_teleportother",
        &mut state.m_controls.key_arti_teleportother,
    );
    M_BindVariable_int(&mut state.m_config, "key_arti_egg", &mut state.m_controls.key_arti_egg);
    M_BindVariable_int(
        &mut state.m_config,
        "key_arti_invulnerability",
        &mut state.m_controls.key_arti_invulnerability,
    );
}
pub fn M_BindStrifeControls(state: &mut GameState) {
    state.m_controls.key_message_refresh = '/' as i32;
    state.m_controls.key_jump = 'a' as i32;
    state.m_controls.key_lookup = KEY_PGUP;
    state.m_controls.key_lookdown = KEY_PGDN;
    state.m_controls.key_invleft = KEY_INS;
    state.m_controls.key_invright = KEY_DEL;
    M_BindVariable_int(&mut state.m_config, "key_jump", &mut state.m_controls.key_jump);
    M_BindVariable_int(&mut state.m_config, "key_lookUp", &mut state.m_controls.key_lookup);
    M_BindVariable_int(&mut state.m_config, "key_lookDown", &mut state.m_controls.key_lookdown);
    M_BindVariable_int(&mut state.m_config, "key_invLeft", &mut state.m_controls.key_invleft);
    M_BindVariable_int(&mut state.m_config, "key_invRight", &mut state.m_controls.key_invright);
    M_BindVariable_int(&mut state.m_config, "key_useHealth", &mut state.m_controls.key_usehealth);
    M_BindVariable_int(&mut state.m_config, "key_invquery", &mut state.m_controls.key_invquery);
    M_BindVariable_int(&mut state.m_config, "key_mission", &mut state.m_controls.key_mission);
    M_BindVariable_int(&mut state.m_config, "key_invPop", &mut state.m_controls.key_invpop);
    M_BindVariable_int(&mut state.m_config, "key_invKey", &mut state.m_controls.key_invkey);
    M_BindVariable_int(&mut state.m_config, "key_invHome", &mut state.m_controls.key_invhome);
    M_BindVariable_int(&mut state.m_config, "key_invEnd", &mut state.m_controls.key_invend);
    M_BindVariable_int(&mut state.m_config, "key_invUse", &mut state.m_controls.key_invuse);
    M_BindVariable_int(&mut state.m_config, "key_invDrop", &mut state.m_controls.key_invdrop);
    M_BindVariable_int(&mut state.m_config, "mouseb_jump", &mut state.m_controls.mousebjump);
    M_BindVariable_int(&mut state.m_config, "joyb_jump", &mut state.m_controls.joybjump);
}
pub fn M_BindWeaponControls(state: &mut GameState) {
    M_BindVariable_int(&mut state.m_config, "key_weapon1", &mut state.m_controls.key_weapon1);
    M_BindVariable_int(&mut state.m_config, "key_weapon2", &mut state.m_controls.key_weapon2);
    M_BindVariable_int(&mut state.m_config, "key_weapon3", &mut state.m_controls.key_weapon3);
    M_BindVariable_int(&mut state.m_config, "key_weapon4", &mut state.m_controls.key_weapon4);
    M_BindVariable_int(&mut state.m_config, "key_weapon5", &mut state.m_controls.key_weapon5);
    M_BindVariable_int(&mut state.m_config, "key_weapon6", &mut state.m_controls.key_weapon6);
    M_BindVariable_int(&mut state.m_config, "key_weapon7", &mut state.m_controls.key_weapon7);
    M_BindVariable_int(&mut state.m_config, "key_weapon8", &mut state.m_controls.key_weapon8);
    M_BindVariable_int(&mut state.m_config, "key_prevweapon", &mut state.m_controls.key_prevweapon);
    M_BindVariable_int(&mut state.m_config, "key_nextweapon", &mut state.m_controls.key_nextweapon);
    M_BindVariable_int(&mut state.m_config, "joyb_prevweapon", &mut state.m_controls.joybprevweapon);
    M_BindVariable_int(&mut state.m_config, "joyb_nextweapon", &mut state.m_controls.joybnextweapon);
    M_BindVariable_int(&mut state.m_config, "mouseb_prevweapon", &mut state.m_controls.mousebprevweapon);
    M_BindVariable_int(&mut state.m_config, "mouseb_nextweapon", &mut state.m_controls.mousebnextweapon);
}
pub fn M_BindMapControls(state: &mut GameState) {
    M_BindVariable_int(&mut state.m_config, "key_map_north", &mut state.m_controls.key_map_north);
    M_BindVariable_int(&mut state.m_config, "key_map_south", &mut state.m_controls.key_map_south);
    M_BindVariable_int(&mut state.m_config, "key_map_east", &mut state.m_controls.key_map_east);
    M_BindVariable_int(&mut state.m_config, "key_map_west", &mut state.m_controls.key_map_west);
    M_BindVariable_int(&mut state.m_config, "key_map_zoomin", &mut state.m_controls.key_map_zoomin);
    M_BindVariable_int(&mut state.m_config, "key_map_zoomout", &mut state.m_controls.key_map_zoomout);
    M_BindVariable_int(&mut state.m_config, "key_map_toggle", &mut state.m_controls.key_map_toggle);
    M_BindVariable_int(&mut state.m_config, "key_map_maxzoom", &mut state.m_controls.key_map_maxzoom);
    M_BindVariable_int(&mut state.m_config, "key_map_follow", &mut state.m_controls.key_map_follow);
    M_BindVariable_int(&mut state.m_config, "key_map_grid", &mut state.m_controls.key_map_grid);
    M_BindVariable_int(&mut state.m_config, "key_map_mark", &mut state.m_controls.key_map_mark);
    M_BindVariable_int(&mut state.m_config, "key_map_clearmark", &mut state.m_controls.key_map_clearmark);
}
pub fn M_BindMenuControls(state: &mut GameState) {
    M_BindVariable_int(&mut state.m_config, "key_menu_activate", &mut state.m_controls.key_menu_activate);
    M_BindVariable_int(&mut state.m_config, "key_menu_up", &mut state.m_controls.key_menu_up);
    M_BindVariable_int(&mut state.m_config, "key_menu_down", &mut state.m_controls.key_menu_down);
    M_BindVariable_int(&mut state.m_config, "key_menu_left", &mut state.m_controls.key_menu_left);
    M_BindVariable_int(&mut state.m_config, "key_menu_right", &mut state.m_controls.key_menu_right);
    M_BindVariable_int(&mut state.m_config, "key_menu_back", &mut state.m_controls.key_menu_back);
    M_BindVariable_int(&mut state.m_config, "key_menu_forward", &mut state.m_controls.key_menu_forward);
    M_BindVariable_int(&mut state.m_config, "key_menu_confirm", &mut state.m_controls.key_menu_confirm);
    M_BindVariable_int(&mut state.m_config, "key_menu_abort", &mut state.m_controls.key_menu_abort);
    M_BindVariable_int(&mut state.m_config, "key_menu_help", &mut state.m_controls.key_menu_help);
    M_BindVariable_int(&mut state.m_config, "key_menu_save", &mut state.m_controls.key_menu_save);
    M_BindVariable_int(&mut state.m_config, "key_menu_load", &mut state.m_controls.key_menu_load);
    M_BindVariable_int(&mut state.m_config, "key_menu_volume", &mut state.m_controls.key_menu_volume);
    M_BindVariable_int(&mut state.m_config, "key_menu_detail", &mut state.m_controls.key_menu_detail);
    M_BindVariable_int(&mut state.m_config, "key_menu_qsave", &mut state.m_controls.key_menu_qsave);
    M_BindVariable_int(&mut state.m_config, "key_menu_endgame", &mut state.m_controls.key_menu_endgame);
    M_BindVariable_int(&mut state.m_config, "key_menu_messages", &mut state.m_controls.key_menu_messages);
    M_BindVariable_int(&mut state.m_config, "key_menu_qload", &mut state.m_controls.key_menu_qload);
    M_BindVariable_int(&mut state.m_config, "key_menu_quit", &mut state.m_controls.key_menu_quit);
    M_BindVariable_int(&mut state.m_config, "key_menu_gamma", &mut state.m_controls.key_menu_gamma);
    M_BindVariable_int(&mut state.m_config, "key_menu_incscreen", &mut state.m_controls.key_menu_incscreen);
    M_BindVariable_int(&mut state.m_config, "key_menu_decscreen", &mut state.m_controls.key_menu_decscreen);
    M_BindVariable_int(&mut state.m_config, "key_menu_screenshot", &mut state.m_controls.key_menu_screenshot);
    M_BindVariable_int(&mut state.m_config, "key_demo_quit", &mut state.m_controls.key_demo_quit);
    M_BindVariable_int(&mut state.m_config, "key_spy", &mut state.m_controls.key_spy);
}
pub fn M_BindChatControls(state: &mut GameState, mut num_players: u32) {
    let mut i: u32 = 0;
    M_BindVariable_int(&mut state.m_config, "key_multi_msg", &mut state.m_controls.key_multi_msg);
    i = 0_u32;
    while i < num_players {
        let name = format!("key_multi_msgplayer{}", i.wrapping_add(1_u32));
        M_BindVariable_int(
            &mut state.m_config,
            &name,
            &mut state.m_controls.key_multi_msgplayer[i as usize],
        );
        i = i.wrapping_add(1);
    }
}
pub fn M_ApplyPlatformDefaults() {}
pub const KEY_CAPSLOCK: i32 = 0x80 + 0x3a_i32;
pub const KEY_SCRLCK: i32 = 0x80 + 0x46_i32;
