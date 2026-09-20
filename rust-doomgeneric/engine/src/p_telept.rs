use crate::d_mode::GameVersion;
use crate::game_state::GameState;
use crate::p_map::teleport_move;
use crate::p_mobj::MobjFlags;
use crate::p_mobj::MobjId;
use crate::p_tick::ThinkerPayload;

use crate::p_mobj::spawn_mobj;
use crate::p_mobj::MobjType;
use crate::p_mobj::ThinkerFn;

use crate::p_setup::LineId;

use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::tables::FINECOSINE;
use crate::tables::FINESINE;
pub fn teleport(state: &mut GameState, line: LineId, side: i32, thing: MobjId) -> bool {
    if state
        .world
        .p_mobj
        .mo(thing)
        .flags
        .contains(MobjFlags::MISSILE)
    {
        return false;
    }
    if side == 1 {
        return false;
    }
    let tag = i32::from(state.world.p_setup.line(line).tag);
    for i in 0..state.world.p_setup.numsectors {
        if i32::from(state.world.p_setup.sectors[i as usize].tag) != tag {
            continue;
        }
        let mut cursor = state.world.p_tick.head();
        while let Some(id) = cursor {
            if let ThinkerPayload::Mobj(m) = state.world.p_tick.payload(id) {
                let (is_live_mobj, m_type, m_subsector, m_x, m_y, m_angle) = {
                    let mo = state.world.p_mobj.mo(m);
                    (
                        matches!(mo.thinker.function, ThinkerFn::Mobj(_)),
                        mo.kind,
                        mo.subsector,
                        mo.x,
                        mo.y,
                        mo.angle,
                    )
                };
                if is_live_mobj && m_type as u32 == MobjType::Teleportman as i32 as u32 {
                    let sector = state.world.p_setup.subsectors[m_subsector.0 as usize].sector;
                    if sector.0 == i as u32 {
                        let (oldx, oldy, oldz) = {
                            let t = state.world.p_mobj.mo(thing);
                            (t.x, t.y, t.z)
                        };
                        if !teleport_move(state, thing, m_x, m_y) {
                            return false;
                        }
                        if state.game.doomstat.gameversion != GameVersion::Final {
                            let t = state.world.p_mobj.mo_mut(thing);
                            t.z = t.floorz;
                        }
                        if let Some(thing_player) = state.world.p_mobj.mo(thing).player {
                            let thing_z = state.world.p_mobj.mo(thing).z;
                            let p = state.game.g_game.player_mut(thing_player);
                            p.viewz = thing_z + p.viewheight;
                        }
                        let fog = spawn_mobj(state, oldx, oldy, oldz, MobjType::Tfog);
                        s_start_sound(state, SoundOrigin::Mobj(fog), SfxName::Telept);
                        let an = m_angle.fine();
                        let thing_z = state.world.p_mobj.mo(thing).z;
                        let fog = spawn_mobj(
                            state,
                            m_x + 20 * FINECOSINE[an],
                            m_y + 20 * FINESINE[an],
                            thing_z,
                            MobjType::Tfog,
                        );
                        s_start_sound(state, SoundOrigin::Mobj(fog), SfxName::Telept);
                        let t = state.world.p_mobj.mo_mut(thing);
                        if t.player.is_some() {
                            t.reactiontime = 18;
                        }
                        t.angle = m_angle;
                        t.momz = 0;
                        t.momy = t.momz;
                        t.momx = t.momy;
                        return true;
                    }
                }
            }
            cursor = state.world.p_tick.next(id);
        }
    }
    false
}
