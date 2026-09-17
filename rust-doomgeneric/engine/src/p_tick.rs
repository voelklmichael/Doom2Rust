use crate::d_player::player_t;
use crate::doomdef::MAXPLAYERS;
use crate::game_state::GameState;
use crate::p_doors::{vldoor_t, DoorId};
use crate::p_lights::{fireflicker_t, glow_t, lightflash_t, strobe_t};
use crate::p_mobj::P_RespawnSpecials;
use crate::p_mobj::{mobj_t, thinker_s, thinker_t, ThinkerFn};
use crate::p_spec::P_UpdateSpecials;
use crate::p_spec::{ceiling_t, floormove_t, plat_t};
use crate::p_user::P_PlayerThink;

// A handle into PTickState's own node table -- never constructed outside
// this module, only handed out by head()/next() and walked by callers.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ThinkerId(u32);

// Mirrors ThinkerFn's payload-carrying variants. Every P_AddThinker caller
// already knows its own concrete type and passes it explicitly -- this
// can't be inferred from the thinker's `.function` value instead, because
// every spawn site except P_SpawnMobj calls P_AddThinker *before* setting
// `.function` to the concrete variant (confirmed by reading every call
// site: p_ceilng.rs/p_doors.rs/p_floor.rs/p_spec.rs/p_lights.rs all add
// first, assign `.function` a line or two later; only p_mobj.rs's
// P_SpawnMobj assigns first). Since Z_Malloc doesn't zero memory, `.function`
// is genuinely uninitialized garbage at add-time for those 8 types --
// reading it to infer a discriminant would be undefined behavior, not just
// a wrong answer (confirmed the hard way: an earlier version of this patch
// tried exactly that and crashed on the very first Xvfb boot test with
// "entered unreachable code", because the uninitialized bytes happened to
// decode as ThinkerFn::Paused). This is the only place that can still tell
// the reaper which per-type owning arena a Removed node's payload needs to
// be released from (by the time a node reaches ThinkerFn::Removed,
// `.function` no longer reveals which concrete type it was either --
// P_RemoveThinker overwrites it).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ThinkerKind {
    Mobj,
    Ceiling,
    Door,
    Floor,
    Plat,
    FireFlicker,
    LightFlash,
    Strobe,
    Glow,
}

// A ThinkerNode's payload identity. Growing this enum with a generation-
// checked id (mirroring DoorId/MobjId) and converting one more kind's
// arena to hand out ids instead of bare pointers is the whole shape of
// this track -- Raw is the not-yet-converted fallback, still exactly the
// type-erased pointer this replaces (mobj_t, vldoor_t, ceiling_t, ...).
#[derive(Copy, Clone)]
pub enum ThinkerPayload {
    Door(DoorId),
    Raw(*mut thinker_s),
}

#[derive(Copy, Clone)]
struct ThinkerNode {
    prev: Option<ThinkerId>,
    next: Option<ThinkerId>,
    payload: ThinkerPayload,
    kind: ThinkerKind,
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

    pub fn payload(&self, id: ThinkerId) -> ThinkerPayload {
        self.nodes[id.0 as usize].payload
    }

    pub fn kind(&self, id: ThinkerId) -> ThinkerKind {
        self.nodes[id.0 as usize].kind
    }
}

// Resolves a ThinkerNode's payload back into the raw pointer every T_*
// function and every existing `state.p_tick.raw(id)` call site still
// expects. Takes the whole GameState (not just &PTickState) because
// resolving a converted kind's id needs its owning arena (e.g. p_doors)
// -- a sibling field PTickState itself has no access to.
pub fn P_ThinkerRaw(state: &GameState, id: ThinkerId) -> *mut thinker_t {
    match state.p_tick.payload(id) {
        ThinkerPayload::Door(door_id) => state
            .p_doors
            .get(door_id)
            .expect("ThinkerNode payload must reference a live door")
            as *mut thinker_t,
        ThinkerPayload::Raw(ptr) => ptr,
    }
}

pub fn P_InitThinkers(state: &mut GameState) {
    state.p_tick.nodes.clear();
    state.p_tick.free_list.clear();
    state.p_tick.head = None;
    state.p_tick.tail = None;
}

pub fn P_AddThinker(
    state: &mut GameState,
    payload: ThinkerPayload,
    kind: ThinkerKind,
) -> ThinkerId {
    let id = if let Some(index) = state.p_tick.free_list.pop() {
        state.p_tick.nodes[index as usize] = ThinkerNode {
            prev: None,
            next: None,
            payload,
            kind,
        };
        ThinkerId(index)
    } else {
        let index = state.p_tick.nodes.len() as u32;
        state.p_tick.nodes.push(ThinkerNode {
            prev: None,
            next: None,
            payload,
            kind,
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
    id
}

pub unsafe fn P_RemoveThinker(mut thinker: *mut thinker_t) {
    (*thinker).function = ThinkerFn::Removed;
}

// Unlinks a node from the externalized list (used only when P_RunThinkers
// finds a ThinkerFn::Removed node to reap). Does not touch the payload
// memory itself -- callers deallocate that separately (each payload type's
// own arena now, no longer Z_Free).
fn P_UnlinkThinkerNode(state: &mut GameState, id: ThinkerId) {
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
        let currentthinker = P_ThinkerRaw(state, id);
        let next;
        match (*currentthinker).function {
            ThinkerFn::Removed => {
                // Capture next before unlinking/freeing -- unlike the
                // pointer-chasing version this replaces, `next` lives in our
                // own node table, not inside the freed payload, so there's
                // no use-after-free hazard either way, but this ordering
                // matches the original semantics most directly.
                next = state.p_tick.next(id);
                let kind = state.p_tick.kind(id);
                P_UnlinkThinkerNode(state, id);
                match kind {
                    // mobj_t's memory is owned by PMobjState's arena now,
                    // not the zone allocator -- deallocate() drops the
                    // owning Box instead of Z_Free.
                    ThinkerKind::Mobj => {
                        let mobj_id = (*(currentthinker as *mut mobj_t)).id;
                        state.p_mobj.deallocate(mobj_id);
                    }
                    // vldoor_t's memory is owned by PDoorsState's arena now,
                    // keyed by the DoorId this node's payload carries (not
                    // by currentthinker -- Door is the one kind that's no
                    // longer a bare pointer here).
                    ThinkerKind::Door => {
                        if let ThinkerPayload::Door(door_id) = state.p_tick.payload(id) {
                            state.p_doors.dealloc(door_id);
                        }
                    }
                    // ceiling_t's memory is owned by PCeilngState's arena now.
                    ThinkerKind::Ceiling => {
                        state.p_ceilng.dealloc(currentthinker as *mut ceiling_t);
                    }
                    // plat_t's memory is owned by PPlatsState's arena now.
                    ThinkerKind::Plat => {
                        state.p_plats.dealloc(currentthinker as *mut plat_t);
                    }
                    // floormove_t's memory is owned by PSpecState's arena now.
                    ThinkerKind::Floor => {
                        state
                            .p_spec
                            .dealloc_floor(currentthinker as *mut floormove_t);
                    }
                    // All 4 remaining kinds' memory is owned by PLightsState's
                    // arenas now -- this was the last phase of the track.
                    ThinkerKind::FireFlicker => {
                        state
                            .p_lights
                            .dealloc_fireflicker(currentthinker as *mut fireflicker_t);
                    }
                    ThinkerKind::LightFlash => {
                        state
                            .p_lights
                            .dealloc_lightflash(currentthinker as *mut lightflash_t);
                    }
                    ThinkerKind::Strobe => {
                        state
                            .p_lights
                            .dealloc_strobe(currentthinker as *mut strobe_t);
                    }
                    ThinkerKind::Glow => {
                        state.p_lights.dealloc_glow(currentthinker as *mut glow_t);
                    }
                }
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
        && state.g_game.players[state.g_game.consoleplayer as usize].viewz != 1_i32
    {
        return;
    }
    i = 0_i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            let player: *mut player_t = &mut state.g_game.players[i as usize];
            P_PlayerThink(state, player);
        }
        i += 1;
    }
    P_RunThinkers(state);
    P_UpdateSpecials(state);
    P_RespawnSpecials(state);
    state.p_tick.leveltime += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doomdef::pixel_t;
    use crate::game_state::init_game_state;
    use crate::platform::DoomPlatform;

    struct NullPlatform;
    impl DoomPlatform for NullPlatform {
        fn init(&mut self, _screen_buffer: *mut pixel_t, _resx: i32, _resy: i32) {}
        fn draw_frame(&mut self) {}
        fn sleep_ms(&mut self, _ms: u32) {}
        fn get_ticks_ms(&mut self) -> u32 {
            0
        }
        fn get_key(&mut self) -> Option<(bool, u8)> {
            None
        }
        fn set_window_title(&mut self, _title: &str) {}
    }

    // Exercises exactly what the DoorId conversion changed: a ThinkerNode's
    // payload round-trips through P_ThinkerRaw back to the arena's live
    // pointer, and the reaper's Door branch deallocs via the id (not a
    // stored raw pointer) -- including that a stale id stays stale even
    // after its arena slot is reused by a later spawn (generation check).
    #[test]
    fn door_thinker_lifecycle_via_id() {
        let state = init_game_state(Box::new(NullPlatform));

        let (door_id, door_ptr) = state.p_doors.spawn(vldoor_t::default());
        let node_id = P_AddThinker(state, ThinkerPayload::Door(door_id), ThinkerKind::Door);
        assert_eq!(P_ThinkerRaw(state, node_id), door_ptr as *mut thinker_t);

        unsafe { P_RemoveThinker(P_ThinkerRaw(state, node_id)) };
        unsafe { P_RunThinkers(state) };
        assert!(
            state.p_doors.get(door_id).is_none(),
            "reaper should have deallocated the door via its DoorId"
        );

        // Reuse: a fresh spawn may land on the same freed slot, but the old
        // id must not resolve to the new door's memory.
        let (door_id2, door_ptr2) = state.p_doors.spawn(vldoor_t::default());
        assert!(state.p_doors.get(door_id).is_none());
        assert_eq!(state.p_doors.get(door_id2), Some(door_ptr2));
    }
}
