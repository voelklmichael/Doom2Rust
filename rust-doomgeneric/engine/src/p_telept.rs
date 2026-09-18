use crate::d_mode::GameVersion;
use crate::game_state::GameState;
use crate::m_fixed::fixed_t;
use crate::p_map::P_TeleportMove;
use crate::p_mobj::MobjId;
use crate::p_tick::ThinkerPayload;

use crate::p_mobj::MobjType;
use crate::p_mobj::P_SpawnMobj;
use crate::p_mobj::ThinkerFn;
use crate::p_mobj::MF_MISSILE;
use crate::p_setup::LineId;

use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::tables::finecosine;
use crate::tables::finesine;
use crate::tables::ANGLETOFINESHIFT;
pub fn EV_Teleport(state: &mut GameState, line: LineId, side: i32, thing: MobjId) -> i32 {
    if state.p_mobj.mo(thing).flags & MF_MISSILE != 0 {
        return 0_i32;
    }
    if side == 1_i32 {
        return 0_i32;
    }
    let tag = state.p_setup.line(line).tag as i32;
    for i in 0..state.p_setup.numsectors {
        if state.p_setup.sectors[i as usize].tag as i32 != tag {
            continue;
        }
        let mut cursor = state.p_tick.head();
        while let Some(id) = cursor {
            if let ThinkerPayload::Mobj(m) = state.p_tick.payload(id) {
                let (is_live_mobj, m_type, m_subsector, m_x, m_y, m_angle) = {
                    let mo = state.p_mobj.mo(m);
                    (
                        matches!(mo.thinker.function, ThinkerFn::Mobj(_)),
                        mo.type_0,
                        mo.subsector,
                        mo.x,
                        mo.y,
                        mo.angle,
                    )
                };
                if is_live_mobj && m_type as u32 == MobjType::MT_TELEPORTMAN as i32 as u32 {
                    let sector = state.p_setup.subsectors[m_subsector.0 as usize].sector;
                    if sector.0 == i as u32 {
                        let (oldx, oldy, oldz) = {
                            let t = state.p_mobj.mo(thing);
                            (t.x, t.y, t.z)
                        };
                        if !P_TeleportMove(state, thing, m_x, m_y) {
                            return 0_i32;
                        }
                        if state.doomstat.gameversion != GameVersion::r#final {
                            let t = state.p_mobj.mo_mut(thing);
                            t.z = t.floorz;
                        }
                        if let Some(thing_player) = state.p_mobj.mo(thing).player {
                            let thing_z = state.p_mobj.mo(thing).z;
                            let p = state.g_game.player_mut(thing_player);
                            p.viewz = thing_z + p.viewheight;
                        }
                        let fog = P_SpawnMobj(state, oldx, oldy, oldz, MobjType::MT_TFOG);
                        S_StartSound(state, SoundOrigin::Mobj(fog), SfxName::sfx_telept as i32);
                        let an = m_angle >> ANGLETOFINESHIFT;
                        let thing_z = state.p_mobj.mo(thing).z;
                        let fog = P_SpawnMobj(
                            state,
                            m_x + 20 as fixed_t * finecosine[an as usize],
                            m_y + 20 as fixed_t * finesine[an as usize],
                            thing_z,
                            MobjType::MT_TFOG,
                        );
                        S_StartSound(state, SoundOrigin::Mobj(fog), SfxName::sfx_telept as i32);
                        let t = state.p_mobj.mo_mut(thing);
                        if t.player.is_some() {
                            t.reactiontime = 18_i32;
                        }
                        t.angle = m_angle;
                        t.momz = 0_i32 as fixed_t;
                        t.momy = t.momz;
                        t.momx = t.momy;
                        return 1_i32;
                    }
                }
            }
            cursor = state.p_tick.next(id);
        }
    }
    0_i32
}
