use crate::src::d_mode::GameVersion;
use crate::src::game_state::GameState;
use crate::src::m_fixed::fixed_t;
use crate::src::p_map::P_TeleportMove;
use crate::src::p_mobj::mobj_t;
use crate::src::p_mobj::P_SpawnMobj;
use crate::src::p_mobj::ThinkerFn;
use crate::src::p_mobj::MF_MISSILE;
use crate::src::p_mobj::{line_t, thinker_t};
use crate::src::p_mobj::{MT_TELEPORTMAN, MT_TFOG};
use crate::src::p_setup::SectorId;
use crate::src::s_sound::S_StartSound;
use crate::src::sounds::sfx_telept;
use crate::src::tables::finecosine;
use crate::src::tables::finesine;
use crate::src::tables::ANGLETOFINESHIFT;
pub unsafe fn EV_Teleport(
    state: &mut GameState,
    mut line: *mut line_t,
    mut side: i32,
    mut thing: *mut mobj_t,
) -> i32 {
    let mut i: i32 = 0;
    let mut tag: i32 = 0;
    let mut m: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut fog: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut an: u32 = 0;
    let mut thinker: *mut thinker_t = ::core::ptr::null_mut::<thinker_t>();
    let mut sector: SectorId = SectorId(0);
    let mut oldx: fixed_t = 0;
    let mut oldy: fixed_t = 0;
    let mut oldz: fixed_t = 0;
    if (*thing).flags & MF_MISSILE as i32 != 0 {
        return 0 as i32;
    }
    if side == 1 as i32 {
        return 0 as i32;
    }
    tag = (*line).tag as i32;
    i = 0 as i32;
    while i < state.p_setup.numsectors {
        if state.p_setup.sectors[i as usize].tag as i32 == tag {
            let mut cursor = state.p_tick.head();
            while let Some(id) = cursor {
                thinker = state.p_tick.raw(id);
                if matches!((*thinker).function, ThinkerFn::Mobj(_)) {
                    m = thinker as *mut mobj_t;
                    if !((*m).type_0 as u32 != MT_TELEPORTMAN as i32 as u32) {
                        sector = state.p_setup.subsectors[(*m).subsector.0 as usize].sector;
                        if !(sector.0 != i as u32) {
                            oldx = (*thing).x;
                            oldy = (*thing).y;
                            oldz = (*thing).z;
                            if !P_TeleportMove(state, thing, (*m).x, (*m).y) {
                                return 0 as i32;
                            }
                            if state.doomstat.gameversion != GameVersion::r#final {
                                (*thing).z = (*thing).floorz;
                            }
                            if let Some(thing_player) = (*thing).player {
                                let thing_player = state.g_game.player_mut(thing_player);
                                (*thing_player).viewz = (*thing).z + (*thing_player).viewheight;
                            }
                            fog = P_SpawnMobj(state, oldx, oldy, oldz, MT_TFOG);
                            S_StartSound(
                                state,
                                fog as *mut ::core::ffi::c_void,
                                sfx_telept as i32,
                            );
                            an = ((*m).angle >> ANGLETOFINESHIFT) as u32;
                            fog = P_SpawnMobj(
                                state,
                                (*m).x + 20 as fixed_t * finecosine[an as isize],
                                (*m).y + 20 as fixed_t * finesine[an as usize],
                                (*thing).z,
                                MT_TFOG,
                            );
                            S_StartSound(
                                state,
                                fog as *mut ::core::ffi::c_void,
                                sfx_telept as i32,
                            );
                            if (*thing).player.is_some() {
                                (*thing).reactiontime = 18 as i32;
                            }
                            (*thing).angle = (*m).angle;
                            (*thing).momz = 0 as i32 as fixed_t;
                            (*thing).momy = (*thing).momz;
                            (*thing).momx = (*thing).momy;
                            return 1 as i32;
                        }
                    }
                }
                cursor = state.p_tick.next(id);
            }
        }
        i += 1;
    }
    return 0 as i32;
}
