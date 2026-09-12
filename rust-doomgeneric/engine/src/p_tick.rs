use crate::src::d_player::player_t;
use crate::src::doomdef::MAXPLAYERS;
use crate::src::game_state::GameState;
use crate::src::p_doors::vldoor_t;
use crate::src::p_lights::{fireflicker_t, glow_t, lightflash_t, strobe_t};
use crate::src::p_mobj::P_RespawnSpecials;
use crate::src::p_mobj::{mobj_t, thinker_s, thinker_t, ThinkerFn};
use crate::src::p_spec::P_UpdateSpecials;
use crate::src::p_spec::{ceiling_t, floormove_t, plat_t};
use crate::src::p_user::P_PlayerThink;
use crate::src::z_zone::Z_Free;

// A handle into PTickState's own node table -- never constructed outside
// this module, only handed out by head()/next() and walked by callers.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ThinkerId(u32);

#[derive(Copy, Clone)]
struct ThinkerNode {
    prev: Option<ThinkerId>,
    next: Option<ThinkerId>,
    // Still points at the thinker's own Z_Malloc'd payload (mobj_t,
    // vldoor_t, ceiling_t, ...) -- this table only externalizes the
    // prev/next list bookkeeping, not the payload storage or the
    // base-struct-downcast dispatch in P_RunThinkers below.
    raw: *mut thinker_s,
}

pub struct PTickState {
    pub leveltime: i32,
    nodes: Vec<ThinkerNode>,
    free_list: Vec<u32>,
    head: Option<ThinkerId>,
    tail: Option<ThinkerId>,
}

impl PTickState {
    pub const fn new() -> Self {
        PTickState {
            leveltime: 0,
            nodes: Vec::new(),
            free_list: Vec::new(),
            head: None,
            tail: None,
        }
    }

    pub fn head(&self) -> Option<ThinkerId> {
        self.head
    }

    pub fn next(&self, id: ThinkerId) -> Option<ThinkerId> {
        self.nodes[id.0 as usize].next
    }

    pub fn raw(&self, id: ThinkerId) -> *mut thinker_t {
        self.nodes[id.0 as usize].raw
    }
}

pub unsafe fn P_InitThinkers(state: &mut GameState) {
    state.p_tick.nodes.clear();
    state.p_tick.free_list.clear();
    state.p_tick.head = None;
    state.p_tick.tail = None;
}

pub unsafe fn P_AddThinker(state: &mut GameState, mut thinker: *mut thinker_t) {
    let id = if let Some(index) = state.p_tick.free_list.pop() {
        state.p_tick.nodes[index as usize] = ThinkerNode {
            prev: None,
            next: None,
            raw: thinker,
        };
        ThinkerId(index)
    } else {
        let index = state.p_tick.nodes.len() as u32;
        state.p_tick.nodes.push(ThinkerNode {
            prev: None,
            next: None,
            raw: thinker,
        });
        ThinkerId(index)
    };
    if let Some(tail_id) = state.p_tick.tail {
        state.p_tick.nodes[tail_id.0 as usize].next = Some(id);
        state.p_tick.nodes[id.0 as usize].prev = Some(tail_id);
    } else {
        state.p_tick.head = Some(id);
    }
    state.p_tick.tail = Some(id);
}

pub unsafe fn P_RemoveThinker(mut thinker: *mut thinker_t) {
    (*thinker).function = ThinkerFn::Removed;
}

// Unlinks a node from the externalized list (used only when P_RunThinkers
// finds a ThinkerFn::Removed node to reap). Does not touch the payload
// memory itself -- callers Z_Free that separately.
unsafe fn P_UnlinkThinkerNode(state: &mut GameState, id: ThinkerId) {
    let prev = state.p_tick.nodes[id.0 as usize].prev;
    let next = state.p_tick.nodes[id.0 as usize].next;
    match prev {
        Some(p) => state.p_tick.nodes[p.0 as usize].next = next,
        None => state.p_tick.head = next,
    }
    match next {
        Some(n) => state.p_tick.nodes[n.0 as usize].prev = prev,
        None => state.p_tick.tail = prev,
    }
    state.p_tick.free_list.push(id.0);
}

pub unsafe fn P_RunThinkers(state: &mut GameState) {
    let mut cursor = state.p_tick.head();
    while let Some(id) = cursor {
        let currentthinker = state.p_tick.raw(id);
        let next;
        match (*currentthinker).function {
            ThinkerFn::Removed => {
                // Capture next before unlinking/freeing -- unlike the
                // pointer-chasing version this replaces, `next` lives in our
                // own node table, not inside the freed payload, so there's
                // no use-after-free hazard either way, but this ordering
                // matches the original semantics most directly.
                next = state.p_tick.next(id);
                P_UnlinkThinkerNode(state, id);
                Z_Free(
                    &mut state.z_zone,
                    currentthinker as *mut ::core::ffi::c_void,
                );
            }
            ThinkerFn::Paused | ThinkerFn::Unresolved => {
                next = state.p_tick.next(id);
            }
            ThinkerFn::Mobj(f) => {
                let mobj_id = (*(currentthinker as *mut mobj_t)).id;
                f(state, mobj_id);
                // Read after the call, not before: a think function can spawn
                // a new mobj (P_AddThinker appends at the tail), and if this
                // node was previously the tail, that newly spawned thinker
                // becomes reachable via .next immediately -- preserving
                // vanilla's same-tick-think-on-spawn behavior.
                next = state.p_tick.next(id);
            }
            ThinkerFn::Ceiling(f) => {
                f(state, currentthinker as *mut ceiling_t);
                next = state.p_tick.next(id);
            }
            ThinkerFn::Door(f) => {
                f(state, currentthinker as *mut vldoor_t);
                next = state.p_tick.next(id);
            }
            ThinkerFn::Floor(f) => {
                f(state, currentthinker as *mut floormove_t);
                next = state.p_tick.next(id);
            }
            ThinkerFn::Plat(f) => {
                f(state, currentthinker as *mut plat_t);
                next = state.p_tick.next(id);
            }
            ThinkerFn::FireFlicker(f) => {
                f(state, currentthinker as *mut fireflicker_t);
                next = state.p_tick.next(id);
            }
            ThinkerFn::LightFlash(f) => {
                f(state, currentthinker as *mut lightflash_t);
                next = state.p_tick.next(id);
            }
            ThinkerFn::Strobe(f) => {
                f(state, currentthinker as *mut strobe_t);
                next = state.p_tick.next(id);
            }
            ThinkerFn::Glow(f) => {
                f(state, currentthinker as *mut glow_t);
                next = state.p_tick.next(id);
            }
        }
        cursor = next;
    }
}
pub unsafe fn P_Ticker(state: &mut GameState) {
    let mut i: i32 = 0;
    if state.g_game.paused {
        return;
    }
    if !state.g_game.netgame
        && state.m_menu.menuactive
        && !state.g_game.demoplayback
        && state.g_game.players[state.g_game.consoleplayer as usize].viewz != 1 as i32
    {
        return;
    }
    i = 0 as i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] != 0 {
            let player = (&raw mut state.g_game.players as *mut player_t).offset(i as isize)
                as *mut player_t;
            P_PlayerThink(state, player);
        }
        i += 1;
    }
    P_RunThinkers(state);
    P_UpdateSpecials(state);
    P_RespawnSpecials(state);
    state.p_tick.leveltime += 1;
}
