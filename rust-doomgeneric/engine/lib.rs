#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(raw_ref_op)]

#[macro_use]
extern crate c2rust_bitfields;

pub mod src {
    pub mod am_map;
    pub mod d_event;
    pub mod d_items;
    pub mod d_iwad;
    pub mod d_loop;
    pub mod d_main;
    pub mod d_mode;
    pub mod d_net;
    pub mod d_player;
    pub mod d_ticcmd;
    pub mod doomdef;
    pub mod doomgeneric;
    pub mod doomstat;
    pub mod dstrings;
    pub mod dummy;
    pub mod f_finale;
    pub mod f_wipe;
    pub mod fixed_cstr;
    pub mod g_game;
    pub mod game_state;
    pub mod hu_lib;
    pub mod hu_stuff;
    pub mod i_cdmus;
    pub mod i_input;
    pub mod i_joystick;
    pub mod i_sound;
    pub mod i_system;
    pub mod i_timer;
    pub mod i_video;
    pub mod info;
    pub mod m_argv;
    pub mod m_bbox;
    pub mod m_cheat;
    pub mod m_config;
    pub mod m_controls;
    pub mod m_fixed;
    pub mod m_menu;
    pub mod m_misc;
    pub mod m_random;
    pub mod mem_compat;
    pub mod p_ceilng;
    pub mod p_doors;
    pub mod p_enemy;
    pub mod p_floor;
    pub mod p_inter;
    pub mod p_lights;
    pub mod p_map;
    pub mod p_maputl;
    pub mod p_mobj;
    pub mod p_plats;
    pub mod p_pspr;
    pub mod p_saveg;
    pub mod p_setup;
    pub mod p_sight;
    pub mod p_spec;
    pub mod p_switch;
    pub mod p_telept;
    pub mod p_tick;
    pub mod p_user;
    pub mod platform;
    pub mod r_bsp;
    pub mod r_data;
    pub mod r_defs;
    pub mod r_draw;
    pub mod r_main;
    pub mod r_plane;
    pub mod r_segs;
    pub mod r_sky;
    pub mod r_things;
    pub mod s_sound;
    pub mod sha1;
    pub mod sounds;
    pub mod st_lib;
    pub mod st_stuff;
    pub mod statdump;
    pub mod stdint_types;
    pub mod tables;
    pub mod v_video;
    pub mod w_checksum;
    pub mod w_file;
    pub mod w_file_stdc;
    pub mod w_main;
    pub mod w_wad;
    pub mod wi_stuff;
} // mod src
