use crate::d_player::PlayerId;
use crate::doomdef::MAXPLAYERS;
use crate::game_state::GameState;
use crate::game_state::World;
use crate::p_ceilng::CeilingId;
use crate::p_doors::DoorId;
use crate::p_lights::{FireFlickerId, GlowId, LightFlashId, StrobeId};
use crate::p_mobj::respawn_specials;
use crate::p_mobj::PMobjState;
use crate::p_mobj::{MobjId, Thinker, ThinkerFn};
use crate::p_plats::PlatId;
use crate::p_spec::update_specials;
use crate::p_spec::FloorId;
use crate::p_user::player_think;
use alloc::vec::Vec;

// A handle into PTickState's own node table -- never constructed outside
// this module, only handed out by head()/next() and walked by callers.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ThinkerId(u32);

// Mirrors ThinkerFn's payload-carrying variants. Every add_thinker caller
// already knows its own concrete type and passes it explicitly -- this
// can't be inferred from the thinker's `.function` value instead, because
// every spawn site except spawn_mobj calls add_thinker *before* setting
// `.function` to the concrete variant (confirmed by reading every call
// site: p_ceilng.rs/p_doors.rs/p_floor.rs/p_spec.rs/p_lights.rs all add
// first, assign `.function` a line or two later; only p_mobj.rs's
// spawn_mobj assigns first). Since Z_Malloc doesn't zero memory, `.function`
// is genuinely uninitialized garbage at add-time for those 8 types --
// reading it to infer a discriminant would be undefined behavior, not just
// a wrong answer (confirmed the hard way: an earlier version of this patch
// tried exactly that and crashed on the very first Xvfb boot test with
// "entered unreachable code", because the uninitialized bytes happened to
// decode as ThinkerFn::Paused). This is the only place that can still tell
// the reaper which per-type owning arena a Removed node's payload needs to
// be released from (by the time a node reaches ThinkerFn::Removed,
// `.function` no longer reveals which concrete type it was either --
// remove_thinker overwrites it).
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

// A ThinkerNode's payload identity. Every one of the 9 thinker kinds now
// carries a generation-checked id into its own owning arena (mirroring
// DoorId/MobjId) instead of a type-erased raw pointer -- this replaces
// what used to be a single `*mut Thinker` shared by every kind
// (Mobj, VlDoor, Ceiling, ...).
#[derive(Copy, Clone)]
pub enum ThinkerPayload {
    Mobj(MobjId),
    Ceiling(CeilingId),
    Door(DoorId),
    Floor(FloorId),
    Plat(PlatId),
    FireFlicker(FireFlickerId),
    LightFlash(LightFlashId),
    Strobe(StrobeId),
    Glow(GlowId),
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

impl Default for PTickState {
    fn default() -> Self {
        Self::new()
    }
}

impl PTickState {
    pub const fn new() -> Self {
        Self {
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

    pub fn ceiling_payload(&self, id: ThinkerId) -> CeilingId {
        match self.payload(id) {
            ThinkerPayload::Ceiling(ceiling_id) => ceiling_id,
            _ => panic!("thinker node is not a ceiling"),
        }
    }

    pub fn door_payload(&self, id: ThinkerId) -> DoorId {
        match self.payload(id) {
            ThinkerPayload::Door(door_id) => door_id,
            _ => panic!("thinker node is not a door"),
        }
    }

    pub fn floor_payload(&self, id: ThinkerId) -> FloorId {
        match self.payload(id) {
            ThinkerPayload::Floor(floor_id) => floor_id,
            _ => panic!("thinker node is not a floor"),
        }
    }

    pub fn plat_payload(&self, id: ThinkerId) -> PlatId {
        match self.payload(id) {
            ThinkerPayload::Plat(plat_id) => plat_id,
            _ => panic!("thinker node is not a plat"),
        }
    }
}

// The thinker header embedded in a ThinkerNode's payload, resolved through
// its owning arena. Takes the whole GameState because the arena is a sibling
// field PTickState has no access to. (A retired-but-not-yet-freed mobj still
// resolves: the reaper has to see ThinkerFn::Removed.)
pub fn thinker_mut(world: &mut World, id: ThinkerId) -> &mut Thinker {
    match world.p_tick.payload(id) {
        ThinkerPayload::Mobj(mobj_id) => {
            &mut world
                .p_mobj
                .mobj_mut(mobj_id)
                .expect("ThinkerNode payload must reference a live mobj")
                .thinker
        }
        ThinkerPayload::Ceiling(ceiling_id) => {
            &mut world
                .p_ceilng
                .get_mut(ceiling_id)
                .expect("ThinkerNode payload must reference a live ceiling")
                .thinker
        }
        ThinkerPayload::Door(door_id) => {
            &mut world
                .p_doors
                .get_mut(door_id)
                .expect("ThinkerNode payload must reference a live door")
                .thinker
        }
        ThinkerPayload::Floor(floor_id) => {
            &mut world
                .p_spec
                .get_floor_mut(floor_id)
                .expect("ThinkerNode payload must reference a live floor")
                .thinker
        }
        ThinkerPayload::Plat(plat_id) => {
            &mut world
                .p_plats
                .get_mut(plat_id)
                .expect("ThinkerNode payload must reference a live plat")
                .thinker
        }
        ThinkerPayload::FireFlicker(fireflicker_id) => {
            &mut world
                .p_lights
                .get_fireflicker_mut(fireflicker_id)
                .expect("ThinkerNode payload must reference a live fireflicker")
                .thinker
        }
        ThinkerPayload::LightFlash(lightflash_id) => {
            &mut world
                .p_lights
                .get_lightflash_mut(lightflash_id)
                .expect("ThinkerNode payload must reference a live lightflash")
                .thinker
        }
        ThinkerPayload::Strobe(strobe_id) => {
            &mut world
                .p_lights
                .get_strobe_mut(strobe_id)
                .expect("ThinkerNode payload must reference a live strobe")
                .thinker
        }
        ThinkerPayload::Glow(glow_id) => {
            &mut world
                .p_lights
                .get_glow_mut(glow_id)
                .expect("ThinkerNode payload must reference a live glow")
                .thinker
        }
    }
}

pub fn thinker_function(world: &mut World, id: ThinkerId) -> ThinkerFn {
    thinker_mut(world, id).function
}

// Every mobj that is still an active Mobj thinker (not yet Removed), in
// thinker-list order.
pub fn mobj_thinker_ids(p_mobj: &PMobjState, p_tick: &PTickState) -> Vec<MobjId> {
    let mut out = Vec::new();
    let mut cursor = p_tick.head();
    while let Some(id) = cursor {
        if let ThinkerPayload::Mobj(mobj_id) = p_tick.payload(id) {
            if matches!(p_mobj.mo(mobj_id).thinker.function, ThinkerFn::Mobj(_)) {
                out.push(mobj_id);
            }
        }
        cursor = p_tick.next(id);
    }
    out
}

pub fn init_thinkers(p_tick: &mut PTickState) {
    p_tick.nodes.clear();
    p_tick.free_list.clear();
    p_tick.head = None;
    p_tick.tail = None;
}

pub fn add_thinker(
    p_tick: &mut PTickState,
    payload: ThinkerPayload,
    kind: ThinkerKind,
) -> ThinkerId {
    let id = if let Some(index) = p_tick.free_list.pop() {
        p_tick.nodes[index as usize] = ThinkerNode {
            prev: None,
            next: None,
            payload,
            kind,
        };
        ThinkerId(index)
    } else {
        let index = p_tick.nodes.len() as u32;
        p_tick.nodes.push(ThinkerNode {
            prev: None,
            next: None,
            payload,
            kind,
        });
        ThinkerId(index)
    };
    if let Some(tail_id) = p_tick.tail {
        p_tick.nodes[tail_id.0 as usize].next = Some(id);
        p_tick.nodes[id.0 as usize].prev = Some(tail_id);
    } else {
        p_tick.head = Some(id);
    }
    p_tick.tail = Some(id);
    id
}

pub fn remove_thinker(thinker: &mut Thinker) {
    thinker.function = ThinkerFn::Removed;
}

// Unlinks a node from the externalized list (used only when run_thinkers
// finds a ThinkerFn::Removed node to reap). Does not touch the payload
// memory itself -- callers deallocate that separately (each payload type's
// own arena now, no longer Z_Free).
fn unlink_thinker_node(p_tick: &mut PTickState, id: ThinkerId) {
    let prev = p_tick.nodes[id.0 as usize].prev;
    let next = p_tick.nodes[id.0 as usize].next;
    match prev {
        Some(p) => p_tick.nodes[p.0 as usize].next = next,
        None => p_tick.head = next,
    }
    match next {
        Some(n) => p_tick.nodes[n.0 as usize].prev = prev,
        None => p_tick.tail = prev,
    }
    p_tick.free_list.push(id.0);
}

pub fn run_thinkers(state: &mut GameState) {
    let mut cursor = state.world.p_tick.head();
    while let Some(id) = cursor {
        let next;
        match thinker_function(&mut state.world, id) {
            ThinkerFn::Removed => {
                // Capture next before unlinking/freeing -- unlike the
                // pointer-chasing version this replaces, `next` lives in our
                // own node table, not inside the freed payload, so there's
                // no use-after-free hazard either way, but this ordering
                // matches the original semantics most directly.
                next = state.world.p_tick.next(id);
                let kind = state.world.p_tick.kind(id);
                unlink_thinker_node(&mut state.world.p_tick, id);
                // Every kind's memory is now owned by its own arena (Mobj
                // by PMobjState, VlDoor by PDoorsState, Ceiling by
                // PCeilngState, Plat by PPlatsState, FloorMove by
                // PSpecState, and the 4 light-effect types by PLightsState),
                // not the zone allocator -- dealloc/deallocate drops the
                // owning Box instead of Z_Free. Each arm reads its id out of
                // the node's payload (not out of currentthinker) since Raw
                // no longer exists -- every kind is id-based now.
                match kind {
                    ThinkerKind::Mobj => {
                        if let ThinkerPayload::Mobj(mobj_id) = state.world.p_tick.payload(id) {
                            state.world.p_mobj.deallocate(mobj_id);
                        }
                    }
                    ThinkerKind::Door => {
                        if let ThinkerPayload::Door(door_id) = state.world.p_tick.payload(id) {
                            state.world.p_doors.dealloc(door_id);
                        }
                    }
                    ThinkerKind::Ceiling => {
                        if let ThinkerPayload::Ceiling(ceiling_id) = state.world.p_tick.payload(id)
                        {
                            state.world.p_ceilng.dealloc(ceiling_id);
                        }
                    }
                    ThinkerKind::Plat => {
                        if let ThinkerPayload::Plat(plat_id) = state.world.p_tick.payload(id) {
                            state.world.p_plats.dealloc(plat_id);
                        }
                    }
                    ThinkerKind::Floor => {
                        if let ThinkerPayload::Floor(floor_id) = state.world.p_tick.payload(id) {
                            state.world.p_spec.dealloc_floor(floor_id);
                        }
                    }
                    ThinkerKind::FireFlicker => {
                        if let ThinkerPayload::FireFlicker(fireflicker_id) =
                            state.world.p_tick.payload(id)
                        {
                            state.world.p_lights.dealloc_fireflicker(fireflicker_id);
                        }
                    }
                    ThinkerKind::LightFlash => {
                        if let ThinkerPayload::LightFlash(lightflash_id) =
                            state.world.p_tick.payload(id)
                        {
                            state.world.p_lights.dealloc_lightflash(lightflash_id);
                        }
                    }
                    ThinkerKind::Strobe => {
                        if let ThinkerPayload::Strobe(strobe_id) = state.world.p_tick.payload(id) {
                            state.world.p_lights.dealloc_strobe(strobe_id);
                        }
                    }
                    ThinkerKind::Glow => {
                        if let ThinkerPayload::Glow(glow_id) = state.world.p_tick.payload(id) {
                            state.world.p_lights.dealloc_glow(glow_id);
                        }
                    }
                }
            }
            ThinkerFn::Paused | ThinkerFn::Unresolved => {
                next = state.world.p_tick.next(id);
            }
            ThinkerFn::Mobj(f) => {
                if let ThinkerPayload::Mobj(mobj_id) = state.world.p_tick.payload(id) {
                    f(state, mobj_id);
                }
                // Read after the call, not before: a think function can spawn
                // a new mobj (add_thinker appends at the tail), and if this
                // node was previously the tail, that newly spawned thinker
                // becomes reachable via .next immediately -- preserving
                // vanilla's same-tick-think-on-spawn behavior.
                next = state.world.p_tick.next(id);
            }
            ThinkerFn::Ceiling(f) => {
                if let ThinkerPayload::Ceiling(ceiling_id) = state.world.p_tick.payload(id) {
                    f(state, ceiling_id);
                }
                next = state.world.p_tick.next(id);
            }
            ThinkerFn::Door(f) => {
                if let ThinkerPayload::Door(door_id) = state.world.p_tick.payload(id) {
                    f(state, door_id);
                }
                next = state.world.p_tick.next(id);
            }
            ThinkerFn::Floor(f) => {
                if let ThinkerPayload::Floor(floor_id) = state.world.p_tick.payload(id) {
                    f(state, floor_id);
                }
                next = state.world.p_tick.next(id);
            }
            ThinkerFn::Plat(f) => {
                if let ThinkerPayload::Plat(plat_id) = state.world.p_tick.payload(id) {
                    f(state, plat_id);
                }
                next = state.world.p_tick.next(id);
            }
            ThinkerFn::FireFlicker(f) => {
                if let ThinkerPayload::FireFlicker(fireflicker_id) = state.world.p_tick.payload(id)
                {
                    f(state, fireflicker_id);
                }
                next = state.world.p_tick.next(id);
            }
            ThinkerFn::LightFlash(f) => {
                if let ThinkerPayload::LightFlash(lightflash_id) = state.world.p_tick.payload(id) {
                    f(state, lightflash_id);
                }
                next = state.world.p_tick.next(id);
            }
            ThinkerFn::Strobe(f) => {
                if let ThinkerPayload::Strobe(strobe_id) = state.world.p_tick.payload(id) {
                    f(state, strobe_id);
                }
                next = state.world.p_tick.next(id);
            }
            ThinkerFn::Glow(f) => {
                if let ThinkerPayload::Glow(glow_id) = state.world.p_tick.payload(id) {
                    f(state, glow_id);
                }
                next = state.world.p_tick.next(id);
            }
        }
        cursor = next;
    }
}
pub fn p_ticker(state: &mut GameState) {
    if state.game.g_game.paused {
        return;
    }
    if !state.game.g_game.netgame
        && state.ui.m_menu.menuactive
        && !state.game.g_game.demoplayback
        && state.game.g_game.players[state.game.g_game.consoleplayer].viewz != 1
    {
        return;
    }
    for i in 0..MAXPLAYERS {
        if state.game.g_game.playeringame[i] {
            player_think(state, PlayerId(i as u8));
        }
    }
    run_thinkers(state);
    update_specials(state);
    respawn_specials(state);
    state.world.p_tick.leveltime += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doomdef::Pixel;
    use crate::filesystem::MemFileSystem;
    use crate::game_state::init_game_state;
    use crate::p_doors::VlDoor;
    use crate::p_lights::{FireFlicker, Glow};
    use crate::p_spec::{Ceiling, FloorMove, Plat};
    use crate::platform::DoomPlatform;
    use alloc::boxed::Box;

    struct NullPlatform;
    impl DoomPlatform for NullPlatform {
        fn init(&mut self, _resx: i32, _resy: i32) {}
        fn draw_frame(&mut self, _frame: &[Pixel]) {}
        fn sleep_ms(&mut self, _ms: u32) {}
        fn get_ticks_ms(&mut self) -> u32 {
            0
        }
        fn get_key(&mut self) -> Option<(bool, u8)> {
            None
        }
        fn set_window_title(&mut self, _title: &str) {}
        fn print(&mut self, _message: &str) {}
        fn eprint(&mut self, _message: &str) {}
        fn quit(&mut self) -> ! {
            panic!("quit")
        }
    }

    // Exercises exactly what the DoorId conversion changed: a ThinkerNode's
    // payload round-trips through P_ThinkerRaw back to the arena's live
    // pointer, and the reaper's Door branch deallocs via the id (not a
    // stored raw pointer) -- including that a stale id stays stale even
    // after its arena slot is reused by a later spawn (generation check).
    #[test]
    fn door_thinker_lifecycle_via_id() {
        let state = init_game_state(Box::new(NullPlatform), Box::new(MemFileSystem::default()));

        let door_id = state.world.p_doors.spawn(VlDoor::default());
        let node_id = add_thinker(
            &mut state.world.p_tick,
            ThinkerPayload::Door(door_id),
            ThinkerKind::Door,
        );

        remove_thinker(thinker_mut(&mut state.world, node_id));
        run_thinkers(state);
        assert!(
            state.world.p_doors.get_ref(door_id).is_none(),
            "reaper should have deallocated the door via its DoorId"
        );

        // Reuse: a fresh spawn may land on the same freed slot, but the old
        // id must not resolve to the new door's memory.
        let door_id2 = state.world.p_doors.spawn(VlDoor::default());
        assert!(state.world.p_doors.get_ref(door_id).is_none());
        assert!(state.world.p_doors.get_ref(door_id2).is_some());
    }

    // Same lifecycle as door_thinker_lifecycle_via_id, but for the Mobj
    // kind -- exercises PMobjState::spawn/mobj_get/deallocate threaded
    // through ThinkerPayload::Mobj instead of a bare Mobj pointer.
    #[test]
    fn mobj_thinker_lifecycle_via_id() {
        let state = init_game_state(Box::new(NullPlatform), Box::new(MemFileSystem::default()));

        let value = state.world.p_mobj.dummy_mobj;
        let mobj_id = state.world.p_mobj.spawn(value);
        let node_id = add_thinker(
            &mut state.world.p_tick,
            ThinkerPayload::Mobj(mobj_id),
            ThinkerKind::Mobj,
        );

        remove_thinker(thinker_mut(&mut state.world, node_id));
        run_thinkers(state);
        assert!(
            state.world.p_mobj.mobj_ref(mobj_id).is_none(),
            "reaper should have deallocated the mobj via its MobjId"
        );

        let value2 = state.world.p_mobj.dummy_mobj;
        let mobj_id2 = state.world.p_mobj.spawn(value2);
        assert!(state.world.p_mobj.mobj_ref(mobj_id).is_none());
        assert!(state.world.p_mobj.mobj_ref(mobj_id2).is_some());
    }

    // Same lifecycle, Ceiling kind -- exercises PCeilngState's new
    // generation-checked arena (spawn/get/dealloc) instead of its old
    // Vec<Box<Ceiling>> + pointer-equality dealloc.
    #[test]
    fn ceiling_thinker_lifecycle_via_id() {
        let state = init_game_state(Box::new(NullPlatform), Box::new(MemFileSystem::default()));

        let ceiling_id = state.world.p_ceilng.spawn(Ceiling::default());
        let node_id = add_thinker(
            &mut state.world.p_tick,
            ThinkerPayload::Ceiling(ceiling_id),
            ThinkerKind::Ceiling,
        );

        remove_thinker(thinker_mut(&mut state.world, node_id));
        run_thinkers(state);
        assert!(
            state.world.p_ceilng.get_ref(ceiling_id).is_none(),
            "reaper should have deallocated the ceiling via its CeilingId"
        );

        let ceiling_id2 = state.world.p_ceilng.spawn(Ceiling::default());
        assert!(state.world.p_ceilng.get_ref(ceiling_id).is_none());
        assert!(state.world.p_ceilng.get_ref(ceiling_id2).is_some());
    }

    // Same lifecycle, Floor kind -- exercises PSpecState's new
    // generation-checked floor arena (spawn_floor/get_floor/dealloc_floor).
    #[test]
    fn floor_thinker_lifecycle_via_id() {
        let state = init_game_state(Box::new(NullPlatform), Box::new(MemFileSystem::default()));

        let floor_id = state.world.p_spec.spawn_floor(FloorMove::default());
        let node_id = add_thinker(
            &mut state.world.p_tick,
            ThinkerPayload::Floor(floor_id),
            ThinkerKind::Floor,
        );

        remove_thinker(thinker_mut(&mut state.world, node_id));
        run_thinkers(state);
        assert!(
            state.world.p_spec.get_floor_ref(floor_id).is_none(),
            "reaper should have deallocated the floor via its FloorId"
        );

        let floor_id2 = state.world.p_spec.spawn_floor(FloorMove::default());
        assert!(state.world.p_spec.get_floor_ref(floor_id).is_none());
        assert!(state.world.p_spec.get_floor_ref(floor_id2).is_some());
    }

    // Same lifecycle, Plat kind -- exercises PPlatsState's new
    // generation-checked arena (spawn/get/dealloc).
    #[test]
    fn plat_thinker_lifecycle_via_id() {
        let state = init_game_state(Box::new(NullPlatform), Box::new(MemFileSystem::default()));

        let plat_id = state.world.p_plats.spawn(Plat::default());
        let node_id = add_thinker(
            &mut state.world.p_tick,
            ThinkerPayload::Plat(plat_id),
            ThinkerKind::Plat,
        );

        remove_thinker(thinker_mut(&mut state.world, node_id));
        run_thinkers(state);
        assert!(
            state.world.p_plats.get_ref(plat_id).is_none(),
            "reaper should have deallocated the plat via its PlatId"
        );

        let plat_id2 = state.world.p_plats.spawn(Plat::default());
        assert!(state.world.p_plats.get_ref(plat_id).is_none());
        assert!(state.world.p_plats.get_ref(plat_id2).is_some());
    }

    // The 4 light-effect kinds (FireFlicker/LightFlash/Strobe/Glow) share
    // an identical arena shape inside PLightsState -- these two tests
    // (FireFlicker and Glow) are representative of all 4; LightFlash and
    // Strobe follow the exact same pattern.
    #[test]
    fn fireflicker_thinker_lifecycle_via_id() {
        let state = init_game_state(Box::new(NullPlatform), Box::new(MemFileSystem::default()));

        let fireflicker_id = state
            .world
            .p_lights
            .spawn_fireflicker(FireFlicker::default());
        let node_id = add_thinker(
            &mut state.world.p_tick,
            ThinkerPayload::FireFlicker(fireflicker_id),
            ThinkerKind::FireFlicker,
        );

        remove_thinker(thinker_mut(&mut state.world, node_id));
        run_thinkers(state);
        assert!(
            state
                .world
                .p_lights
                .get_fireflicker_ref(fireflicker_id)
                .is_none(),
            "reaper should have deallocated the fireflicker via its FireFlickerId"
        );

        let fireflicker_id2 = state
            .world
            .p_lights
            .spawn_fireflicker(FireFlicker::default());
        assert!(state
            .world
            .p_lights
            .get_fireflicker_ref(fireflicker_id)
            .is_none());
        assert!(state
            .world
            .p_lights
            .get_fireflicker_ref(fireflicker_id2)
            .is_some());
    }

    #[test]
    fn glow_thinker_lifecycle_via_id() {
        let state = init_game_state(Box::new(NullPlatform), Box::new(MemFileSystem::default()));

        let glow_id = state.world.p_lights.spawn_glow(Glow::default());
        let node_id = add_thinker(
            &mut state.world.p_tick,
            ThinkerPayload::Glow(glow_id),
            ThinkerKind::Glow,
        );

        remove_thinker(thinker_mut(&mut state.world, node_id));
        run_thinkers(state);
        assert!(
            state.world.p_lights.get_glow_ref(glow_id).is_none(),
            "reaper should have deallocated the glow via its GlowId"
        );

        let glow_id2 = state.world.p_lights.spawn_glow(Glow::default());
        assert!(state.world.p_lights.get_glow_ref(glow_id).is_none());
        assert!(state.world.p_lights.get_glow_ref(glow_id2).is_some());
    }
}
