use crate::d_mode::GameVersion;
use crate::game_state::GameState;
use crate::p_mobj::MobjId;
use crate::m_fixed::fixed_t;
use crate::p_map::P_TeleportMove;
use crate::p_mobj::mobj_t;
use crate::p_mobj::thinker_t;
use crate::p_mobj::MobjType;
use crate::p_mobj::P_SpawnMobjPtr;
use crate::p_mobj::ThinkerFn;
use crate::p_mobj::MF_MISSILE;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_tick::P_ThinkerRaw;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::sfx_telept;
use crate::tables::finecosine;
use crate::tables::finesine;
use crate::tables::ANGLETOFINESHIFT;
pub unsafe fn EV_Teleport(
    state: &mut GameState,
    mut line: LineId,
    mut side: i32,
    thing: MobjId,
) -> i32 {
    let thing: *mut mobj_t = state.p_mobj.mobj_ptr(thing);
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
        return 0_i32;
    }
    if side == 1_i32 {
        return 0_i32;
    }
    tag = state.p_setup.line(line).tag as i32;
    i = 0_i32;
    while i < state.p_setup.numsectors {
        if state.p_setup.sectors[i as usize].tag as i32 == tag {
            let mut cursor = state.p_tick.head();
            while let Some(id) = cursor {
                thinker = P_ThinkerRaw(state, id);
                if matches!((*thinker).function, ThinkerFn::Mobj(_)) {
                    m = thinker as *mut mobj_t;
                    if (*m).type_0 as u32 == MobjType::MT_TELEPORTMAN as i32 as u32 {
                        sector = state.p_setup.subsectors[(*m).subsector.0 as usize].sector;
                        if sector.0 == i as u32 {
                            oldx = (*thing).x;
                            oldy = (*thing).y;
                            oldz = (*thing).z;
                            if !P_TeleportMove(state, (*thing).id, (*m).x, (*m).y) {
                                return 0_i32;
                            }
                            if state.doomstat.gameversion != GameVersion::r#final {
                                (*thing).z = (*thing).floorz;
                            }
                            if let Some(thing_player) = (*thing).player {
                                let thing_player = state.g_game.player_mut(thing_player);
                                (*thing_player).viewz = (*thing).z + (*thing_player).viewheight;
                            }
                            fog = P_SpawnMobjPtr(state, oldx, oldy, oldz, MobjType::MT_TFOG);
                            S_StartSound(state, SoundOrigin::Mobj((*(fog)).id), sfx_telept as i32);
                            an = (*m).angle >> ANGLETOFINESHIFT;
                            fog = P_SpawnMobjPtr(
                                state,
                                (*m).x + 20 as fixed_t * finecosine[an as isize],
                                (*m).y + 20 as fixed_t * finesine[an as usize],
                                (*thing).z,
                                MobjType::MT_TFOG,
                            );
                            S_StartSound(state, SoundOrigin::Mobj((*(fog)).id), sfx_telept as i32);
                            if (*thing).player.is_some() {
                                (*thing).reactiontime = 18_i32;
                            }
                            (*thing).angle = (*m).angle;
                            (*thing).momz = 0_i32 as fixed_t;
                            (*thing).momy = (*thing).momz;
                            (*thing).momx = (*thing).momy;
                            return 1_i32;
                        }
                    }
                }
                cursor = state.p_tick.next(id);
            }
        }
        i += 1;
    }
    0_i32
}
