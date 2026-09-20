use crate::m_fixed::{Fixed, FRACUNIT};
use crate::p_enemy::{
    a_fire, a_tracer, baby_metal, boss_death, brain_awake, brain_die, brain_explode, brain_pain,
    brain_scream, brain_spit, bruis_attack, bspi_attack, chase, close_shotgun2, cpos_attack,
    cpos_refire, cyber_attack, explode, face_target, fall, fat_attack1, fat_attack2, fat_attack3,
    fat_raise, fire_crackle, head_attack, hoof, keen_die, load_shotgun2, look, metal,
    open_shotgun2, pain, pain_attack, pain_die, player_scream, pos_attack, sarg_attack, scream,
    skel_fist, skel_missile, skel_whoosh, skull_attack, spawn_fly, spawn_sound, spid_refire,
    spos_attack, start_fire, troop_attack, vile_attack, vile_chase, vile_start, vile_target,
    xscream,
};
use crate::p_mobj::MobjFlags;
use crate::p_mobj::SpriteNum;
use crate::p_mobj::StateNum;
use crate::p_mobj::{MobjInfo, MobjType, State, StateAction};
use crate::p_pspr::{
    bfgsound, bfgspray, check_reload, fire_bfg, fire_cgun, fire_missile, fire_pistol, fire_plasma,
    fire_shotgun, fire_shotgun2, gun_flash, light0, light1, light2, lower, punch, raise, re_fire,
    saw, weapon_ready,
};
use crate::sounds::SfxName;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StateId(pub u32);

const fn state(
    sprite: SpriteNum,
    frame: i32,
    tics: i32,
    action: StateAction,
    nextstate: StateNum,
) -> State {
    State {
        sprite,
        frame,
        tics,
        action,
        nextstate,
        misc1: 0,
        misc2: 0,
    }
}

/// The state table, in `StateNum` order: sprite, frame, tics, action, next state.
#[rustfmt::skip]
static STATES: [State; 967] = [
    state(SpriteNum::Troo, 0, -1, StateAction::None, StateNum::Null), // Null
    state(SpriteNum::Shtg, 4, 0, StateAction::Weapon(light0), StateNum::Null), // Lightdone
    state(SpriteNum::Pung, 0, 1, StateAction::Weapon(weapon_ready), StateNum::Punch), // Punch
    state(SpriteNum::Pung, 0, 1, StateAction::Weapon(lower), StateNum::Punchdown), // Punchdown
    state(SpriteNum::Pung, 0, 1, StateAction::Weapon(raise), StateNum::Punchup), // Punchup
    state(SpriteNum::Pung, 1, 4, StateAction::None, StateNum::Punch2), // Punch1
    state(SpriteNum::Pung, 2, 4, StateAction::Weapon(punch), StateNum::Punch3), // Punch2
    state(SpriteNum::Pung, 3, 5, StateAction::None, StateNum::Punch4), // Punch3
    state(SpriteNum::Pung, 2, 4, StateAction::None, StateNum::Punch5), // Punch4
    state(SpriteNum::Pung, 1, 5, StateAction::Weapon(re_fire), StateNum::Punch), // Punch5
    state(SpriteNum::Pisg, 0, 1, StateAction::Weapon(weapon_ready), StateNum::Pistol), // Pistol
    state(SpriteNum::Pisg, 0, 1, StateAction::Weapon(lower), StateNum::Pistoldown), // Pistoldown
    state(SpriteNum::Pisg, 0, 1, StateAction::Weapon(raise), StateNum::Pistolup), // Pistolup
    state(SpriteNum::Pisg, 0, 4, StateAction::None, StateNum::Pistol2), // Pistol1
    state(SpriteNum::Pisg, 1, 6, StateAction::Weapon(fire_pistol), StateNum::Pistol3), // Pistol2
    state(SpriteNum::Pisg, 2, 4, StateAction::None, StateNum::Pistol4), // Pistol3
    state(SpriteNum::Pisg, 1, 5, StateAction::Weapon(re_fire), StateNum::Pistol), // Pistol4
    state(SpriteNum::Pisf, 32768, 7, StateAction::Weapon(light1), StateNum::Lightdone), // Pistolflash
    state(SpriteNum::Shtg, 0, 1, StateAction::Weapon(weapon_ready), StateNum::Sgun), // Sgun
    state(SpriteNum::Shtg, 0, 1, StateAction::Weapon(lower), StateNum::Sgundown), // Sgundown
    state(SpriteNum::Shtg, 0, 1, StateAction::Weapon(raise), StateNum::Sgunup), // Sgunup
    state(SpriteNum::Shtg, 0, 3, StateAction::None, StateNum::Sgun2), // Sgun1
    state(SpriteNum::Shtg, 0, 7, StateAction::Weapon(fire_shotgun), StateNum::Sgun3), // Sgun2
    state(SpriteNum::Shtg, 1, 5, StateAction::None, StateNum::Sgun4), // Sgun3
    state(SpriteNum::Shtg, 2, 5, StateAction::None, StateNum::Sgun5), // Sgun4
    state(SpriteNum::Shtg, 3, 4, StateAction::None, StateNum::Sgun6), // Sgun5
    state(SpriteNum::Shtg, 2, 5, StateAction::None, StateNum::Sgun7), // Sgun6
    state(SpriteNum::Shtg, 1, 5, StateAction::None, StateNum::Sgun8), // Sgun7
    state(SpriteNum::Shtg, 0, 3, StateAction::None, StateNum::Sgun9), // Sgun8
    state(SpriteNum::Shtg, 0, 7, StateAction::Weapon(re_fire), StateNum::Sgun), // Sgun9
    state(SpriteNum::Shtf, 32768, 4, StateAction::Weapon(light1), StateNum::Sgunflash2), // Sgunflash1
    state(SpriteNum::Shtf, 32769, 3, StateAction::Weapon(light2), StateNum::Lightdone), // Sgunflash2
    state(SpriteNum::Sht2, 0, 1, StateAction::Weapon(weapon_ready), StateNum::Dsgun), // Dsgun
    state(SpriteNum::Sht2, 0, 1, StateAction::Weapon(lower), StateNum::Dsgundown), // Dsgundown
    state(SpriteNum::Sht2, 0, 1, StateAction::Weapon(raise), StateNum::Dsgunup), // Dsgunup
    state(SpriteNum::Sht2, 0, 3, StateAction::None, StateNum::Dsgun2), // Dsgun1
    state(SpriteNum::Sht2, 0, 7, StateAction::Weapon(fire_shotgun2), StateNum::Dsgun3), // Dsgun2
    state(SpriteNum::Sht2, 1, 7, StateAction::None, StateNum::Dsgun4), // Dsgun3
    state(SpriteNum::Sht2, 2, 7, StateAction::Weapon(check_reload), StateNum::Dsgun5), // Dsgun4
    state(SpriteNum::Sht2, 3, 7, StateAction::Weapon(open_shotgun2), StateNum::Dsgun6), // Dsgun5
    state(SpriteNum::Sht2, 4, 7, StateAction::None, StateNum::Dsgun7), // Dsgun6
    state(SpriteNum::Sht2, 5, 7, StateAction::Weapon(load_shotgun2), StateNum::Dsgun8), // Dsgun7
    state(SpriteNum::Sht2, 6, 6, StateAction::None, StateNum::Dsgun9), // Dsgun8
    state(SpriteNum::Sht2, 7, 6, StateAction::Weapon(close_shotgun2), StateNum::Dsgun10), // Dsgun9
    state(SpriteNum::Sht2, 0, 5, StateAction::Weapon(re_fire), StateNum::Dsgun), // Dsgun10
    state(SpriteNum::Sht2, 1, 7, StateAction::None, StateNum::Dsnr2), // Dsnr1
    state(SpriteNum::Sht2, 0, 3, StateAction::None, StateNum::Dsgundown), // Dsnr2
    state(SpriteNum::Sht2, 32776, 5, StateAction::Weapon(light1), StateNum::Dsgunflash2), // Dsgunflash1
    state(SpriteNum::Sht2, 32777, 4, StateAction::Weapon(light2), StateNum::Lightdone), // Dsgunflash2
    state(SpriteNum::Chgg, 0, 1, StateAction::Weapon(weapon_ready), StateNum::Chain), // Chain
    state(SpriteNum::Chgg, 0, 1, StateAction::Weapon(lower), StateNum::Chaindown), // Chaindown
    state(SpriteNum::Chgg, 0, 1, StateAction::Weapon(raise), StateNum::Chainup), // Chainup
    state(SpriteNum::Chgg, 0, 4, StateAction::Weapon(fire_cgun), StateNum::Chain2), // Chain1
    state(SpriteNum::Chgg, 1, 4, StateAction::Weapon(fire_cgun), StateNum::Chain3), // Chain2
    state(SpriteNum::Chgg, 1, 0, StateAction::Weapon(re_fire), StateNum::Chain), // Chain3
    state(SpriteNum::Chgf, 32768, 5, StateAction::Weapon(light1), StateNum::Lightdone), // Chainflash1
    state(SpriteNum::Chgf, 32769, 5, StateAction::Weapon(light2), StateNum::Lightdone), // Chainflash2
    state(SpriteNum::Misg, 0, 1, StateAction::Weapon(weapon_ready), StateNum::Missile), // Missile
    state(SpriteNum::Misg, 0, 1, StateAction::Weapon(lower), StateNum::Missiledown), // Missiledown
    state(SpriteNum::Misg, 0, 1, StateAction::Weapon(raise), StateNum::Missileup), // Missileup
    state(SpriteNum::Misg, 1, 8, StateAction::Weapon(gun_flash), StateNum::Missile2), // Missile1
    state(SpriteNum::Misg, 1, 12, StateAction::Weapon(fire_missile), StateNum::Missile3), // Missile2
    state(SpriteNum::Misg, 1, 0, StateAction::Weapon(re_fire), StateNum::Missile), // Missile3
    state(SpriteNum::Misf, 32768, 3, StateAction::Weapon(light1), StateNum::Missileflash2), // Missileflash1
    state(SpriteNum::Misf, 32769, 4, StateAction::None, StateNum::Missileflash3), // Missileflash2
    state(SpriteNum::Misf, 32770, 4, StateAction::Weapon(light2), StateNum::Missileflash4), // Missileflash3
    state(SpriteNum::Misf, 32771, 4, StateAction::Weapon(light2), StateNum::Lightdone), // Missileflash4
    state(SpriteNum::Sawg, 2, 4, StateAction::Weapon(weapon_ready), StateNum::Sawb), // Saw
    state(SpriteNum::Sawg, 3, 4, StateAction::Weapon(weapon_ready), StateNum::Saw), // Sawb
    state(SpriteNum::Sawg, 2, 1, StateAction::Weapon(lower), StateNum::Sawdown), // Sawdown
    state(SpriteNum::Sawg, 2, 1, StateAction::Weapon(raise), StateNum::Sawup), // Sawup
    state(SpriteNum::Sawg, 0, 4, StateAction::Weapon(saw), StateNum::Saw2), // Saw1
    state(SpriteNum::Sawg, 1, 4, StateAction::Weapon(saw), StateNum::Saw3), // Saw2
    state(SpriteNum::Sawg, 1, 0, StateAction::Weapon(re_fire), StateNum::Saw), // Saw3
    state(SpriteNum::Plsg, 0, 1, StateAction::Weapon(weapon_ready), StateNum::Plasma), // Plasma
    state(SpriteNum::Plsg, 0, 1, StateAction::Weapon(lower), StateNum::Plasmadown), // Plasmadown
    state(SpriteNum::Plsg, 0, 1, StateAction::Weapon(raise), StateNum::Plasmaup), // Plasmaup
    state(SpriteNum::Plsg, 0, 3, StateAction::Weapon(fire_plasma), StateNum::Plasma2), // Plasma1
    state(SpriteNum::Plsg, 1, 20, StateAction::Weapon(re_fire), StateNum::Plasma), // Plasma2
    state(SpriteNum::Plsf, 32768, 4, StateAction::Weapon(light1), StateNum::Lightdone), // Plasmaflash1
    state(SpriteNum::Plsf, 32769, 4, StateAction::Weapon(light1), StateNum::Lightdone), // Plasmaflash2
    state(SpriteNum::Bfgg, 0, 1, StateAction::Weapon(weapon_ready), StateNum::Bfg), // Bfg
    state(SpriteNum::Bfgg, 0, 1, StateAction::Weapon(lower), StateNum::Bfgdown), // Bfgdown
    state(SpriteNum::Bfgg, 0, 1, StateAction::Weapon(raise), StateNum::Bfgup), // Bfgup
    state(SpriteNum::Bfgg, 0, 20, StateAction::Weapon(bfgsound), StateNum::Bfg2), // Bfg1
    state(SpriteNum::Bfgg, 1, 10, StateAction::Weapon(gun_flash), StateNum::Bfg3), // Bfg2
    state(SpriteNum::Bfgg, 1, 10, StateAction::Weapon(fire_bfg), StateNum::Bfg4), // Bfg3
    state(SpriteNum::Bfgg, 1, 20, StateAction::Weapon(re_fire), StateNum::Bfg), // Bfg4
    state(SpriteNum::Bfgf, 32768, 11, StateAction::Weapon(light1), StateNum::Bfgflash2), // Bfgflash1
    state(SpriteNum::Bfgf, 32769, 6, StateAction::Weapon(light2), StateNum::Lightdone), // Bfgflash2
    state(SpriteNum::Blud, 2, 8, StateAction::None, StateNum::Blood2), // Blood1
    state(SpriteNum::Blud, 1, 8, StateAction::None, StateNum::Blood3), // Blood2
    state(SpriteNum::Blud, 0, 8, StateAction::None, StateNum::Null), // Blood3
    state(SpriteNum::Puff, 32768, 4, StateAction::None, StateNum::Puff2), // Puff1
    state(SpriteNum::Puff, 1, 4, StateAction::None, StateNum::Puff3), // Puff2
    state(SpriteNum::Puff, 2, 4, StateAction::None, StateNum::Puff4), // Puff3
    state(SpriteNum::Puff, 3, 4, StateAction::None, StateNum::Null), // Puff4
    state(SpriteNum::Bal1, 32768, 4, StateAction::None, StateNum::Tball2), // Tball1
    state(SpriteNum::Bal1, 32769, 4, StateAction::None, StateNum::Tball1), // Tball2
    state(SpriteNum::Bal1, 32770, 6, StateAction::None, StateNum::Tballx2), // Tballx1
    state(SpriteNum::Bal1, 32771, 6, StateAction::None, StateNum::Tballx3), // Tballx2
    state(SpriteNum::Bal1, 32772, 6, StateAction::None, StateNum::Null), // Tballx3
    state(SpriteNum::Bal2, 32768, 4, StateAction::None, StateNum::Rball2), // Rball1
    state(SpriteNum::Bal2, 32769, 4, StateAction::None, StateNum::Rball1), // Rball2
    state(SpriteNum::Bal2, 32770, 6, StateAction::None, StateNum::Rballx2), // Rballx1
    state(SpriteNum::Bal2, 32771, 6, StateAction::None, StateNum::Rballx3), // Rballx2
    state(SpriteNum::Bal2, 32772, 6, StateAction::None, StateNum::Null), // Rballx3
    state(SpriteNum::Plss, 32768, 6, StateAction::None, StateNum::Plasball2), // Plasball
    state(SpriteNum::Plss, 32769, 6, StateAction::None, StateNum::Plasball), // Plasball2
    state(SpriteNum::Plse, 32768, 4, StateAction::None, StateNum::Plasexp2), // Plasexp
    state(SpriteNum::Plse, 32769, 4, StateAction::None, StateNum::Plasexp3), // Plasexp2
    state(SpriteNum::Plse, 32770, 4, StateAction::None, StateNum::Plasexp4), // Plasexp3
    state(SpriteNum::Plse, 32771, 4, StateAction::None, StateNum::Plasexp5), // Plasexp4
    state(SpriteNum::Plse, 32772, 4, StateAction::None, StateNum::Null), // Plasexp5
    state(SpriteNum::Misl, 32768, 1, StateAction::None, StateNum::Rocket), // Rocket
    state(SpriteNum::Bfs1, 32768, 4, StateAction::None, StateNum::Bfgshot2), // Bfgshot
    state(SpriteNum::Bfs1, 32769, 4, StateAction::None, StateNum::Bfgshot), // Bfgshot2
    state(SpriteNum::Bfe1, 32768, 8, StateAction::None, StateNum::Bfgland2), // Bfgland
    state(SpriteNum::Bfe1, 32769, 8, StateAction::None, StateNum::Bfgland3), // Bfgland2
    state(SpriteNum::Bfe1, 32770, 8, StateAction::Mobj(bfgspray), StateNum::Bfgland4), // Bfgland3
    state(SpriteNum::Bfe1, 32771, 8, StateAction::None, StateNum::Bfgland5), // Bfgland4
    state(SpriteNum::Bfe1, 32772, 8, StateAction::None, StateNum::Bfgland6), // Bfgland5
    state(SpriteNum::Bfe1, 32773, 8, StateAction::None, StateNum::Null), // Bfgland6
    state(SpriteNum::Bfe2, 32768, 8, StateAction::None, StateNum::Bfgexp2), // Bfgexp
    state(SpriteNum::Bfe2, 32769, 8, StateAction::None, StateNum::Bfgexp3), // Bfgexp2
    state(SpriteNum::Bfe2, 32770, 8, StateAction::None, StateNum::Bfgexp4), // Bfgexp3
    state(SpriteNum::Bfe2, 32771, 8, StateAction::None, StateNum::Null), // Bfgexp4
    state(SpriteNum::Misl, 32769, 8, StateAction::Mobj(explode), StateNum::Explode2), // Explode1
    state(SpriteNum::Misl, 32770, 6, StateAction::None, StateNum::Explode3), // Explode2
    state(SpriteNum::Misl, 32771, 4, StateAction::None, StateNum::Null), // Explode3
    state(SpriteNum::Tfog, 32768, 6, StateAction::None, StateNum::Tfog01), // Tfog
    state(SpriteNum::Tfog, 32769, 6, StateAction::None, StateNum::Tfog02), // Tfog01
    state(SpriteNum::Tfog, 32768, 6, StateAction::None, StateNum::Tfog2), // Tfog02
    state(SpriteNum::Tfog, 32769, 6, StateAction::None, StateNum::Tfog3), // Tfog2
    state(SpriteNum::Tfog, 32770, 6, StateAction::None, StateNum::Tfog4), // Tfog3
    state(SpriteNum::Tfog, 32771, 6, StateAction::None, StateNum::Tfog5), // Tfog4
    state(SpriteNum::Tfog, 32772, 6, StateAction::None, StateNum::Tfog6), // Tfog5
    state(SpriteNum::Tfog, 32773, 6, StateAction::None, StateNum::Tfog7), // Tfog6
    state(SpriteNum::Tfog, 32774, 6, StateAction::None, StateNum::Tfog8), // Tfog7
    state(SpriteNum::Tfog, 32775, 6, StateAction::None, StateNum::Tfog9), // Tfog8
    state(SpriteNum::Tfog, 32776, 6, StateAction::None, StateNum::Tfog10), // Tfog9
    state(SpriteNum::Tfog, 32777, 6, StateAction::None, StateNum::Null), // Tfog10
    state(SpriteNum::Ifog, 32768, 6, StateAction::None, StateNum::Ifog01), // Ifog
    state(SpriteNum::Ifog, 32769, 6, StateAction::None, StateNum::Ifog02), // Ifog01
    state(SpriteNum::Ifog, 32768, 6, StateAction::None, StateNum::Ifog2), // Ifog02
    state(SpriteNum::Ifog, 32769, 6, StateAction::None, StateNum::Ifog3), // Ifog2
    state(SpriteNum::Ifog, 32770, 6, StateAction::None, StateNum::Ifog4), // Ifog3
    state(SpriteNum::Ifog, 32771, 6, StateAction::None, StateNum::Ifog5), // Ifog4
    state(SpriteNum::Ifog, 32772, 6, StateAction::None, StateNum::Null), // Ifog5
    state(SpriteNum::Play, 0, -1, StateAction::None, StateNum::Null), // Play
    state(SpriteNum::Play, 0, 4, StateAction::None, StateNum::PlayRun2), // PlayRun1
    state(SpriteNum::Play, 1, 4, StateAction::None, StateNum::PlayRun3), // PlayRun2
    state(SpriteNum::Play, 2, 4, StateAction::None, StateNum::PlayRun4), // PlayRun3
    state(SpriteNum::Play, 3, 4, StateAction::None, StateNum::PlayRun1), // PlayRun4
    state(SpriteNum::Play, 4, 12, StateAction::None, StateNum::Play), // PlayAtk1
    state(SpriteNum::Play, 32773, 6, StateAction::None, StateNum::PlayAtk1), // PlayAtk2
    state(SpriteNum::Play, 6, 4, StateAction::None, StateNum::PlayPain2), // PlayPain
    state(SpriteNum::Play, 6, 4, StateAction::Mobj(pain), StateNum::Play), // PlayPain2
    state(SpriteNum::Play, 7, 10, StateAction::None, StateNum::PlayDie2), // PlayDie1
    state(SpriteNum::Play, 8, 10, StateAction::Mobj(player_scream), StateNum::PlayDie3), // PlayDie2
    state(SpriteNum::Play, 9, 10, StateAction::Mobj(fall), StateNum::PlayDie4), // PlayDie3
    state(SpriteNum::Play, 10, 10, StateAction::None, StateNum::PlayDie5), // PlayDie4
    state(SpriteNum::Play, 11, 10, StateAction::None, StateNum::PlayDie6), // PlayDie5
    state(SpriteNum::Play, 12, 10, StateAction::None, StateNum::PlayDie7), // PlayDie6
    state(SpriteNum::Play, 13, -1, StateAction::None, StateNum::Null), // PlayDie7
    state(SpriteNum::Play, 14, 5, StateAction::None, StateNum::PlayXdie2), // PlayXdie1
    state(SpriteNum::Play, 15, 5, StateAction::Mobj(xscream), StateNum::PlayXdie3), // PlayXdie2
    state(SpriteNum::Play, 16, 5, StateAction::Mobj(fall), StateNum::PlayXdie4), // PlayXdie3
    state(SpriteNum::Play, 17, 5, StateAction::None, StateNum::PlayXdie5), // PlayXdie4
    state(SpriteNum::Play, 18, 5, StateAction::None, StateNum::PlayXdie6), // PlayXdie5
    state(SpriteNum::Play, 19, 5, StateAction::None, StateNum::PlayXdie7), // PlayXdie6
    state(SpriteNum::Play, 20, 5, StateAction::None, StateNum::PlayXdie8), // PlayXdie7
    state(SpriteNum::Play, 21, 5, StateAction::None, StateNum::PlayXdie9), // PlayXdie8
    state(SpriteNum::Play, 22, -1, StateAction::None, StateNum::Null), // PlayXdie9
    state(SpriteNum::Poss, 0, 10, StateAction::Mobj(look), StateNum::PossStnd2), // PossStnd
    state(SpriteNum::Poss, 1, 10, StateAction::Mobj(look), StateNum::PossStnd), // PossStnd2
    state(SpriteNum::Poss, 0, 4, StateAction::Mobj(chase), StateNum::PossRun2), // PossRun1
    state(SpriteNum::Poss, 0, 4, StateAction::Mobj(chase), StateNum::PossRun3), // PossRun2
    state(SpriteNum::Poss, 1, 4, StateAction::Mobj(chase), StateNum::PossRun4), // PossRun3
    state(SpriteNum::Poss, 1, 4, StateAction::Mobj(chase), StateNum::PossRun5), // PossRun4
    state(SpriteNum::Poss, 2, 4, StateAction::Mobj(chase), StateNum::PossRun6), // PossRun5
    state(SpriteNum::Poss, 2, 4, StateAction::Mobj(chase), StateNum::PossRun7), // PossRun6
    state(SpriteNum::Poss, 3, 4, StateAction::Mobj(chase), StateNum::PossRun8), // PossRun7
    state(SpriteNum::Poss, 3, 4, StateAction::Mobj(chase), StateNum::PossRun1), // PossRun8
    state(SpriteNum::Poss, 4, 10, StateAction::Mobj(face_target), StateNum::PossAtk2), // PossAtk1
    state(SpriteNum::Poss, 5, 8, StateAction::Mobj(pos_attack), StateNum::PossAtk3), // PossAtk2
    state(SpriteNum::Poss, 4, 8, StateAction::None, StateNum::PossRun1), // PossAtk3
    state(SpriteNum::Poss, 6, 3, StateAction::None, StateNum::PossPain2), // PossPain
    state(SpriteNum::Poss, 6, 3, StateAction::Mobj(pain), StateNum::PossRun1), // PossPain2
    state(SpriteNum::Poss, 7, 5, StateAction::None, StateNum::PossDie2), // PossDie1
    state(SpriteNum::Poss, 8, 5, StateAction::Mobj(scream), StateNum::PossDie3), // PossDie2
    state(SpriteNum::Poss, 9, 5, StateAction::Mobj(fall), StateNum::PossDie4), // PossDie3
    state(SpriteNum::Poss, 10, 5, StateAction::None, StateNum::PossDie5), // PossDie4
    state(SpriteNum::Poss, 11, -1, StateAction::None, StateNum::Null), // PossDie5
    state(SpriteNum::Poss, 12, 5, StateAction::None, StateNum::PossXdie2), // PossXdie1
    state(SpriteNum::Poss, 13, 5, StateAction::Mobj(xscream), StateNum::PossXdie3), // PossXdie2
    state(SpriteNum::Poss, 14, 5, StateAction::Mobj(fall), StateNum::PossXdie4), // PossXdie3
    state(SpriteNum::Poss, 15, 5, StateAction::None, StateNum::PossXdie5), // PossXdie4
    state(SpriteNum::Poss, 16, 5, StateAction::None, StateNum::PossXdie6), // PossXdie5
    state(SpriteNum::Poss, 17, 5, StateAction::None, StateNum::PossXdie7), // PossXdie6
    state(SpriteNum::Poss, 18, 5, StateAction::None, StateNum::PossXdie8), // PossXdie7
    state(SpriteNum::Poss, 19, 5, StateAction::None, StateNum::PossXdie9), // PossXdie8
    state(SpriteNum::Poss, 20, -1, StateAction::None, StateNum::Null), // PossXdie9
    state(SpriteNum::Poss, 10, 5, StateAction::None, StateNum::PossRaise2), // PossRaise1
    state(SpriteNum::Poss, 9, 5, StateAction::None, StateNum::PossRaise3), // PossRaise2
    state(SpriteNum::Poss, 8, 5, StateAction::None, StateNum::PossRaise4), // PossRaise3
    state(SpriteNum::Poss, 7, 5, StateAction::None, StateNum::PossRun1), // PossRaise4
    state(SpriteNum::Spos, 0, 10, StateAction::Mobj(look), StateNum::SposStnd2), // SposStnd
    state(SpriteNum::Spos, 1, 10, StateAction::Mobj(look), StateNum::SposStnd), // SposStnd2
    state(SpriteNum::Spos, 0, 3, StateAction::Mobj(chase), StateNum::SposRun2), // SposRun1
    state(SpriteNum::Spos, 0, 3, StateAction::Mobj(chase), StateNum::SposRun3), // SposRun2
    state(SpriteNum::Spos, 1, 3, StateAction::Mobj(chase), StateNum::SposRun4), // SposRun3
    state(SpriteNum::Spos, 1, 3, StateAction::Mobj(chase), StateNum::SposRun5), // SposRun4
    state(SpriteNum::Spos, 2, 3, StateAction::Mobj(chase), StateNum::SposRun6), // SposRun5
    state(SpriteNum::Spos, 2, 3, StateAction::Mobj(chase), StateNum::SposRun7), // SposRun6
    state(SpriteNum::Spos, 3, 3, StateAction::Mobj(chase), StateNum::SposRun8), // SposRun7
    state(SpriteNum::Spos, 3, 3, StateAction::Mobj(chase), StateNum::SposRun1), // SposRun8
    state(SpriteNum::Spos, 4, 10, StateAction::Mobj(face_target), StateNum::SposAtk2), // SposAtk1
    state(SpriteNum::Spos, 32773, 10, StateAction::Mobj(spos_attack), StateNum::SposAtk3), // SposAtk2
    state(SpriteNum::Spos, 4, 10, StateAction::None, StateNum::SposRun1), // SposAtk3
    state(SpriteNum::Spos, 6, 3, StateAction::None, StateNum::SposPain2), // SposPain
    state(SpriteNum::Spos, 6, 3, StateAction::Mobj(pain), StateNum::SposRun1), // SposPain2
    state(SpriteNum::Spos, 7, 5, StateAction::None, StateNum::SposDie2), // SposDie1
    state(SpriteNum::Spos, 8, 5, StateAction::Mobj(scream), StateNum::SposDie3), // SposDie2
    state(SpriteNum::Spos, 9, 5, StateAction::Mobj(fall), StateNum::SposDie4), // SposDie3
    state(SpriteNum::Spos, 10, 5, StateAction::None, StateNum::SposDie5), // SposDie4
    state(SpriteNum::Spos, 11, -1, StateAction::None, StateNum::Null), // SposDie5
    state(SpriteNum::Spos, 12, 5, StateAction::None, StateNum::SposXdie2), // SposXdie1
    state(SpriteNum::Spos, 13, 5, StateAction::Mobj(xscream), StateNum::SposXdie3), // SposXdie2
    state(SpriteNum::Spos, 14, 5, StateAction::Mobj(fall), StateNum::SposXdie4), // SposXdie3
    state(SpriteNum::Spos, 15, 5, StateAction::None, StateNum::SposXdie5), // SposXdie4
    state(SpriteNum::Spos, 16, 5, StateAction::None, StateNum::SposXdie6), // SposXdie5
    state(SpriteNum::Spos, 17, 5, StateAction::None, StateNum::SposXdie7), // SposXdie6
    state(SpriteNum::Spos, 18, 5, StateAction::None, StateNum::SposXdie8), // SposXdie7
    state(SpriteNum::Spos, 19, 5, StateAction::None, StateNum::SposXdie9), // SposXdie8
    state(SpriteNum::Spos, 20, -1, StateAction::None, StateNum::Null), // SposXdie9
    state(SpriteNum::Spos, 11, 5, StateAction::None, StateNum::SposRaise2), // SposRaise1
    state(SpriteNum::Spos, 10, 5, StateAction::None, StateNum::SposRaise3), // SposRaise2
    state(SpriteNum::Spos, 9, 5, StateAction::None, StateNum::SposRaise4), // SposRaise3
    state(SpriteNum::Spos, 8, 5, StateAction::None, StateNum::SposRaise5), // SposRaise4
    state(SpriteNum::Spos, 7, 5, StateAction::None, StateNum::SposRun1), // SposRaise5
    state(SpriteNum::Vile, 0, 10, StateAction::Mobj(look), StateNum::VileStnd2), // VileStnd
    state(SpriteNum::Vile, 1, 10, StateAction::Mobj(look), StateNum::VileStnd), // VileStnd2
    state(SpriteNum::Vile, 0, 2, StateAction::Mobj(vile_chase), StateNum::VileRun2), // VileRun1
    state(SpriteNum::Vile, 0, 2, StateAction::Mobj(vile_chase), StateNum::VileRun3), // VileRun2
    state(SpriteNum::Vile, 1, 2, StateAction::Mobj(vile_chase), StateNum::VileRun4), // VileRun3
    state(SpriteNum::Vile, 1, 2, StateAction::Mobj(vile_chase), StateNum::VileRun5), // VileRun4
    state(SpriteNum::Vile, 2, 2, StateAction::Mobj(vile_chase), StateNum::VileRun6), // VileRun5
    state(SpriteNum::Vile, 2, 2, StateAction::Mobj(vile_chase), StateNum::VileRun7), // VileRun6
    state(SpriteNum::Vile, 3, 2, StateAction::Mobj(vile_chase), StateNum::VileRun8), // VileRun7
    state(SpriteNum::Vile, 3, 2, StateAction::Mobj(vile_chase), StateNum::VileRun9), // VileRun8
    state(SpriteNum::Vile, 4, 2, StateAction::Mobj(vile_chase), StateNum::VileRun10), // VileRun9
    state(SpriteNum::Vile, 4, 2, StateAction::Mobj(vile_chase), StateNum::VileRun11), // VileRun10
    state(SpriteNum::Vile, 5, 2, StateAction::Mobj(vile_chase), StateNum::VileRun12), // VileRun11
    state(SpriteNum::Vile, 5, 2, StateAction::Mobj(vile_chase), StateNum::VileRun1), // VileRun12
    state(SpriteNum::Vile, 32774, 0, StateAction::Mobj(vile_start), StateNum::VileAtk2), // VileAtk1
    state(SpriteNum::Vile, 32774, 10, StateAction::Mobj(face_target), StateNum::VileAtk3), // VileAtk2
    state(SpriteNum::Vile, 32775, 8, StateAction::Mobj(vile_target), StateNum::VileAtk4), // VileAtk3
    state(SpriteNum::Vile, 32776, 8, StateAction::Mobj(face_target), StateNum::VileAtk5), // VileAtk4
    state(SpriteNum::Vile, 32777, 8, StateAction::Mobj(face_target), StateNum::VileAtk6), // VileAtk5
    state(SpriteNum::Vile, 32778, 8, StateAction::Mobj(face_target), StateNum::VileAtk7), // VileAtk6
    state(SpriteNum::Vile, 32779, 8, StateAction::Mobj(face_target), StateNum::VileAtk8), // VileAtk7
    state(SpriteNum::Vile, 32780, 8, StateAction::Mobj(face_target), StateNum::VileAtk9), // VileAtk8
    state(SpriteNum::Vile, 32781, 8, StateAction::Mobj(face_target), StateNum::VileAtk10), // VileAtk9
    state(SpriteNum::Vile, 32782, 8, StateAction::Mobj(vile_attack), StateNum::VileAtk11), // VileAtk10
    state(SpriteNum::Vile, 32783, 20, StateAction::None, StateNum::VileRun1), // VileAtk11
    state(SpriteNum::Vile, 32794, 10, StateAction::None, StateNum::VileHeal2), // VileHeal1
    state(SpriteNum::Vile, 32795, 10, StateAction::None, StateNum::VileHeal3), // VileHeal2
    state(SpriteNum::Vile, 32796, 10, StateAction::None, StateNum::VileRun1), // VileHeal3
    state(SpriteNum::Vile, 16, 5, StateAction::None, StateNum::VilePain2), // VilePain
    state(SpriteNum::Vile, 16, 5, StateAction::Mobj(pain), StateNum::VileRun1), // VilePain2
    state(SpriteNum::Vile, 16, 7, StateAction::None, StateNum::VileDie2), // VileDie1
    state(SpriteNum::Vile, 17, 7, StateAction::Mobj(scream), StateNum::VileDie3), // VileDie2
    state(SpriteNum::Vile, 18, 7, StateAction::Mobj(fall), StateNum::VileDie4), // VileDie3
    state(SpriteNum::Vile, 19, 7, StateAction::None, StateNum::VileDie5), // VileDie4
    state(SpriteNum::Vile, 20, 7, StateAction::None, StateNum::VileDie6), // VileDie5
    state(SpriteNum::Vile, 21, 7, StateAction::None, StateNum::VileDie7), // VileDie6
    state(SpriteNum::Vile, 22, 7, StateAction::None, StateNum::VileDie8), // VileDie7
    state(SpriteNum::Vile, 23, 5, StateAction::None, StateNum::VileDie9), // VileDie8
    state(SpriteNum::Vile, 24, 5, StateAction::None, StateNum::VileDie10), // VileDie9
    state(SpriteNum::Vile, 25, -1, StateAction::None, StateNum::Null), // VileDie10
    state(SpriteNum::Fire, 32768, 2, StateAction::Mobj(start_fire), StateNum::Fire2), // Fire1
    state(SpriteNum::Fire, 32769, 2, StateAction::Mobj(a_fire), StateNum::Fire3), // Fire2
    state(SpriteNum::Fire, 32768, 2, StateAction::Mobj(a_fire), StateNum::Fire4), // Fire3
    state(SpriteNum::Fire, 32769, 2, StateAction::Mobj(a_fire), StateNum::Fire5), // Fire4
    state(SpriteNum::Fire, 32770, 2, StateAction::Mobj(fire_crackle), StateNum::Fire6), // Fire5
    state(SpriteNum::Fire, 32769, 2, StateAction::Mobj(a_fire), StateNum::Fire7), // Fire6
    state(SpriteNum::Fire, 32770, 2, StateAction::Mobj(a_fire), StateNum::Fire8), // Fire7
    state(SpriteNum::Fire, 32769, 2, StateAction::Mobj(a_fire), StateNum::Fire9), // Fire8
    state(SpriteNum::Fire, 32770, 2, StateAction::Mobj(a_fire), StateNum::Fire10), // Fire9
    state(SpriteNum::Fire, 32771, 2, StateAction::Mobj(a_fire), StateNum::Fire11), // Fire10
    state(SpriteNum::Fire, 32770, 2, StateAction::Mobj(a_fire), StateNum::Fire12), // Fire11
    state(SpriteNum::Fire, 32771, 2, StateAction::Mobj(a_fire), StateNum::Fire13), // Fire12
    state(SpriteNum::Fire, 32770, 2, StateAction::Mobj(a_fire), StateNum::Fire14), // Fire13
    state(SpriteNum::Fire, 32771, 2, StateAction::Mobj(a_fire), StateNum::Fire15), // Fire14
    state(SpriteNum::Fire, 32772, 2, StateAction::Mobj(a_fire), StateNum::Fire16), // Fire15
    state(SpriteNum::Fire, 32771, 2, StateAction::Mobj(a_fire), StateNum::Fire17), // Fire16
    state(SpriteNum::Fire, 32772, 2, StateAction::Mobj(a_fire), StateNum::Fire18), // Fire17
    state(SpriteNum::Fire, 32771, 2, StateAction::Mobj(a_fire), StateNum::Fire19), // Fire18
    state(SpriteNum::Fire, 32772, 2, StateAction::Mobj(fire_crackle), StateNum::Fire20), // Fire19
    state(SpriteNum::Fire, 32773, 2, StateAction::Mobj(a_fire), StateNum::Fire21), // Fire20
    state(SpriteNum::Fire, 32772, 2, StateAction::Mobj(a_fire), StateNum::Fire22), // Fire21
    state(SpriteNum::Fire, 32773, 2, StateAction::Mobj(a_fire), StateNum::Fire23), // Fire22
    state(SpriteNum::Fire, 32772, 2, StateAction::Mobj(a_fire), StateNum::Fire24), // Fire23
    state(SpriteNum::Fire, 32773, 2, StateAction::Mobj(a_fire), StateNum::Fire25), // Fire24
    state(SpriteNum::Fire, 32774, 2, StateAction::Mobj(a_fire), StateNum::Fire26), // Fire25
    state(SpriteNum::Fire, 32775, 2, StateAction::Mobj(a_fire), StateNum::Fire27), // Fire26
    state(SpriteNum::Fire, 32774, 2, StateAction::Mobj(a_fire), StateNum::Fire28), // Fire27
    state(SpriteNum::Fire, 32775, 2, StateAction::Mobj(a_fire), StateNum::Fire29), // Fire28
    state(SpriteNum::Fire, 32774, 2, StateAction::Mobj(a_fire), StateNum::Fire30), // Fire29
    state(SpriteNum::Fire, 32775, 2, StateAction::Mobj(a_fire), StateNum::Null), // Fire30
    state(SpriteNum::Puff, 1, 4, StateAction::None, StateNum::Smoke2), // Smoke1
    state(SpriteNum::Puff, 2, 4, StateAction::None, StateNum::Smoke3), // Smoke2
    state(SpriteNum::Puff, 1, 4, StateAction::None, StateNum::Smoke4), // Smoke3
    state(SpriteNum::Puff, 2, 4, StateAction::None, StateNum::Smoke5), // Smoke4
    state(SpriteNum::Puff, 3, 4, StateAction::None, StateNum::Null), // Smoke5
    state(SpriteNum::Fatb, 32768, 2, StateAction::Mobj(a_tracer), StateNum::Tracer2), // Tracer
    state(SpriteNum::Fatb, 32769, 2, StateAction::Mobj(a_tracer), StateNum::Tracer), // Tracer2
    state(SpriteNum::Fbxp, 32768, 8, StateAction::None, StateNum::Traceexp2), // Traceexp1
    state(SpriteNum::Fbxp, 32769, 6, StateAction::None, StateNum::Traceexp3), // Traceexp2
    state(SpriteNum::Fbxp, 32770, 4, StateAction::None, StateNum::Null), // Traceexp3
    state(SpriteNum::Skel, 0, 10, StateAction::Mobj(look), StateNum::SkelStnd2), // SkelStnd
    state(SpriteNum::Skel, 1, 10, StateAction::Mobj(look), StateNum::SkelStnd), // SkelStnd2
    state(SpriteNum::Skel, 0, 2, StateAction::Mobj(chase), StateNum::SkelRun2), // SkelRun1
    state(SpriteNum::Skel, 0, 2, StateAction::Mobj(chase), StateNum::SkelRun3), // SkelRun2
    state(SpriteNum::Skel, 1, 2, StateAction::Mobj(chase), StateNum::SkelRun4), // SkelRun3
    state(SpriteNum::Skel, 1, 2, StateAction::Mobj(chase), StateNum::SkelRun5), // SkelRun4
    state(SpriteNum::Skel, 2, 2, StateAction::Mobj(chase), StateNum::SkelRun6), // SkelRun5
    state(SpriteNum::Skel, 2, 2, StateAction::Mobj(chase), StateNum::SkelRun7), // SkelRun6
    state(SpriteNum::Skel, 3, 2, StateAction::Mobj(chase), StateNum::SkelRun8), // SkelRun7
    state(SpriteNum::Skel, 3, 2, StateAction::Mobj(chase), StateNum::SkelRun9), // SkelRun8
    state(SpriteNum::Skel, 4, 2, StateAction::Mobj(chase), StateNum::SkelRun10), // SkelRun9
    state(SpriteNum::Skel, 4, 2, StateAction::Mobj(chase), StateNum::SkelRun11), // SkelRun10
    state(SpriteNum::Skel, 5, 2, StateAction::Mobj(chase), StateNum::SkelRun12), // SkelRun11
    state(SpriteNum::Skel, 5, 2, StateAction::Mobj(chase), StateNum::SkelRun1), // SkelRun12
    state(SpriteNum::Skel, 6, 0, StateAction::Mobj(face_target), StateNum::SkelFist2), // SkelFist1
    state(SpriteNum::Skel, 6, 6, StateAction::Mobj(skel_whoosh), StateNum::SkelFist3), // SkelFist2
    state(SpriteNum::Skel, 7, 6, StateAction::Mobj(face_target), StateNum::SkelFist4), // SkelFist3
    state(SpriteNum::Skel, 8, 6, StateAction::Mobj(skel_fist), StateNum::SkelRun1), // SkelFist4
    state(SpriteNum::Skel, 32777, 0, StateAction::Mobj(face_target), StateNum::SkelMiss2), // SkelMiss1
    state(SpriteNum::Skel, 32777, 10, StateAction::Mobj(face_target), StateNum::SkelMiss3), // SkelMiss2
    state(SpriteNum::Skel, 10, 10, StateAction::Mobj(skel_missile), StateNum::SkelMiss4), // SkelMiss3
    state(SpriteNum::Skel, 10, 10, StateAction::Mobj(face_target), StateNum::SkelRun1), // SkelMiss4
    state(SpriteNum::Skel, 11, 5, StateAction::None, StateNum::SkelPain2), // SkelPain
    state(SpriteNum::Skel, 11, 5, StateAction::Mobj(pain), StateNum::SkelRun1), // SkelPain2
    state(SpriteNum::Skel, 11, 7, StateAction::None, StateNum::SkelDie2), // SkelDie1
    state(SpriteNum::Skel, 12, 7, StateAction::None, StateNum::SkelDie3), // SkelDie2
    state(SpriteNum::Skel, 13, 7, StateAction::Mobj(scream), StateNum::SkelDie4), // SkelDie3
    state(SpriteNum::Skel, 14, 7, StateAction::Mobj(fall), StateNum::SkelDie5), // SkelDie4
    state(SpriteNum::Skel, 15, 7, StateAction::None, StateNum::SkelDie6), // SkelDie5
    state(SpriteNum::Skel, 16, -1, StateAction::None, StateNum::Null), // SkelDie6
    state(SpriteNum::Skel, 16, 5, StateAction::None, StateNum::SkelRaise2), // SkelRaise1
    state(SpriteNum::Skel, 15, 5, StateAction::None, StateNum::SkelRaise3), // SkelRaise2
    state(SpriteNum::Skel, 14, 5, StateAction::None, StateNum::SkelRaise4), // SkelRaise3
    state(SpriteNum::Skel, 13, 5, StateAction::None, StateNum::SkelRaise5), // SkelRaise4
    state(SpriteNum::Skel, 12, 5, StateAction::None, StateNum::SkelRaise6), // SkelRaise5
    state(SpriteNum::Skel, 11, 5, StateAction::None, StateNum::SkelRun1), // SkelRaise6
    state(SpriteNum::Manf, 32768, 4, StateAction::None, StateNum::Fatshot2), // Fatshot1
    state(SpriteNum::Manf, 32769, 4, StateAction::None, StateNum::Fatshot1), // Fatshot2
    state(SpriteNum::Misl, 32769, 8, StateAction::None, StateNum::Fatshotx2), // Fatshotx1
    state(SpriteNum::Misl, 32770, 6, StateAction::None, StateNum::Fatshotx3), // Fatshotx2
    state(SpriteNum::Misl, 32771, 4, StateAction::None, StateNum::Null), // Fatshotx3
    state(SpriteNum::Fatt, 0, 15, StateAction::Mobj(look), StateNum::FattStnd2), // FattStnd
    state(SpriteNum::Fatt, 1, 15, StateAction::Mobj(look), StateNum::FattStnd), // FattStnd2
    state(SpriteNum::Fatt, 0, 4, StateAction::Mobj(chase), StateNum::FattRun2), // FattRun1
    state(SpriteNum::Fatt, 0, 4, StateAction::Mobj(chase), StateNum::FattRun3), // FattRun2
    state(SpriteNum::Fatt, 1, 4, StateAction::Mobj(chase), StateNum::FattRun4), // FattRun3
    state(SpriteNum::Fatt, 1, 4, StateAction::Mobj(chase), StateNum::FattRun5), // FattRun4
    state(SpriteNum::Fatt, 2, 4, StateAction::Mobj(chase), StateNum::FattRun6), // FattRun5
    state(SpriteNum::Fatt, 2, 4, StateAction::Mobj(chase), StateNum::FattRun7), // FattRun6
    state(SpriteNum::Fatt, 3, 4, StateAction::Mobj(chase), StateNum::FattRun8), // FattRun7
    state(SpriteNum::Fatt, 3, 4, StateAction::Mobj(chase), StateNum::FattRun9), // FattRun8
    state(SpriteNum::Fatt, 4, 4, StateAction::Mobj(chase), StateNum::FattRun10), // FattRun9
    state(SpriteNum::Fatt, 4, 4, StateAction::Mobj(chase), StateNum::FattRun11), // FattRun10
    state(SpriteNum::Fatt, 5, 4, StateAction::Mobj(chase), StateNum::FattRun12), // FattRun11
    state(SpriteNum::Fatt, 5, 4, StateAction::Mobj(chase), StateNum::FattRun1), // FattRun12
    state(SpriteNum::Fatt, 6, 20, StateAction::Mobj(fat_raise), StateNum::FattAtk2), // FattAtk1
    state(SpriteNum::Fatt, 32775, 10, StateAction::Mobj(fat_attack1), StateNum::FattAtk3), // FattAtk2
    state(SpriteNum::Fatt, 8, 5, StateAction::Mobj(face_target), StateNum::FattAtk4), // FattAtk3
    state(SpriteNum::Fatt, 6, 5, StateAction::Mobj(face_target), StateNum::FattAtk5), // FattAtk4
    state(SpriteNum::Fatt, 32775, 10, StateAction::Mobj(fat_attack2), StateNum::FattAtk6), // FattAtk5
    state(SpriteNum::Fatt, 8, 5, StateAction::Mobj(face_target), StateNum::FattAtk7), // FattAtk6
    state(SpriteNum::Fatt, 6, 5, StateAction::Mobj(face_target), StateNum::FattAtk8), // FattAtk7
    state(SpriteNum::Fatt, 32775, 10, StateAction::Mobj(fat_attack3), StateNum::FattAtk9), // FattAtk8
    state(SpriteNum::Fatt, 8, 5, StateAction::Mobj(face_target), StateNum::FattAtk10), // FattAtk9
    state(SpriteNum::Fatt, 6, 5, StateAction::Mobj(face_target), StateNum::FattRun1), // FattAtk10
    state(SpriteNum::Fatt, 9, 3, StateAction::None, StateNum::FattPain2), // FattPain
    state(SpriteNum::Fatt, 9, 3, StateAction::Mobj(pain), StateNum::FattRun1), // FattPain2
    state(SpriteNum::Fatt, 10, 6, StateAction::None, StateNum::FattDie2), // FattDie1
    state(SpriteNum::Fatt, 11, 6, StateAction::Mobj(scream), StateNum::FattDie3), // FattDie2
    state(SpriteNum::Fatt, 12, 6, StateAction::Mobj(fall), StateNum::FattDie4), // FattDie3
    state(SpriteNum::Fatt, 13, 6, StateAction::None, StateNum::FattDie5), // FattDie4
    state(SpriteNum::Fatt, 14, 6, StateAction::None, StateNum::FattDie6), // FattDie5
    state(SpriteNum::Fatt, 15, 6, StateAction::None, StateNum::FattDie7), // FattDie6
    state(SpriteNum::Fatt, 16, 6, StateAction::None, StateNum::FattDie8), // FattDie7
    state(SpriteNum::Fatt, 17, 6, StateAction::None, StateNum::FattDie9), // FattDie8
    state(SpriteNum::Fatt, 18, 6, StateAction::None, StateNum::FattDie10), // FattDie9
    state(SpriteNum::Fatt, 19, -1, StateAction::Mobj(boss_death), StateNum::Null), // FattDie10
    state(SpriteNum::Fatt, 17, 5, StateAction::None, StateNum::FattRaise2), // FattRaise1
    state(SpriteNum::Fatt, 16, 5, StateAction::None, StateNum::FattRaise3), // FattRaise2
    state(SpriteNum::Fatt, 15, 5, StateAction::None, StateNum::FattRaise4), // FattRaise3
    state(SpriteNum::Fatt, 14, 5, StateAction::None, StateNum::FattRaise5), // FattRaise4
    state(SpriteNum::Fatt, 13, 5, StateAction::None, StateNum::FattRaise6), // FattRaise5
    state(SpriteNum::Fatt, 12, 5, StateAction::None, StateNum::FattRaise7), // FattRaise6
    state(SpriteNum::Fatt, 11, 5, StateAction::None, StateNum::FattRaise8), // FattRaise7
    state(SpriteNum::Fatt, 10, 5, StateAction::None, StateNum::FattRun1), // FattRaise8
    state(SpriteNum::Cpos, 0, 10, StateAction::Mobj(look), StateNum::CposStnd2), // CposStnd
    state(SpriteNum::Cpos, 1, 10, StateAction::Mobj(look), StateNum::CposStnd), // CposStnd2
    state(SpriteNum::Cpos, 0, 3, StateAction::Mobj(chase), StateNum::CposRun2), // CposRun1
    state(SpriteNum::Cpos, 0, 3, StateAction::Mobj(chase), StateNum::CposRun3), // CposRun2
    state(SpriteNum::Cpos, 1, 3, StateAction::Mobj(chase), StateNum::CposRun4), // CposRun3
    state(SpriteNum::Cpos, 1, 3, StateAction::Mobj(chase), StateNum::CposRun5), // CposRun4
    state(SpriteNum::Cpos, 2, 3, StateAction::Mobj(chase), StateNum::CposRun6), // CposRun5
    state(SpriteNum::Cpos, 2, 3, StateAction::Mobj(chase), StateNum::CposRun7), // CposRun6
    state(SpriteNum::Cpos, 3, 3, StateAction::Mobj(chase), StateNum::CposRun8), // CposRun7
    state(SpriteNum::Cpos, 3, 3, StateAction::Mobj(chase), StateNum::CposRun1), // CposRun8
    state(SpriteNum::Cpos, 4, 10, StateAction::Mobj(face_target), StateNum::CposAtk2), // CposAtk1
    state(SpriteNum::Cpos, 32773, 4, StateAction::Mobj(cpos_attack), StateNum::CposAtk3), // CposAtk2
    state(SpriteNum::Cpos, 32772, 4, StateAction::Mobj(cpos_attack), StateNum::CposAtk4), // CposAtk3
    state(SpriteNum::Cpos, 5, 1, StateAction::Mobj(cpos_refire), StateNum::CposAtk2), // CposAtk4
    state(SpriteNum::Cpos, 6, 3, StateAction::None, StateNum::CposPain2), // CposPain
    state(SpriteNum::Cpos, 6, 3, StateAction::Mobj(pain), StateNum::CposRun1), // CposPain2
    state(SpriteNum::Cpos, 7, 5, StateAction::None, StateNum::CposDie2), // CposDie1
    state(SpriteNum::Cpos, 8, 5, StateAction::Mobj(scream), StateNum::CposDie3), // CposDie2
    state(SpriteNum::Cpos, 9, 5, StateAction::Mobj(fall), StateNum::CposDie4), // CposDie3
    state(SpriteNum::Cpos, 10, 5, StateAction::None, StateNum::CposDie5), // CposDie4
    state(SpriteNum::Cpos, 11, 5, StateAction::None, StateNum::CposDie6), // CposDie5
    state(SpriteNum::Cpos, 12, 5, StateAction::None, StateNum::CposDie7), // CposDie6
    state(SpriteNum::Cpos, 13, -1, StateAction::None, StateNum::Null), // CposDie7
    state(SpriteNum::Cpos, 14, 5, StateAction::None, StateNum::CposXdie2), // CposXdie1
    state(SpriteNum::Cpos, 15, 5, StateAction::Mobj(xscream), StateNum::CposXdie3), // CposXdie2
    state(SpriteNum::Cpos, 16, 5, StateAction::Mobj(fall), StateNum::CposXdie4), // CposXdie3
    state(SpriteNum::Cpos, 17, 5, StateAction::None, StateNum::CposXdie5), // CposXdie4
    state(SpriteNum::Cpos, 18, 5, StateAction::None, StateNum::CposXdie6), // CposXdie5
    state(SpriteNum::Cpos, 19, -1, StateAction::None, StateNum::Null), // CposXdie6
    state(SpriteNum::Cpos, 13, 5, StateAction::None, StateNum::CposRaise2), // CposRaise1
    state(SpriteNum::Cpos, 12, 5, StateAction::None, StateNum::CposRaise3), // CposRaise2
    state(SpriteNum::Cpos, 11, 5, StateAction::None, StateNum::CposRaise4), // CposRaise3
    state(SpriteNum::Cpos, 10, 5, StateAction::None, StateNum::CposRaise5), // CposRaise4
    state(SpriteNum::Cpos, 9, 5, StateAction::None, StateNum::CposRaise6), // CposRaise5
    state(SpriteNum::Cpos, 8, 5, StateAction::None, StateNum::CposRaise7), // CposRaise6
    state(SpriteNum::Cpos, 7, 5, StateAction::None, StateNum::CposRun1), // CposRaise7
    state(SpriteNum::Troo, 0, 10, StateAction::Mobj(look), StateNum::TrooStnd2), // TrooStnd
    state(SpriteNum::Troo, 1, 10, StateAction::Mobj(look), StateNum::TrooStnd), // TrooStnd2
    state(SpriteNum::Troo, 0, 3, StateAction::Mobj(chase), StateNum::TrooRun2), // TrooRun1
    state(SpriteNum::Troo, 0, 3, StateAction::Mobj(chase), StateNum::TrooRun3), // TrooRun2
    state(SpriteNum::Troo, 1, 3, StateAction::Mobj(chase), StateNum::TrooRun4), // TrooRun3
    state(SpriteNum::Troo, 1, 3, StateAction::Mobj(chase), StateNum::TrooRun5), // TrooRun4
    state(SpriteNum::Troo, 2, 3, StateAction::Mobj(chase), StateNum::TrooRun6), // TrooRun5
    state(SpriteNum::Troo, 2, 3, StateAction::Mobj(chase), StateNum::TrooRun7), // TrooRun6
    state(SpriteNum::Troo, 3, 3, StateAction::Mobj(chase), StateNum::TrooRun8), // TrooRun7
    state(SpriteNum::Troo, 3, 3, StateAction::Mobj(chase), StateNum::TrooRun1), // TrooRun8
    state(SpriteNum::Troo, 4, 8, StateAction::Mobj(face_target), StateNum::TrooAtk2), // TrooAtk1
    state(SpriteNum::Troo, 5, 8, StateAction::Mobj(face_target), StateNum::TrooAtk3), // TrooAtk2
    state(SpriteNum::Troo, 6, 6, StateAction::Mobj(troop_attack), StateNum::TrooRun1), // TrooAtk3
    state(SpriteNum::Troo, 7, 2, StateAction::None, StateNum::TrooPain2), // TrooPain
    state(SpriteNum::Troo, 7, 2, StateAction::Mobj(pain), StateNum::TrooRun1), // TrooPain2
    state(SpriteNum::Troo, 8, 8, StateAction::None, StateNum::TrooDie2), // TrooDie1
    state(SpriteNum::Troo, 9, 8, StateAction::Mobj(scream), StateNum::TrooDie3), // TrooDie2
    state(SpriteNum::Troo, 10, 6, StateAction::None, StateNum::TrooDie4), // TrooDie3
    state(SpriteNum::Troo, 11, 6, StateAction::Mobj(fall), StateNum::TrooDie5), // TrooDie4
    state(SpriteNum::Troo, 12, -1, StateAction::None, StateNum::Null), // TrooDie5
    state(SpriteNum::Troo, 13, 5, StateAction::None, StateNum::TrooXdie2), // TrooXdie1
    state(SpriteNum::Troo, 14, 5, StateAction::Mobj(xscream), StateNum::TrooXdie3), // TrooXdie2
    state(SpriteNum::Troo, 15, 5, StateAction::None, StateNum::TrooXdie4), // TrooXdie3
    state(SpriteNum::Troo, 16, 5, StateAction::Mobj(fall), StateNum::TrooXdie5), // TrooXdie4
    state(SpriteNum::Troo, 17, 5, StateAction::None, StateNum::TrooXdie6), // TrooXdie5
    state(SpriteNum::Troo, 18, 5, StateAction::None, StateNum::TrooXdie7), // TrooXdie6
    state(SpriteNum::Troo, 19, 5, StateAction::None, StateNum::TrooXdie8), // TrooXdie7
    state(SpriteNum::Troo, 20, -1, StateAction::None, StateNum::Null), // TrooXdie8
    state(SpriteNum::Troo, 12, 8, StateAction::None, StateNum::TrooRaise2), // TrooRaise1
    state(SpriteNum::Troo, 11, 8, StateAction::None, StateNum::TrooRaise3), // TrooRaise2
    state(SpriteNum::Troo, 10, 6, StateAction::None, StateNum::TrooRaise4), // TrooRaise3
    state(SpriteNum::Troo, 9, 6, StateAction::None, StateNum::TrooRaise5), // TrooRaise4
    state(SpriteNum::Troo, 8, 6, StateAction::None, StateNum::TrooRun1), // TrooRaise5
    state(SpriteNum::Sarg, 0, 10, StateAction::Mobj(look), StateNum::SargStnd2), // SargStnd
    state(SpriteNum::Sarg, 1, 10, StateAction::Mobj(look), StateNum::SargStnd), // SargStnd2
    state(SpriteNum::Sarg, 0, 2, StateAction::Mobj(chase), StateNum::SargRun2), // SargRun1
    state(SpriteNum::Sarg, 0, 2, StateAction::Mobj(chase), StateNum::SargRun3), // SargRun2
    state(SpriteNum::Sarg, 1, 2, StateAction::Mobj(chase), StateNum::SargRun4), // SargRun3
    state(SpriteNum::Sarg, 1, 2, StateAction::Mobj(chase), StateNum::SargRun5), // SargRun4
    state(SpriteNum::Sarg, 2, 2, StateAction::Mobj(chase), StateNum::SargRun6), // SargRun5
    state(SpriteNum::Sarg, 2, 2, StateAction::Mobj(chase), StateNum::SargRun7), // SargRun6
    state(SpriteNum::Sarg, 3, 2, StateAction::Mobj(chase), StateNum::SargRun8), // SargRun7
    state(SpriteNum::Sarg, 3, 2, StateAction::Mobj(chase), StateNum::SargRun1), // SargRun8
    state(SpriteNum::Sarg, 4, 8, StateAction::Mobj(face_target), StateNum::SargAtk2), // SargAtk1
    state(SpriteNum::Sarg, 5, 8, StateAction::Mobj(face_target), StateNum::SargAtk3), // SargAtk2
    state(SpriteNum::Sarg, 6, 8, StateAction::Mobj(sarg_attack), StateNum::SargRun1), // SargAtk3
    state(SpriteNum::Sarg, 7, 2, StateAction::None, StateNum::SargPain2), // SargPain
    state(SpriteNum::Sarg, 7, 2, StateAction::Mobj(pain), StateNum::SargRun1), // SargPain2
    state(SpriteNum::Sarg, 8, 8, StateAction::None, StateNum::SargDie2), // SargDie1
    state(SpriteNum::Sarg, 9, 8, StateAction::Mobj(scream), StateNum::SargDie3), // SargDie2
    state(SpriteNum::Sarg, 10, 4, StateAction::None, StateNum::SargDie4), // SargDie3
    state(SpriteNum::Sarg, 11, 4, StateAction::Mobj(fall), StateNum::SargDie5), // SargDie4
    state(SpriteNum::Sarg, 12, 4, StateAction::None, StateNum::SargDie6), // SargDie5
    state(SpriteNum::Sarg, 13, -1, StateAction::None, StateNum::Null), // SargDie6
    state(SpriteNum::Sarg, 13, 5, StateAction::None, StateNum::SargRaise2), // SargRaise1
    state(SpriteNum::Sarg, 12, 5, StateAction::None, StateNum::SargRaise3), // SargRaise2
    state(SpriteNum::Sarg, 11, 5, StateAction::None, StateNum::SargRaise4), // SargRaise3
    state(SpriteNum::Sarg, 10, 5, StateAction::None, StateNum::SargRaise5), // SargRaise4
    state(SpriteNum::Sarg, 9, 5, StateAction::None, StateNum::SargRaise6), // SargRaise5
    state(SpriteNum::Sarg, 8, 5, StateAction::None, StateNum::SargRun1), // SargRaise6
    state(SpriteNum::Head, 0, 10, StateAction::Mobj(look), StateNum::HeadStnd), // HeadStnd
    state(SpriteNum::Head, 0, 3, StateAction::Mobj(chase), StateNum::HeadRun1), // HeadRun1
    state(SpriteNum::Head, 1, 5, StateAction::Mobj(face_target), StateNum::HeadAtk2), // HeadAtk1
    state(SpriteNum::Head, 2, 5, StateAction::Mobj(face_target), StateNum::HeadAtk3), // HeadAtk2
    state(SpriteNum::Head, 32771, 5, StateAction::Mobj(head_attack), StateNum::HeadRun1), // HeadAtk3
    state(SpriteNum::Head, 4, 3, StateAction::None, StateNum::HeadPain2), // HeadPain
    state(SpriteNum::Head, 4, 3, StateAction::Mobj(pain), StateNum::HeadPain3), // HeadPain2
    state(SpriteNum::Head, 5, 6, StateAction::None, StateNum::HeadRun1), // HeadPain3
    state(SpriteNum::Head, 6, 8, StateAction::None, StateNum::HeadDie2), // HeadDie1
    state(SpriteNum::Head, 7, 8, StateAction::Mobj(scream), StateNum::HeadDie3), // HeadDie2
    state(SpriteNum::Head, 8, 8, StateAction::None, StateNum::HeadDie4), // HeadDie3
    state(SpriteNum::Head, 9, 8, StateAction::None, StateNum::HeadDie5), // HeadDie4
    state(SpriteNum::Head, 10, 8, StateAction::Mobj(fall), StateNum::HeadDie6), // HeadDie5
    state(SpriteNum::Head, 11, -1, StateAction::None, StateNum::Null), // HeadDie6
    state(SpriteNum::Head, 11, 8, StateAction::None, StateNum::HeadRaise2), // HeadRaise1
    state(SpriteNum::Head, 10, 8, StateAction::None, StateNum::HeadRaise3), // HeadRaise2
    state(SpriteNum::Head, 9, 8, StateAction::None, StateNum::HeadRaise4), // HeadRaise3
    state(SpriteNum::Head, 8, 8, StateAction::None, StateNum::HeadRaise5), // HeadRaise4
    state(SpriteNum::Head, 7, 8, StateAction::None, StateNum::HeadRaise6), // HeadRaise5
    state(SpriteNum::Head, 6, 8, StateAction::None, StateNum::HeadRun1), // HeadRaise6
    state(SpriteNum::Bal7, 32768, 4, StateAction::None, StateNum::Brball2), // Brball1
    state(SpriteNum::Bal7, 32769, 4, StateAction::None, StateNum::Brball1), // Brball2
    state(SpriteNum::Bal7, 32770, 6, StateAction::None, StateNum::Brballx2), // Brballx1
    state(SpriteNum::Bal7, 32771, 6, StateAction::None, StateNum::Brballx3), // Brballx2
    state(SpriteNum::Bal7, 32772, 6, StateAction::None, StateNum::Null), // Brballx3
    state(SpriteNum::Boss, 0, 10, StateAction::Mobj(look), StateNum::BossStnd2), // BossStnd
    state(SpriteNum::Boss, 1, 10, StateAction::Mobj(look), StateNum::BossStnd), // BossStnd2
    state(SpriteNum::Boss, 0, 3, StateAction::Mobj(chase), StateNum::BossRun2), // BossRun1
    state(SpriteNum::Boss, 0, 3, StateAction::Mobj(chase), StateNum::BossRun3), // BossRun2
    state(SpriteNum::Boss, 1, 3, StateAction::Mobj(chase), StateNum::BossRun4), // BossRun3
    state(SpriteNum::Boss, 1, 3, StateAction::Mobj(chase), StateNum::BossRun5), // BossRun4
    state(SpriteNum::Boss, 2, 3, StateAction::Mobj(chase), StateNum::BossRun6), // BossRun5
    state(SpriteNum::Boss, 2, 3, StateAction::Mobj(chase), StateNum::BossRun7), // BossRun6
    state(SpriteNum::Boss, 3, 3, StateAction::Mobj(chase), StateNum::BossRun8), // BossRun7
    state(SpriteNum::Boss, 3, 3, StateAction::Mobj(chase), StateNum::BossRun1), // BossRun8
    state(SpriteNum::Boss, 4, 8, StateAction::Mobj(face_target), StateNum::BossAtk2), // BossAtk1
    state(SpriteNum::Boss, 5, 8, StateAction::Mobj(face_target), StateNum::BossAtk3), // BossAtk2
    state(SpriteNum::Boss, 6, 8, StateAction::Mobj(bruis_attack), StateNum::BossRun1), // BossAtk3
    state(SpriteNum::Boss, 7, 2, StateAction::None, StateNum::BossPain2), // BossPain
    state(SpriteNum::Boss, 7, 2, StateAction::Mobj(pain), StateNum::BossRun1), // BossPain2
    state(SpriteNum::Boss, 8, 8, StateAction::None, StateNum::BossDie2), // BossDie1
    state(SpriteNum::Boss, 9, 8, StateAction::Mobj(scream), StateNum::BossDie3), // BossDie2
    state(SpriteNum::Boss, 10, 8, StateAction::None, StateNum::BossDie4), // BossDie3
    state(SpriteNum::Boss, 11, 8, StateAction::Mobj(fall), StateNum::BossDie5), // BossDie4
    state(SpriteNum::Boss, 12, 8, StateAction::None, StateNum::BossDie6), // BossDie5
    state(SpriteNum::Boss, 13, 8, StateAction::None, StateNum::BossDie7), // BossDie6
    state(SpriteNum::Boss, 14, -1, StateAction::Mobj(boss_death), StateNum::Null), // BossDie7
    state(SpriteNum::Boss, 14, 8, StateAction::None, StateNum::BossRaise2), // BossRaise1
    state(SpriteNum::Boss, 13, 8, StateAction::None, StateNum::BossRaise3), // BossRaise2
    state(SpriteNum::Boss, 12, 8, StateAction::None, StateNum::BossRaise4), // BossRaise3
    state(SpriteNum::Boss, 11, 8, StateAction::None, StateNum::BossRaise5), // BossRaise4
    state(SpriteNum::Boss, 10, 8, StateAction::None, StateNum::BossRaise6), // BossRaise5
    state(SpriteNum::Boss, 9, 8, StateAction::None, StateNum::BossRaise7), // BossRaise6
    state(SpriteNum::Boss, 8, 8, StateAction::None, StateNum::BossRun1), // BossRaise7
    state(SpriteNum::Bos2, 0, 10, StateAction::Mobj(look), StateNum::Bos2Stnd2), // Bos2Stnd
    state(SpriteNum::Bos2, 1, 10, StateAction::Mobj(look), StateNum::Bos2Stnd), // Bos2Stnd2
    state(SpriteNum::Bos2, 0, 3, StateAction::Mobj(chase), StateNum::Bos2Run2), // Bos2Run1
    state(SpriteNum::Bos2, 0, 3, StateAction::Mobj(chase), StateNum::Bos2Run3), // Bos2Run2
    state(SpriteNum::Bos2, 1, 3, StateAction::Mobj(chase), StateNum::Bos2Run4), // Bos2Run3
    state(SpriteNum::Bos2, 1, 3, StateAction::Mobj(chase), StateNum::Bos2Run5), // Bos2Run4
    state(SpriteNum::Bos2, 2, 3, StateAction::Mobj(chase), StateNum::Bos2Run6), // Bos2Run5
    state(SpriteNum::Bos2, 2, 3, StateAction::Mobj(chase), StateNum::Bos2Run7), // Bos2Run6
    state(SpriteNum::Bos2, 3, 3, StateAction::Mobj(chase), StateNum::Bos2Run8), // Bos2Run7
    state(SpriteNum::Bos2, 3, 3, StateAction::Mobj(chase), StateNum::Bos2Run1), // Bos2Run8
    state(SpriteNum::Bos2, 4, 8, StateAction::Mobj(face_target), StateNum::Bos2Atk2), // Bos2Atk1
    state(SpriteNum::Bos2, 5, 8, StateAction::Mobj(face_target), StateNum::Bos2Atk3), // Bos2Atk2
    state(SpriteNum::Bos2, 6, 8, StateAction::Mobj(bruis_attack), StateNum::Bos2Run1), // Bos2Atk3
    state(SpriteNum::Bos2, 7, 2, StateAction::None, StateNum::Bos2Pain2), // Bos2Pain
    state(SpriteNum::Bos2, 7, 2, StateAction::Mobj(pain), StateNum::Bos2Run1), // Bos2Pain2
    state(SpriteNum::Bos2, 8, 8, StateAction::None, StateNum::Bos2Die2), // Bos2Die1
    state(SpriteNum::Bos2, 9, 8, StateAction::Mobj(scream), StateNum::Bos2Die3), // Bos2Die2
    state(SpriteNum::Bos2, 10, 8, StateAction::None, StateNum::Bos2Die4), // Bos2Die3
    state(SpriteNum::Bos2, 11, 8, StateAction::Mobj(fall), StateNum::Bos2Die5), // Bos2Die4
    state(SpriteNum::Bos2, 12, 8, StateAction::None, StateNum::Bos2Die6), // Bos2Die5
    state(SpriteNum::Bos2, 13, 8, StateAction::None, StateNum::Bos2Die7), // Bos2Die6
    state(SpriteNum::Bos2, 14, -1, StateAction::None, StateNum::Null), // Bos2Die7
    state(SpriteNum::Bos2, 14, 8, StateAction::None, StateNum::Bos2Raise2), // Bos2Raise1
    state(SpriteNum::Bos2, 13, 8, StateAction::None, StateNum::Bos2Raise3), // Bos2Raise2
    state(SpriteNum::Bos2, 12, 8, StateAction::None, StateNum::Bos2Raise4), // Bos2Raise3
    state(SpriteNum::Bos2, 11, 8, StateAction::None, StateNum::Bos2Raise5), // Bos2Raise4
    state(SpriteNum::Bos2, 10, 8, StateAction::None, StateNum::Bos2Raise6), // Bos2Raise5
    state(SpriteNum::Bos2, 9, 8, StateAction::None, StateNum::Bos2Raise7), // Bos2Raise6
    state(SpriteNum::Bos2, 8, 8, StateAction::None, StateNum::Bos2Run1), // Bos2Raise7
    state(SpriteNum::Skul, 32768, 10, StateAction::Mobj(look), StateNum::SkullStnd2), // SkullStnd
    state(SpriteNum::Skul, 32769, 10, StateAction::Mobj(look), StateNum::SkullStnd), // SkullStnd2
    state(SpriteNum::Skul, 32768, 6, StateAction::Mobj(chase), StateNum::SkullRun2), // SkullRun1
    state(SpriteNum::Skul, 32769, 6, StateAction::Mobj(chase), StateNum::SkullRun1), // SkullRun2
    state(SpriteNum::Skul, 32770, 10, StateAction::Mobj(face_target), StateNum::SkullAtk2), // SkullAtk1
    state(SpriteNum::Skul, 32771, 4, StateAction::Mobj(skull_attack), StateNum::SkullAtk3), // SkullAtk2
    state(SpriteNum::Skul, 32770, 4, StateAction::None, StateNum::SkullAtk4), // SkullAtk3
    state(SpriteNum::Skul, 32771, 4, StateAction::None, StateNum::SkullAtk3), // SkullAtk4
    state(SpriteNum::Skul, 32772, 3, StateAction::None, StateNum::SkullPain2), // SkullPain
    state(SpriteNum::Skul, 32772, 3, StateAction::Mobj(pain), StateNum::SkullRun1), // SkullPain2
    state(SpriteNum::Skul, 32773, 6, StateAction::None, StateNum::SkullDie2), // SkullDie1
    state(SpriteNum::Skul, 32774, 6, StateAction::Mobj(scream), StateNum::SkullDie3), // SkullDie2
    state(SpriteNum::Skul, 32775, 6, StateAction::None, StateNum::SkullDie4), // SkullDie3
    state(SpriteNum::Skul, 32776, 6, StateAction::Mobj(fall), StateNum::SkullDie5), // SkullDie4
    state(SpriteNum::Skul, 9, 6, StateAction::None, StateNum::SkullDie6), // SkullDie5
    state(SpriteNum::Skul, 10, 6, StateAction::None, StateNum::Null), // SkullDie6
    state(SpriteNum::Spid, 0, 10, StateAction::Mobj(look), StateNum::SpidStnd2), // SpidStnd
    state(SpriteNum::Spid, 1, 10, StateAction::Mobj(look), StateNum::SpidStnd), // SpidStnd2
    state(SpriteNum::Spid, 0, 3, StateAction::Mobj(metal), StateNum::SpidRun2), // SpidRun1
    state(SpriteNum::Spid, 0, 3, StateAction::Mobj(chase), StateNum::SpidRun3), // SpidRun2
    state(SpriteNum::Spid, 1, 3, StateAction::Mobj(chase), StateNum::SpidRun4), // SpidRun3
    state(SpriteNum::Spid, 1, 3, StateAction::Mobj(chase), StateNum::SpidRun5), // SpidRun4
    state(SpriteNum::Spid, 2, 3, StateAction::Mobj(metal), StateNum::SpidRun6), // SpidRun5
    state(SpriteNum::Spid, 2, 3, StateAction::Mobj(chase), StateNum::SpidRun7), // SpidRun6
    state(SpriteNum::Spid, 3, 3, StateAction::Mobj(chase), StateNum::SpidRun8), // SpidRun7
    state(SpriteNum::Spid, 3, 3, StateAction::Mobj(chase), StateNum::SpidRun9), // SpidRun8
    state(SpriteNum::Spid, 4, 3, StateAction::Mobj(metal), StateNum::SpidRun10), // SpidRun9
    state(SpriteNum::Spid, 4, 3, StateAction::Mobj(chase), StateNum::SpidRun11), // SpidRun10
    state(SpriteNum::Spid, 5, 3, StateAction::Mobj(chase), StateNum::SpidRun12), // SpidRun11
    state(SpriteNum::Spid, 5, 3, StateAction::Mobj(chase), StateNum::SpidRun1), // SpidRun12
    state(SpriteNum::Spid, 32768, 20, StateAction::Mobj(face_target), StateNum::SpidAtk2), // SpidAtk1
    state(SpriteNum::Spid, 32774, 4, StateAction::Mobj(spos_attack), StateNum::SpidAtk3), // SpidAtk2
    state(SpriteNum::Spid, 32775, 4, StateAction::Mobj(spos_attack), StateNum::SpidAtk4), // SpidAtk3
    state(SpriteNum::Spid, 32775, 1, StateAction::Mobj(spid_refire), StateNum::SpidAtk2), // SpidAtk4
    state(SpriteNum::Spid, 8, 3, StateAction::None, StateNum::SpidPain2), // SpidPain
    state(SpriteNum::Spid, 8, 3, StateAction::Mobj(pain), StateNum::SpidRun1), // SpidPain2
    state(SpriteNum::Spid, 9, 20, StateAction::Mobj(scream), StateNum::SpidDie2), // SpidDie1
    state(SpriteNum::Spid, 10, 10, StateAction::Mobj(fall), StateNum::SpidDie3), // SpidDie2
    state(SpriteNum::Spid, 11, 10, StateAction::None, StateNum::SpidDie4), // SpidDie3
    state(SpriteNum::Spid, 12, 10, StateAction::None, StateNum::SpidDie5), // SpidDie4
    state(SpriteNum::Spid, 13, 10, StateAction::None, StateNum::SpidDie6), // SpidDie5
    state(SpriteNum::Spid, 14, 10, StateAction::None, StateNum::SpidDie7), // SpidDie6
    state(SpriteNum::Spid, 15, 10, StateAction::None, StateNum::SpidDie8), // SpidDie7
    state(SpriteNum::Spid, 16, 10, StateAction::None, StateNum::SpidDie9), // SpidDie8
    state(SpriteNum::Spid, 17, 10, StateAction::None, StateNum::SpidDie10), // SpidDie9
    state(SpriteNum::Spid, 18, 30, StateAction::None, StateNum::SpidDie11), // SpidDie10
    state(SpriteNum::Spid, 18, -1, StateAction::Mobj(boss_death), StateNum::Null), // SpidDie11
    state(SpriteNum::Bspi, 0, 10, StateAction::Mobj(look), StateNum::BspiStnd2), // BspiStnd
    state(SpriteNum::Bspi, 1, 10, StateAction::Mobj(look), StateNum::BspiStnd), // BspiStnd2
    state(SpriteNum::Bspi, 0, 20, StateAction::None, StateNum::BspiRun1), // BspiSight
    state(SpriteNum::Bspi, 0, 3, StateAction::Mobj(baby_metal), StateNum::BspiRun2), // BspiRun1
    state(SpriteNum::Bspi, 0, 3, StateAction::Mobj(chase), StateNum::BspiRun3), // BspiRun2
    state(SpriteNum::Bspi, 1, 3, StateAction::Mobj(chase), StateNum::BspiRun4), // BspiRun3
    state(SpriteNum::Bspi, 1, 3, StateAction::Mobj(chase), StateNum::BspiRun5), // BspiRun4
    state(SpriteNum::Bspi, 2, 3, StateAction::Mobj(chase), StateNum::BspiRun6), // BspiRun5
    state(SpriteNum::Bspi, 2, 3, StateAction::Mobj(chase), StateNum::BspiRun7), // BspiRun6
    state(SpriteNum::Bspi, 3, 3, StateAction::Mobj(baby_metal), StateNum::BspiRun8), // BspiRun7
    state(SpriteNum::Bspi, 3, 3, StateAction::Mobj(chase), StateNum::BspiRun9), // BspiRun8
    state(SpriteNum::Bspi, 4, 3, StateAction::Mobj(chase), StateNum::BspiRun10), // BspiRun9
    state(SpriteNum::Bspi, 4, 3, StateAction::Mobj(chase), StateNum::BspiRun11), // BspiRun10
    state(SpriteNum::Bspi, 5, 3, StateAction::Mobj(chase), StateNum::BspiRun12), // BspiRun11
    state(SpriteNum::Bspi, 5, 3, StateAction::Mobj(chase), StateNum::BspiRun1), // BspiRun12
    state(SpriteNum::Bspi, 32768, 20, StateAction::Mobj(face_target), StateNum::BspiAtk2), // BspiAtk1
    state(SpriteNum::Bspi, 32774, 4, StateAction::Mobj(bspi_attack), StateNum::BspiAtk3), // BspiAtk2
    state(SpriteNum::Bspi, 32775, 4, StateAction::None, StateNum::BspiAtk4), // BspiAtk3
    state(SpriteNum::Bspi, 32775, 1, StateAction::Mobj(spid_refire), StateNum::BspiAtk2), // BspiAtk4
    state(SpriteNum::Bspi, 8, 3, StateAction::None, StateNum::BspiPain2), // BspiPain
    state(SpriteNum::Bspi, 8, 3, StateAction::Mobj(pain), StateNum::BspiRun1), // BspiPain2
    state(SpriteNum::Bspi, 9, 20, StateAction::Mobj(scream), StateNum::BspiDie2), // BspiDie1
    state(SpriteNum::Bspi, 10, 7, StateAction::Mobj(fall), StateNum::BspiDie3), // BspiDie2
    state(SpriteNum::Bspi, 11, 7, StateAction::None, StateNum::BspiDie4), // BspiDie3
    state(SpriteNum::Bspi, 12, 7, StateAction::None, StateNum::BspiDie5), // BspiDie4
    state(SpriteNum::Bspi, 13, 7, StateAction::None, StateNum::BspiDie6), // BspiDie5
    state(SpriteNum::Bspi, 14, 7, StateAction::None, StateNum::BspiDie7), // BspiDie6
    state(SpriteNum::Bspi, 15, -1, StateAction::Mobj(boss_death), StateNum::Null), // BspiDie7
    state(SpriteNum::Bspi, 15, 5, StateAction::None, StateNum::BspiRaise2), // BspiRaise1
    state(SpriteNum::Bspi, 14, 5, StateAction::None, StateNum::BspiRaise3), // BspiRaise2
    state(SpriteNum::Bspi, 13, 5, StateAction::None, StateNum::BspiRaise4), // BspiRaise3
    state(SpriteNum::Bspi, 12, 5, StateAction::None, StateNum::BspiRaise5), // BspiRaise4
    state(SpriteNum::Bspi, 11, 5, StateAction::None, StateNum::BspiRaise6), // BspiRaise5
    state(SpriteNum::Bspi, 10, 5, StateAction::None, StateNum::BspiRaise7), // BspiRaise6
    state(SpriteNum::Bspi, 9, 5, StateAction::None, StateNum::BspiRun1), // BspiRaise7
    state(SpriteNum::Apls, 32768, 5, StateAction::None, StateNum::ArachPlaz2), // ArachPlaz
    state(SpriteNum::Apls, 32769, 5, StateAction::None, StateNum::ArachPlaz), // ArachPlaz2
    state(SpriteNum::Apbx, 32768, 5, StateAction::None, StateNum::ArachPlex2), // ArachPlex
    state(SpriteNum::Apbx, 32769, 5, StateAction::None, StateNum::ArachPlex3), // ArachPlex2
    state(SpriteNum::Apbx, 32770, 5, StateAction::None, StateNum::ArachPlex4), // ArachPlex3
    state(SpriteNum::Apbx, 32771, 5, StateAction::None, StateNum::ArachPlex5), // ArachPlex4
    state(SpriteNum::Apbx, 32772, 5, StateAction::None, StateNum::Null), // ArachPlex5
    state(SpriteNum::Cybr, 0, 10, StateAction::Mobj(look), StateNum::CyberStnd2), // CyberStnd
    state(SpriteNum::Cybr, 1, 10, StateAction::Mobj(look), StateNum::CyberStnd), // CyberStnd2
    state(SpriteNum::Cybr, 0, 3, StateAction::Mobj(hoof), StateNum::CyberRun2), // CyberRun1
    state(SpriteNum::Cybr, 0, 3, StateAction::Mobj(chase), StateNum::CyberRun3), // CyberRun2
    state(SpriteNum::Cybr, 1, 3, StateAction::Mobj(chase), StateNum::CyberRun4), // CyberRun3
    state(SpriteNum::Cybr, 1, 3, StateAction::Mobj(chase), StateNum::CyberRun5), // CyberRun4
    state(SpriteNum::Cybr, 2, 3, StateAction::Mobj(chase), StateNum::CyberRun6), // CyberRun5
    state(SpriteNum::Cybr, 2, 3, StateAction::Mobj(chase), StateNum::CyberRun7), // CyberRun6
    state(SpriteNum::Cybr, 3, 3, StateAction::Mobj(metal), StateNum::CyberRun8), // CyberRun7
    state(SpriteNum::Cybr, 3, 3, StateAction::Mobj(chase), StateNum::CyberRun1), // CyberRun8
    state(SpriteNum::Cybr, 4, 6, StateAction::Mobj(face_target), StateNum::CyberAtk2), // CyberAtk1
    state(SpriteNum::Cybr, 5, 12, StateAction::Mobj(cyber_attack), StateNum::CyberAtk3), // CyberAtk2
    state(SpriteNum::Cybr, 4, 12, StateAction::Mobj(face_target), StateNum::CyberAtk4), // CyberAtk3
    state(SpriteNum::Cybr, 5, 12, StateAction::Mobj(cyber_attack), StateNum::CyberAtk5), // CyberAtk4
    state(SpriteNum::Cybr, 4, 12, StateAction::Mobj(face_target), StateNum::CyberAtk6), // CyberAtk5
    state(SpriteNum::Cybr, 5, 12, StateAction::Mobj(cyber_attack), StateNum::CyberRun1), // CyberAtk6
    state(SpriteNum::Cybr, 6, 10, StateAction::Mobj(pain), StateNum::CyberRun1), // CyberPain
    state(SpriteNum::Cybr, 7, 10, StateAction::None, StateNum::CyberDie2), // CyberDie1
    state(SpriteNum::Cybr, 8, 10, StateAction::Mobj(scream), StateNum::CyberDie3), // CyberDie2
    state(SpriteNum::Cybr, 9, 10, StateAction::None, StateNum::CyberDie4), // CyberDie3
    state(SpriteNum::Cybr, 10, 10, StateAction::None, StateNum::CyberDie5), // CyberDie4
    state(SpriteNum::Cybr, 11, 10, StateAction::None, StateNum::CyberDie6), // CyberDie5
    state(SpriteNum::Cybr, 12, 10, StateAction::Mobj(fall), StateNum::CyberDie7), // CyberDie6
    state(SpriteNum::Cybr, 13, 10, StateAction::None, StateNum::CyberDie8), // CyberDie7
    state(SpriteNum::Cybr, 14, 10, StateAction::None, StateNum::CyberDie9), // CyberDie8
    state(SpriteNum::Cybr, 15, 30, StateAction::None, StateNum::CyberDie10), // CyberDie9
    state(SpriteNum::Cybr, 15, -1, StateAction::Mobj(boss_death), StateNum::Null), // CyberDie10
    state(SpriteNum::Pain, 0, 10, StateAction::Mobj(look), StateNum::PainStnd), // PainStnd
    state(SpriteNum::Pain, 0, 3, StateAction::Mobj(chase), StateNum::PainRun2), // PainRun1
    state(SpriteNum::Pain, 0, 3, StateAction::Mobj(chase), StateNum::PainRun3), // PainRun2
    state(SpriteNum::Pain, 1, 3, StateAction::Mobj(chase), StateNum::PainRun4), // PainRun3
    state(SpriteNum::Pain, 1, 3, StateAction::Mobj(chase), StateNum::PainRun5), // PainRun4
    state(SpriteNum::Pain, 2, 3, StateAction::Mobj(chase), StateNum::PainRun6), // PainRun5
    state(SpriteNum::Pain, 2, 3, StateAction::Mobj(chase), StateNum::PainRun1), // PainRun6
    state(SpriteNum::Pain, 3, 5, StateAction::Mobj(face_target), StateNum::PainAtk2), // PainAtk1
    state(SpriteNum::Pain, 4, 5, StateAction::Mobj(face_target), StateNum::PainAtk3), // PainAtk2
    state(SpriteNum::Pain, 32773, 5, StateAction::Mobj(face_target), StateNum::PainAtk4), // PainAtk3
    state(SpriteNum::Pain, 32773, 0, StateAction::Mobj(pain_attack), StateNum::PainRun1), // PainAtk4
    state(SpriteNum::Pain, 6, 6, StateAction::None, StateNum::PainPain2), // PainPain
    state(SpriteNum::Pain, 6, 6, StateAction::Mobj(pain), StateNum::PainRun1), // PainPain2
    state(SpriteNum::Pain, 32775, 8, StateAction::None, StateNum::PainDie2), // PainDie1
    state(SpriteNum::Pain, 32776, 8, StateAction::Mobj(scream), StateNum::PainDie3), // PainDie2
    state(SpriteNum::Pain, 32777, 8, StateAction::None, StateNum::PainDie4), // PainDie3
    state(SpriteNum::Pain, 32778, 8, StateAction::None, StateNum::PainDie5), // PainDie4
    state(SpriteNum::Pain, 32779, 8, StateAction::Mobj(pain_die), StateNum::PainDie6), // PainDie5
    state(SpriteNum::Pain, 32780, 8, StateAction::None, StateNum::Null), // PainDie6
    state(SpriteNum::Pain, 12, 8, StateAction::None, StateNum::PainRaise2), // PainRaise1
    state(SpriteNum::Pain, 11, 8, StateAction::None, StateNum::PainRaise3), // PainRaise2
    state(SpriteNum::Pain, 10, 8, StateAction::None, StateNum::PainRaise4), // PainRaise3
    state(SpriteNum::Pain, 9, 8, StateAction::None, StateNum::PainRaise5), // PainRaise4
    state(SpriteNum::Pain, 8, 8, StateAction::None, StateNum::PainRaise6), // PainRaise5
    state(SpriteNum::Pain, 7, 8, StateAction::None, StateNum::PainRun1), // PainRaise6
    state(SpriteNum::Sswv, 0, 10, StateAction::Mobj(look), StateNum::SswvStnd2), // SswvStnd
    state(SpriteNum::Sswv, 1, 10, StateAction::Mobj(look), StateNum::SswvStnd), // SswvStnd2
    state(SpriteNum::Sswv, 0, 3, StateAction::Mobj(chase), StateNum::SswvRun2), // SswvRun1
    state(SpriteNum::Sswv, 0, 3, StateAction::Mobj(chase), StateNum::SswvRun3), // SswvRun2
    state(SpriteNum::Sswv, 1, 3, StateAction::Mobj(chase), StateNum::SswvRun4), // SswvRun3
    state(SpriteNum::Sswv, 1, 3, StateAction::Mobj(chase), StateNum::SswvRun5), // SswvRun4
    state(SpriteNum::Sswv, 2, 3, StateAction::Mobj(chase), StateNum::SswvRun6), // SswvRun5
    state(SpriteNum::Sswv, 2, 3, StateAction::Mobj(chase), StateNum::SswvRun7), // SswvRun6
    state(SpriteNum::Sswv, 3, 3, StateAction::Mobj(chase), StateNum::SswvRun8), // SswvRun7
    state(SpriteNum::Sswv, 3, 3, StateAction::Mobj(chase), StateNum::SswvRun1), // SswvRun8
    state(SpriteNum::Sswv, 4, 10, StateAction::Mobj(face_target), StateNum::SswvAtk2), // SswvAtk1
    state(SpriteNum::Sswv, 5, 10, StateAction::Mobj(face_target), StateNum::SswvAtk3), // SswvAtk2
    state(SpriteNum::Sswv, 32774, 4, StateAction::Mobj(cpos_attack), StateNum::SswvAtk4), // SswvAtk3
    state(SpriteNum::Sswv, 5, 6, StateAction::Mobj(face_target), StateNum::SswvAtk5), // SswvAtk4
    state(SpriteNum::Sswv, 32774, 4, StateAction::Mobj(cpos_attack), StateNum::SswvAtk6), // SswvAtk5
    state(SpriteNum::Sswv, 5, 1, StateAction::Mobj(cpos_refire), StateNum::SswvAtk2), // SswvAtk6
    state(SpriteNum::Sswv, 7, 3, StateAction::None, StateNum::SswvPain2), // SswvPain
    state(SpriteNum::Sswv, 7, 3, StateAction::Mobj(pain), StateNum::SswvRun1), // SswvPain2
    state(SpriteNum::Sswv, 8, 5, StateAction::None, StateNum::SswvDie2), // SswvDie1
    state(SpriteNum::Sswv, 9, 5, StateAction::Mobj(scream), StateNum::SswvDie3), // SswvDie2
    state(SpriteNum::Sswv, 10, 5, StateAction::Mobj(fall), StateNum::SswvDie4), // SswvDie3
    state(SpriteNum::Sswv, 11, 5, StateAction::None, StateNum::SswvDie5), // SswvDie4
    state(SpriteNum::Sswv, 12, -1, StateAction::None, StateNum::Null), // SswvDie5
    state(SpriteNum::Sswv, 13, 5, StateAction::None, StateNum::SswvXdie2), // SswvXdie1
    state(SpriteNum::Sswv, 14, 5, StateAction::Mobj(xscream), StateNum::SswvXdie3), // SswvXdie2
    state(SpriteNum::Sswv, 15, 5, StateAction::Mobj(fall), StateNum::SswvXdie4), // SswvXdie3
    state(SpriteNum::Sswv, 16, 5, StateAction::None, StateNum::SswvXdie5), // SswvXdie4
    state(SpriteNum::Sswv, 17, 5, StateAction::None, StateNum::SswvXdie6), // SswvXdie5
    state(SpriteNum::Sswv, 18, 5, StateAction::None, StateNum::SswvXdie7), // SswvXdie6
    state(SpriteNum::Sswv, 19, 5, StateAction::None, StateNum::SswvXdie8), // SswvXdie7
    state(SpriteNum::Sswv, 20, 5, StateAction::None, StateNum::SswvXdie9), // SswvXdie8
    state(SpriteNum::Sswv, 21, -1, StateAction::None, StateNum::Null), // SswvXdie9
    state(SpriteNum::Sswv, 12, 5, StateAction::None, StateNum::SswvRaise2), // SswvRaise1
    state(SpriteNum::Sswv, 11, 5, StateAction::None, StateNum::SswvRaise3), // SswvRaise2
    state(SpriteNum::Sswv, 10, 5, StateAction::None, StateNum::SswvRaise4), // SswvRaise3
    state(SpriteNum::Sswv, 9, 5, StateAction::None, StateNum::SswvRaise5), // SswvRaise4
    state(SpriteNum::Sswv, 8, 5, StateAction::None, StateNum::SswvRun1), // SswvRaise5
    state(SpriteNum::Keen, 0, -1, StateAction::None, StateNum::Keenstnd), // Keenstnd
    state(SpriteNum::Keen, 0, 6, StateAction::None, StateNum::Commkeen2), // Commkeen
    state(SpriteNum::Keen, 1, 6, StateAction::None, StateNum::Commkeen3), // Commkeen2
    state(SpriteNum::Keen, 2, 6, StateAction::Mobj(scream), StateNum::Commkeen4), // Commkeen3
    state(SpriteNum::Keen, 3, 6, StateAction::None, StateNum::Commkeen5), // Commkeen4
    state(SpriteNum::Keen, 4, 6, StateAction::None, StateNum::Commkeen6), // Commkeen5
    state(SpriteNum::Keen, 5, 6, StateAction::None, StateNum::Commkeen7), // Commkeen6
    state(SpriteNum::Keen, 6, 6, StateAction::None, StateNum::Commkeen8), // Commkeen7
    state(SpriteNum::Keen, 7, 6, StateAction::None, StateNum::Commkeen9), // Commkeen8
    state(SpriteNum::Keen, 8, 6, StateAction::None, StateNum::Commkeen10), // Commkeen9
    state(SpriteNum::Keen, 9, 6, StateAction::None, StateNum::Commkeen11), // Commkeen10
    state(SpriteNum::Keen, 10, 6, StateAction::Mobj(keen_die), StateNum::Commkeen12), // Commkeen11
    state(SpriteNum::Keen, 11, -1, StateAction::None, StateNum::Null), // Commkeen12
    state(SpriteNum::Keen, 12, 4, StateAction::None, StateNum::Keenpain2), // Keenpain
    state(SpriteNum::Keen, 12, 8, StateAction::Mobj(pain), StateNum::Keenstnd), // Keenpain2
    state(SpriteNum::Bbrn, 0, -1, StateAction::None, StateNum::Null), // Brain
    state(SpriteNum::Bbrn, 1, 36, StateAction::Mobj(brain_pain), StateNum::Brain), // BrainPain
    state(SpriteNum::Bbrn, 0, 100, StateAction::Mobj(brain_scream), StateNum::BrainDie2), // BrainDie1
    state(SpriteNum::Bbrn, 0, 10, StateAction::None, StateNum::BrainDie3), // BrainDie2
    state(SpriteNum::Bbrn, 0, 10, StateAction::None, StateNum::BrainDie4), // BrainDie3
    state(SpriteNum::Bbrn, 0, -1, StateAction::Mobj(brain_die), StateNum::Null), // BrainDie4
    state(SpriteNum::Sswv, 0, 10, StateAction::Mobj(look), StateNum::Braineye), // Braineye
    state(SpriteNum::Sswv, 0, 181, StateAction::Mobj(brain_awake), StateNum::Braineye1), // Braineyesee
    state(SpriteNum::Sswv, 0, 150, StateAction::Mobj(brain_spit), StateNum::Braineye1), // Braineye1
    state(SpriteNum::Bosf, 32768, 3, StateAction::Mobj(spawn_sound), StateNum::Spawn2), // Spawn1
    state(SpriteNum::Bosf, 32769, 3, StateAction::Mobj(spawn_fly), StateNum::Spawn3), // Spawn2
    state(SpriteNum::Bosf, 32770, 3, StateAction::Mobj(spawn_fly), StateNum::Spawn4), // Spawn3
    state(SpriteNum::Bosf, 32771, 3, StateAction::Mobj(spawn_fly), StateNum::Spawn1), // Spawn4
    state(SpriteNum::Fire, 32768, 4, StateAction::Mobj(a_fire), StateNum::Spawnfire2), // Spawnfire1
    state(SpriteNum::Fire, 32769, 4, StateAction::Mobj(a_fire), StateNum::Spawnfire3), // Spawnfire2
    state(SpriteNum::Fire, 32770, 4, StateAction::Mobj(a_fire), StateNum::Spawnfire4), // Spawnfire3
    state(SpriteNum::Fire, 32771, 4, StateAction::Mobj(a_fire), StateNum::Spawnfire5), // Spawnfire4
    state(SpriteNum::Fire, 32772, 4, StateAction::Mobj(a_fire), StateNum::Spawnfire6), // Spawnfire5
    state(SpriteNum::Fire, 32773, 4, StateAction::Mobj(a_fire), StateNum::Spawnfire7), // Spawnfire6
    state(SpriteNum::Fire, 32774, 4, StateAction::Mobj(a_fire), StateNum::Spawnfire8), // Spawnfire7
    state(SpriteNum::Fire, 32775, 4, StateAction::Mobj(a_fire), StateNum::Null), // Spawnfire8
    state(SpriteNum::Misl, 32769, 10, StateAction::None, StateNum::Brainexplode2), // Brainexplode1
    state(SpriteNum::Misl, 32770, 10, StateAction::None, StateNum::Brainexplode3), // Brainexplode2
    state(SpriteNum::Misl, 32771, 10, StateAction::Mobj(brain_explode), StateNum::Null), // Brainexplode3
    state(SpriteNum::Arm1, 0, 6, StateAction::None, StateNum::Arm1a), // Arm1
    state(SpriteNum::Arm1, 32769, 7, StateAction::None, StateNum::Arm1), // Arm1a
    state(SpriteNum::Arm2, 0, 6, StateAction::None, StateNum::Arm2a), // Arm2
    state(SpriteNum::Arm2, 32769, 6, StateAction::None, StateNum::Arm2), // Arm2a
    state(SpriteNum::Bar1, 0, 6, StateAction::None, StateNum::Bar2), // Bar1
    state(SpriteNum::Bar1, 1, 6, StateAction::None, StateNum::Bar1), // Bar2
    state(SpriteNum::Bexp, 32768, 5, StateAction::None, StateNum::Bexp2), // Bexp
    state(SpriteNum::Bexp, 32769, 5, StateAction::Mobj(scream), StateNum::Bexp3), // Bexp2
    state(SpriteNum::Bexp, 32770, 5, StateAction::None, StateNum::Bexp4), // Bexp3
    state(SpriteNum::Bexp, 32771, 10, StateAction::Mobj(explode), StateNum::Bexp5), // Bexp4
    state(SpriteNum::Bexp, 32772, 10, StateAction::None, StateNum::Null), // Bexp5
    state(SpriteNum::Fcan, 32768, 4, StateAction::None, StateNum::Bbar2), // Bbar1
    state(SpriteNum::Fcan, 32769, 4, StateAction::None, StateNum::Bbar3), // Bbar2
    state(SpriteNum::Fcan, 32770, 4, StateAction::None, StateNum::Bbar1), // Bbar3
    state(SpriteNum::Bon1, 0, 6, StateAction::None, StateNum::Bon1a), // Bon1
    state(SpriteNum::Bon1, 1, 6, StateAction::None, StateNum::Bon1b), // Bon1a
    state(SpriteNum::Bon1, 2, 6, StateAction::None, StateNum::Bon1c), // Bon1b
    state(SpriteNum::Bon1, 3, 6, StateAction::None, StateNum::Bon1d), // Bon1c
    state(SpriteNum::Bon1, 2, 6, StateAction::None, StateNum::Bon1e), // Bon1d
    state(SpriteNum::Bon1, 1, 6, StateAction::None, StateNum::Bon1), // Bon1e
    state(SpriteNum::Bon2, 0, 6, StateAction::None, StateNum::Bon2a), // Bon2
    state(SpriteNum::Bon2, 1, 6, StateAction::None, StateNum::Bon2b), // Bon2a
    state(SpriteNum::Bon2, 2, 6, StateAction::None, StateNum::Bon2c), // Bon2b
    state(SpriteNum::Bon2, 3, 6, StateAction::None, StateNum::Bon2d), // Bon2c
    state(SpriteNum::Bon2, 2, 6, StateAction::None, StateNum::Bon2e), // Bon2d
    state(SpriteNum::Bon2, 1, 6, StateAction::None, StateNum::Bon2), // Bon2e
    state(SpriteNum::Bkey, 0, 10, StateAction::None, StateNum::Bkey2), // Bkey
    state(SpriteNum::Bkey, 32769, 10, StateAction::None, StateNum::Bkey), // Bkey2
    state(SpriteNum::Rkey, 0, 10, StateAction::None, StateNum::Rkey2), // Rkey
    state(SpriteNum::Rkey, 32769, 10, StateAction::None, StateNum::Rkey), // Rkey2
    state(SpriteNum::Ykey, 0, 10, StateAction::None, StateNum::Ykey2), // Ykey
    state(SpriteNum::Ykey, 32769, 10, StateAction::None, StateNum::Ykey), // Ykey2
    state(SpriteNum::Bsku, 0, 10, StateAction::None, StateNum::Bskull2), // Bskull
    state(SpriteNum::Bsku, 32769, 10, StateAction::None, StateNum::Bskull), // Bskull2
    state(SpriteNum::Rsku, 0, 10, StateAction::None, StateNum::Rskull2), // Rskull
    state(SpriteNum::Rsku, 32769, 10, StateAction::None, StateNum::Rskull), // Rskull2
    state(SpriteNum::Ysku, 0, 10, StateAction::None, StateNum::Yskull2), // Yskull
    state(SpriteNum::Ysku, 32769, 10, StateAction::None, StateNum::Yskull), // Yskull2
    state(SpriteNum::Stim, 0, -1, StateAction::None, StateNum::Null), // Stim
    state(SpriteNum::Medi, 0, -1, StateAction::None, StateNum::Null), // Medi
    state(SpriteNum::Soul, 32768, 6, StateAction::None, StateNum::Soul2), // Soul
    state(SpriteNum::Soul, 32769, 6, StateAction::None, StateNum::Soul3), // Soul2
    state(SpriteNum::Soul, 32770, 6, StateAction::None, StateNum::Soul4), // Soul3
    state(SpriteNum::Soul, 32771, 6, StateAction::None, StateNum::Soul5), // Soul4
    state(SpriteNum::Soul, 32770, 6, StateAction::None, StateNum::Soul6), // Soul5
    state(SpriteNum::Soul, 32769, 6, StateAction::None, StateNum::Soul), // Soul6
    state(SpriteNum::Pinv, 32768, 6, StateAction::None, StateNum::Pinv2), // Pinv
    state(SpriteNum::Pinv, 32769, 6, StateAction::None, StateNum::Pinv3), // Pinv2
    state(SpriteNum::Pinv, 32770, 6, StateAction::None, StateNum::Pinv4), // Pinv3
    state(SpriteNum::Pinv, 32771, 6, StateAction::None, StateNum::Pinv), // Pinv4
    state(SpriteNum::Pstr, 32768, -1, StateAction::None, StateNum::Null), // Pstr
    state(SpriteNum::Pins, 32768, 6, StateAction::None, StateNum::Pins2), // Pins
    state(SpriteNum::Pins, 32769, 6, StateAction::None, StateNum::Pins3), // Pins2
    state(SpriteNum::Pins, 32770, 6, StateAction::None, StateNum::Pins4), // Pins3
    state(SpriteNum::Pins, 32771, 6, StateAction::None, StateNum::Pins), // Pins4
    state(SpriteNum::Mega, 32768, 6, StateAction::None, StateNum::Mega2), // Mega
    state(SpriteNum::Mega, 32769, 6, StateAction::None, StateNum::Mega3), // Mega2
    state(SpriteNum::Mega, 32770, 6, StateAction::None, StateNum::Mega4), // Mega3
    state(SpriteNum::Mega, 32771, 6, StateAction::None, StateNum::Mega), // Mega4
    state(SpriteNum::Suit, 32768, -1, StateAction::None, StateNum::Null), // Suit
    state(SpriteNum::Pmap, 32768, 6, StateAction::None, StateNum::Pmap2), // Pmap
    state(SpriteNum::Pmap, 32769, 6, StateAction::None, StateNum::Pmap3), // Pmap2
    state(SpriteNum::Pmap, 32770, 6, StateAction::None, StateNum::Pmap4), // Pmap3
    state(SpriteNum::Pmap, 32771, 6, StateAction::None, StateNum::Pmap5), // Pmap4
    state(SpriteNum::Pmap, 32770, 6, StateAction::None, StateNum::Pmap6), // Pmap5
    state(SpriteNum::Pmap, 32769, 6, StateAction::None, StateNum::Pmap), // Pmap6
    state(SpriteNum::Pvis, 32768, 6, StateAction::None, StateNum::Pvis2), // Pvis
    state(SpriteNum::Pvis, 1, 6, StateAction::None, StateNum::Pvis), // Pvis2
    state(SpriteNum::Clip, 0, -1, StateAction::None, StateNum::Null), // Clip
    state(SpriteNum::Ammo, 0, -1, StateAction::None, StateNum::Null), // Ammo
    state(SpriteNum::Rock, 0, -1, StateAction::None, StateNum::Null), // Rock
    state(SpriteNum::Brok, 0, -1, StateAction::None, StateNum::Null), // Brok
    state(SpriteNum::Cell, 0, -1, StateAction::None, StateNum::Null), // Cell
    state(SpriteNum::Celp, 0, -1, StateAction::None, StateNum::Null), // Celp
    state(SpriteNum::Shel, 0, -1, StateAction::None, StateNum::Null), // Shel
    state(SpriteNum::Sbox, 0, -1, StateAction::None, StateNum::Null), // Sbox
    state(SpriteNum::Bpak, 0, -1, StateAction::None, StateNum::Null), // Bpak
    state(SpriteNum::Bfug, 0, -1, StateAction::None, StateNum::Null), // Bfug
    state(SpriteNum::Mgun, 0, -1, StateAction::None, StateNum::Null), // Mgun
    state(SpriteNum::Csaw, 0, -1, StateAction::None, StateNum::Null), // Csaw
    state(SpriteNum::Laun, 0, -1, StateAction::None, StateNum::Null), // Laun
    state(SpriteNum::Plas, 0, -1, StateAction::None, StateNum::Null), // Plas
    state(SpriteNum::Shot, 0, -1, StateAction::None, StateNum::Null), // Shot
    state(SpriteNum::Sgn2, 0, -1, StateAction::None, StateNum::Null), // Shot2
    state(SpriteNum::Colu, 32768, -1, StateAction::None, StateNum::Null), // Colu
    state(SpriteNum::Smt2, 0, -1, StateAction::None, StateNum::Null), // Stalag
    state(SpriteNum::Gor1, 0, 10, StateAction::None, StateNum::Bloodytwitch2), // Bloodytwitch
    state(SpriteNum::Gor1, 1, 15, StateAction::None, StateNum::Bloodytwitch3), // Bloodytwitch2
    state(SpriteNum::Gor1, 2, 8, StateAction::None, StateNum::Bloodytwitch4), // Bloodytwitch3
    state(SpriteNum::Gor1, 1, 6, StateAction::None, StateNum::Bloodytwitch), // Bloodytwitch4
    state(SpriteNum::Play, 13, -1, StateAction::None, StateNum::Null), // Deadtorso
    state(SpriteNum::Play, 18, -1, StateAction::None, StateNum::Null), // Deadbottom
    state(SpriteNum::Pol2, 0, -1, StateAction::None, StateNum::Null), // Headsonstick
    state(SpriteNum::Pol5, 0, -1, StateAction::None, StateNum::Null), // Gibs
    state(SpriteNum::Pol4, 0, -1, StateAction::None, StateNum::Null), // Headonastick
    state(SpriteNum::Pol3, 32768, 6, StateAction::None, StateNum::Headcandles2), // Headcandles
    state(SpriteNum::Pol3, 32769, 6, StateAction::None, StateNum::Headcandles), // Headcandles2
    state(SpriteNum::Pol1, 0, -1, StateAction::None, StateNum::Null), // Deadstick
    state(SpriteNum::Pol6, 0, 6, StateAction::None, StateNum::Livestick2), // Livestick
    state(SpriteNum::Pol6, 1, 8, StateAction::None, StateNum::Livestick), // Livestick2
    state(SpriteNum::Gor2, 0, -1, StateAction::None, StateNum::Null), // Meat2
    state(SpriteNum::Gor3, 0, -1, StateAction::None, StateNum::Null), // Meat3
    state(SpriteNum::Gor4, 0, -1, StateAction::None, StateNum::Null), // Meat4
    state(SpriteNum::Gor5, 0, -1, StateAction::None, StateNum::Null), // Meat5
    state(SpriteNum::Smit, 0, -1, StateAction::None, StateNum::Null), // Stalagtite
    state(SpriteNum::Col1, 0, -1, StateAction::None, StateNum::Null), // Tallgrncol
    state(SpriteNum::Col2, 0, -1, StateAction::None, StateNum::Null), // Shrtgrncol
    state(SpriteNum::Col3, 0, -1, StateAction::None, StateNum::Null), // Tallredcol
    state(SpriteNum::Col4, 0, -1, StateAction::None, StateNum::Null), // Shrtredcol
    state(SpriteNum::Cand, 32768, -1, StateAction::None, StateNum::Null), // Candlestik
    state(SpriteNum::Cbra, 32768, -1, StateAction::None, StateNum::Null), // Candelabra
    state(SpriteNum::Col6, 0, -1, StateAction::None, StateNum::Null), // Skullcol
    state(SpriteNum::Tre1, 0, -1, StateAction::None, StateNum::Null), // Torchtree
    state(SpriteNum::Tre2, 0, -1, StateAction::None, StateNum::Null), // Bigtree
    state(SpriteNum::Elec, 0, -1, StateAction::None, StateNum::Null), // Techpillar
    state(SpriteNum::Ceye, 32768, 6, StateAction::None, StateNum::Evileye2), // Evileye
    state(SpriteNum::Ceye, 32769, 6, StateAction::None, StateNum::Evileye3), // Evileye2
    state(SpriteNum::Ceye, 32770, 6, StateAction::None, StateNum::Evileye4), // Evileye3
    state(SpriteNum::Ceye, 32769, 6, StateAction::None, StateNum::Evileye), // Evileye4
    state(SpriteNum::Fsku, 32768, 6, StateAction::None, StateNum::Floatskull2), // Floatskull
    state(SpriteNum::Fsku, 32769, 6, StateAction::None, StateNum::Floatskull3), // Floatskull2
    state(SpriteNum::Fsku, 32770, 6, StateAction::None, StateNum::Floatskull), // Floatskull3
    state(SpriteNum::Col5, 0, 14, StateAction::None, StateNum::Heartcol2), // Heartcol
    state(SpriteNum::Col5, 1, 14, StateAction::None, StateNum::Heartcol), // Heartcol2
    state(SpriteNum::Tblu, 32768, 4, StateAction::None, StateNum::Bluetorch2), // Bluetorch
    state(SpriteNum::Tblu, 32769, 4, StateAction::None, StateNum::Bluetorch3), // Bluetorch2
    state(SpriteNum::Tblu, 32770, 4, StateAction::None, StateNum::Bluetorch4), // Bluetorch3
    state(SpriteNum::Tblu, 32771, 4, StateAction::None, StateNum::Bluetorch), // Bluetorch4
    state(SpriteNum::Tgrn, 32768, 4, StateAction::None, StateNum::Greentorch2), // Greentorch
    state(SpriteNum::Tgrn, 32769, 4, StateAction::None, StateNum::Greentorch3), // Greentorch2
    state(SpriteNum::Tgrn, 32770, 4, StateAction::None, StateNum::Greentorch4), // Greentorch3
    state(SpriteNum::Tgrn, 32771, 4, StateAction::None, StateNum::Greentorch), // Greentorch4
    state(SpriteNum::Tred, 32768, 4, StateAction::None, StateNum::Redtorch2), // Redtorch
    state(SpriteNum::Tred, 32769, 4, StateAction::None, StateNum::Redtorch3), // Redtorch2
    state(SpriteNum::Tred, 32770, 4, StateAction::None, StateNum::Redtorch4), // Redtorch3
    state(SpriteNum::Tred, 32771, 4, StateAction::None, StateNum::Redtorch), // Redtorch4
    state(SpriteNum::Smbt, 32768, 4, StateAction::None, StateNum::Btorchshrt2), // Btorchshrt
    state(SpriteNum::Smbt, 32769, 4, StateAction::None, StateNum::Btorchshrt3), // Btorchshrt2
    state(SpriteNum::Smbt, 32770, 4, StateAction::None, StateNum::Btorchshrt4), // Btorchshrt3
    state(SpriteNum::Smbt, 32771, 4, StateAction::None, StateNum::Btorchshrt), // Btorchshrt4
    state(SpriteNum::Smgt, 32768, 4, StateAction::None, StateNum::Gtorchshrt2), // Gtorchshrt
    state(SpriteNum::Smgt, 32769, 4, StateAction::None, StateNum::Gtorchshrt3), // Gtorchshrt2
    state(SpriteNum::Smgt, 32770, 4, StateAction::None, StateNum::Gtorchshrt4), // Gtorchshrt3
    state(SpriteNum::Smgt, 32771, 4, StateAction::None, StateNum::Gtorchshrt), // Gtorchshrt4
    state(SpriteNum::Smrt, 32768, 4, StateAction::None, StateNum::Rtorchshrt2), // Rtorchshrt
    state(SpriteNum::Smrt, 32769, 4, StateAction::None, StateNum::Rtorchshrt3), // Rtorchshrt2
    state(SpriteNum::Smrt, 32770, 4, StateAction::None, StateNum::Rtorchshrt4), // Rtorchshrt3
    state(SpriteNum::Smrt, 32771, 4, StateAction::None, StateNum::Rtorchshrt), // Rtorchshrt4
    state(SpriteNum::Hdb1, 0, -1, StateAction::None, StateNum::Null), // Hangnoguts
    state(SpriteNum::Hdb2, 0, -1, StateAction::None, StateNum::Null), // Hangbnobrain
    state(SpriteNum::Hdb3, 0, -1, StateAction::None, StateNum::Null), // Hangtlookdn
    state(SpriteNum::Hdb4, 0, -1, StateAction::None, StateNum::Null), // Hangtskull
    state(SpriteNum::Hdb5, 0, -1, StateAction::None, StateNum::Null), // Hangtlookup
    state(SpriteNum::Hdb6, 0, -1, StateAction::None, StateNum::Null), // Hangtnobrain
    state(SpriteNum::Pob1, 0, -1, StateAction::None, StateNum::Null), // Colongibs
    state(SpriteNum::Pob2, 0, -1, StateAction::None, StateNum::Null), // Smallpool
    state(SpriteNum::Brs1, 0, -1, StateAction::None, StateNum::Null), // Brainstem
    state(SpriteNum::Tlmp, 32768, 4, StateAction::None, StateNum::Techlamp2), // Techlamp
    state(SpriteNum::Tlmp, 32769, 4, StateAction::None, StateNum::Techlamp3), // Techlamp2
    state(SpriteNum::Tlmp, 32770, 4, StateAction::None, StateNum::Techlamp4), // Techlamp3
    state(SpriteNum::Tlmp, 32771, 4, StateAction::None, StateNum::Techlamp), // Techlamp4
    state(SpriteNum::Tlp2, 32768, 4, StateAction::None, StateNum::Tech2lamp2), // Tech2lamp
    state(SpriteNum::Tlp2, 32769, 4, StateAction::None, StateNum::Tech2lamp3), // Tech2lamp2
    state(SpriteNum::Tlp2, 32770, 4, StateAction::None, StateNum::Tech2lamp4), // Tech2lamp3
    state(SpriteNum::Tlp2, 32771, 4, StateAction::None, StateNum::Tech2lamp), // Tech2lamp4
];

impl MobjInfo {
    /// A thing with no states, sounds or flags. The entries of the table below spell out only the
    /// fields that differ from this.
    const DEFAULT: Self = Self {
        doomednum: -1,
        spawnstate: StateNum::Null,
        spawnhealth: 1000,
        seestate: StateNum::Null,
        seesound: SfxName::SfxNone,
        reactiontime: 8,
        attacksound: SfxName::SfxNone,
        painstate: StateNum::Null,
        painchance: 0,
        painsound: SfxName::SfxNone,
        meleestate: StateNum::Null,
        missilestate: StateNum::Null,
        deathstate: StateNum::Null,
        xdeathstate: StateNum::Null,
        deathsound: SfxName::SfxNone,
        speed: 0,
        radius: Fixed::from_int(20),
        height: Fixed::from_int(16),
        mass: 100,
        damage: 0,
        activesound: SfxName::SfxNone,
        flags: MobjFlags::empty(),
        raisestate: StateNum::Null,
    };
}

pub struct InfoState {
    pub sprnames: [&'static str; 138],
    pub states: [State; 967],
    pub mobjinfo: [MobjInfo; 137],
}

impl Default for InfoState {
    fn default() -> Self {
        Self::new()
    }
}

impl InfoState {
    pub fn state_mut(&mut self, id: StateId) -> &mut State {
        &mut self.states[id.0 as usize]
    }

    pub fn mobjinfo_mut(&mut self, t: MobjType) -> &mut MobjInfo {
        &mut self.mobjinfo[t as usize]
    }

    // Kept out of line: `GameState::new` inlines every state constructor, and once
    // `init_game_state` passes 256 KB the Xtensa linker fails ("dangerous relocation:
    // l32r: literal target out of range") building the firmware. These are the biggest.
    #[inline(never)]
    pub fn new() -> Self {
        Self {
            sprnames: [
                "TROO", "SHTG", "PUNG", "PISG", "PISF", "SHTF", "SHT2", "CHGG", "CHGF", "MISG",
                "MISF", "SAWG", "PLSG", "PLSF", "BFGG", "BFGF", "BLUD", "PUFF", "BAL1", "BAL2",
                "PLSS", "PLSE", "MISL", "BFS1", "BFE1", "BFE2", "TFOG", "IFOG", "PLAY", "POSS",
                "SPOS", "VILE", "FIRE", "FATB", "FBXP", "SKEL", "MANF", "FATT", "CPOS", "SARG",
                "HEAD", "BAL7", "BOSS", "BOS2", "SKUL", "SPID", "BSPI", "APLS", "APBX", "CYBR",
                "PAIN", "SSWV", "KEEN", "BBRN", "BOSF", "ARM1", "ARM2", "BAR1", "BEXP", "FCAN",
                "BON1", "BON2", "BKEY", "RKEY", "YKEY", "BSKU", "RSKU", "YSKU", "STIM", "MEDI",
                "SOUL", "PINV", "PSTR", "PINS", "MEGA", "SUIT", "PMAP", "PVIS", "CLIP", "AMMO",
                "ROCK", "BROK", "CELL", "CELP", "SHEL", "SBOX", "BPAK", "BFUG", "MGUN", "CSAW",
                "LAUN", "PLAS", "SHOT", "SGN2", "COLU", "SMT2", "GOR1", "POL2", "POL5", "POL4",
                "POL3", "POL1", "POL6", "GOR2", "GOR3", "GOR4", "GOR5", "SMIT", "COL1", "COL2",
                "COL3", "COL4", "CAND", "CBRA", "COL6", "TRE1", "TRE2", "ELEC", "CEYE", "FSKU",
                "COL5", "TBLU", "TGRN", "TRED", "SMBT", "SMGT", "SMRT", "HDB1", "HDB2", "HDB3",
                "HDB4", "HDB5", "HDB6", "POB1", "POB2", "BRS1", "TLMP", "TLP2",
            ],
            states: STATES,
            mobjinfo: [
                MobjInfo {
                    spawnstate: StateNum::Play,
                    spawnhealth: 100,
                    seestate: StateNum::PlayRun1,
                    reactiontime: 0,
                    painstate: StateNum::PlayPain,
                    painchance: 255,
                    painsound: SfxName::Plpain,
                    missilestate: StateNum::PlayAtk1,
                    deathstate: StateNum::PlayDie1,
                    xdeathstate: StateNum::PlayXdie1,
                    deathsound: SfxName::Pldeth,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(56),
                    flags: MobjFlags::SOLID
                        | MobjFlags::SHOOTABLE
                        | MobjFlags::DROPOFF
                        | MobjFlags::PICKUP
                        | MobjFlags::NOTDMATCH,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 3004,
                    spawnstate: StateNum::PossStnd,
                    spawnhealth: 20,
                    seestate: StateNum::PossRun1,
                    seesound: SfxName::Posit1,
                    attacksound: SfxName::Pistol,
                    painstate: StateNum::PossPain,
                    painchance: 200,
                    painsound: SfxName::Popain,
                    missilestate: StateNum::PossAtk1,
                    deathstate: StateNum::PossDie1,
                    xdeathstate: StateNum::PossXdie1,
                    deathsound: SfxName::Podth1,
                    speed: 8,
                    height: Fixed::from_int(56),
                    activesound: SfxName::Posact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    raisestate: StateNum::PossRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 9,
                    spawnstate: StateNum::SposStnd,
                    spawnhealth: 30,
                    seestate: StateNum::SposRun1,
                    seesound: SfxName::Posit2,
                    painstate: StateNum::SposPain,
                    painchance: 170,
                    painsound: SfxName::Popain,
                    missilestate: StateNum::SposAtk1,
                    deathstate: StateNum::SposDie1,
                    xdeathstate: StateNum::SposXdie1,
                    deathsound: SfxName::Podth2,
                    speed: 8,
                    height: Fixed::from_int(56),
                    activesound: SfxName::Posact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    raisestate: StateNum::SposRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 64,
                    spawnstate: StateNum::VileStnd,
                    spawnhealth: 700,
                    seestate: StateNum::VileRun1,
                    seesound: SfxName::Vilsit,
                    painstate: StateNum::VilePain,
                    painchance: 10,
                    painsound: SfxName::Vipain,
                    missilestate: StateNum::VileAtk1,
                    deathstate: StateNum::VileDie1,
                    deathsound: SfxName::Vildth,
                    speed: 15,
                    height: Fixed::from_int(56),
                    mass: 500,
                    activesound: SfxName::Vilact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Fire1,
                    flags: MobjFlags::NOBLOCKMAP | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 66,
                    spawnstate: StateNum::SkelStnd,
                    spawnhealth: 300,
                    seestate: StateNum::SkelRun1,
                    seesound: SfxName::Skesit,
                    painstate: StateNum::SkelPain,
                    painchance: 100,
                    painsound: SfxName::Popain,
                    meleestate: StateNum::SkelFist1,
                    missilestate: StateNum::SkelMiss1,
                    deathstate: StateNum::SkelDie1,
                    deathsound: SfxName::Skedth,
                    speed: 10,
                    height: Fixed::from_int(56),
                    mass: 500,
                    activesound: SfxName::Skeact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    raisestate: StateNum::SkelRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Tracer,
                    seesound: SfxName::Skeatk,
                    deathstate: StateNum::Traceexp1,
                    deathsound: SfxName::Barexp,
                    speed: 10 * FRACUNIT.to_bits(),
                    radius: Fixed::from_int(11),
                    height: Fixed::from_int(8),
                    damage: 10,
                    flags: MobjFlags::NOBLOCKMAP
                        | MobjFlags::MISSILE
                        | MobjFlags::DROPOFF
                        | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Smoke1,
                    flags: MobjFlags::NOBLOCKMAP | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 67,
                    spawnstate: StateNum::FattStnd,
                    spawnhealth: 600,
                    seestate: StateNum::FattRun1,
                    seesound: SfxName::Mansit,
                    painstate: StateNum::FattPain,
                    painchance: 80,
                    painsound: SfxName::Mnpain,
                    missilestate: StateNum::FattAtk1,
                    deathstate: StateNum::FattDie1,
                    deathsound: SfxName::Mandth,
                    speed: 8,
                    radius: Fixed::from_int(48),
                    height: Fixed::from_int(64),
                    mass: 1000,
                    activesound: SfxName::Posact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    raisestate: StateNum::FattRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Fatshot1,
                    seesound: SfxName::Firsht,
                    deathstate: StateNum::Fatshotx1,
                    deathsound: SfxName::Firxpl,
                    speed: 20 * FRACUNIT.to_bits(),
                    radius: Fixed::from_int(6),
                    height: Fixed::from_int(8),
                    damage: 8,
                    flags: MobjFlags::NOBLOCKMAP
                        | MobjFlags::MISSILE
                        | MobjFlags::DROPOFF
                        | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 65,
                    spawnstate: StateNum::CposStnd,
                    spawnhealth: 70,
                    seestate: StateNum::CposRun1,
                    seesound: SfxName::Posit2,
                    painstate: StateNum::CposPain,
                    painchance: 170,
                    painsound: SfxName::Popain,
                    missilestate: StateNum::CposAtk1,
                    deathstate: StateNum::CposDie1,
                    xdeathstate: StateNum::CposXdie1,
                    deathsound: SfxName::Podth2,
                    speed: 8,
                    height: Fixed::from_int(56),
                    activesound: SfxName::Posact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    raisestate: StateNum::CposRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 3001,
                    spawnstate: StateNum::TrooStnd,
                    spawnhealth: 60,
                    seestate: StateNum::TrooRun1,
                    seesound: SfxName::Bgsit1,
                    painstate: StateNum::TrooPain,
                    painchance: 200,
                    painsound: SfxName::Popain,
                    meleestate: StateNum::TrooAtk1,
                    missilestate: StateNum::TrooAtk1,
                    deathstate: StateNum::TrooDie1,
                    xdeathstate: StateNum::TrooXdie1,
                    deathsound: SfxName::Bgdth1,
                    speed: 8,
                    height: Fixed::from_int(56),
                    activesound: SfxName::Bgact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    raisestate: StateNum::TrooRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 3002,
                    spawnstate: StateNum::SargStnd,
                    spawnhealth: 150,
                    seestate: StateNum::SargRun1,
                    seesound: SfxName::Sgtsit,
                    attacksound: SfxName::Sgtatk,
                    painstate: StateNum::SargPain,
                    painchance: 180,
                    painsound: SfxName::Dmpain,
                    meleestate: StateNum::SargAtk1,
                    deathstate: StateNum::SargDie1,
                    deathsound: SfxName::Sgtdth,
                    speed: 10,
                    radius: Fixed::from_int(30),
                    height: Fixed::from_int(56),
                    mass: 400,
                    activesound: SfxName::Dmact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    raisestate: StateNum::SargRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 58,
                    spawnstate: StateNum::SargStnd,
                    spawnhealth: 150,
                    seestate: StateNum::SargRun1,
                    seesound: SfxName::Sgtsit,
                    attacksound: SfxName::Sgtatk,
                    painstate: StateNum::SargPain,
                    painchance: 180,
                    painsound: SfxName::Dmpain,
                    meleestate: StateNum::SargAtk1,
                    deathstate: StateNum::SargDie1,
                    deathsound: SfxName::Sgtdth,
                    speed: 10,
                    radius: Fixed::from_int(30),
                    height: Fixed::from_int(56),
                    mass: 400,
                    activesound: SfxName::Dmact,
                    flags: MobjFlags::SOLID
                        | MobjFlags::SHOOTABLE
                        | MobjFlags::SHADOW
                        | MobjFlags::COUNTKILL,
                    raisestate: StateNum::SargRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 3005,
                    spawnstate: StateNum::HeadStnd,
                    spawnhealth: 400,
                    seestate: StateNum::HeadRun1,
                    seesound: SfxName::Cacsit,
                    painstate: StateNum::HeadPain,
                    painchance: 128,
                    painsound: SfxName::Dmpain,
                    missilestate: StateNum::HeadAtk1,
                    deathstate: StateNum::HeadDie1,
                    deathsound: SfxName::Cacdth,
                    speed: 8,
                    radius: Fixed::from_int(31),
                    height: Fixed::from_int(56),
                    mass: 400,
                    activesound: SfxName::Dmact,
                    flags: MobjFlags::SOLID
                        | MobjFlags::SHOOTABLE
                        | MobjFlags::FLOAT
                        | MobjFlags::NOGRAVITY
                        | MobjFlags::COUNTKILL,
                    raisestate: StateNum::HeadRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 3003,
                    spawnstate: StateNum::BossStnd,
                    seestate: StateNum::BossRun1,
                    seesound: SfxName::Brssit,
                    painstate: StateNum::BossPain,
                    painchance: 50,
                    painsound: SfxName::Dmpain,
                    meleestate: StateNum::BossAtk1,
                    missilestate: StateNum::BossAtk1,
                    deathstate: StateNum::BossDie1,
                    deathsound: SfxName::Brsdth,
                    speed: 8,
                    radius: Fixed::from_int(24),
                    height: Fixed::from_int(64),
                    mass: 1000,
                    activesound: SfxName::Dmact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    raisestate: StateNum::BossRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Brball1,
                    seesound: SfxName::Firsht,
                    deathstate: StateNum::Brballx1,
                    deathsound: SfxName::Firxpl,
                    speed: 15 * FRACUNIT.to_bits(),
                    radius: Fixed::from_int(6),
                    height: Fixed::from_int(8),
                    damage: 8,
                    flags: MobjFlags::NOBLOCKMAP
                        | MobjFlags::MISSILE
                        | MobjFlags::DROPOFF
                        | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 69,
                    spawnstate: StateNum::Bos2Stnd,
                    spawnhealth: 500,
                    seestate: StateNum::Bos2Run1,
                    seesound: SfxName::Kntsit,
                    painstate: StateNum::Bos2Pain,
                    painchance: 50,
                    painsound: SfxName::Dmpain,
                    meleestate: StateNum::Bos2Atk1,
                    missilestate: StateNum::Bos2Atk1,
                    deathstate: StateNum::Bos2Die1,
                    deathsound: SfxName::Kntdth,
                    speed: 8,
                    radius: Fixed::from_int(24),
                    height: Fixed::from_int(64),
                    mass: 1000,
                    activesound: SfxName::Dmact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    raisestate: StateNum::Bos2Raise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 3006,
                    spawnstate: StateNum::SkullStnd,
                    spawnhealth: 100,
                    seestate: StateNum::SkullRun1,
                    attacksound: SfxName::Sklatk,
                    painstate: StateNum::SkullPain,
                    painchance: 256,
                    painsound: SfxName::Dmpain,
                    missilestate: StateNum::SkullAtk1,
                    deathstate: StateNum::SkullDie1,
                    deathsound: SfxName::Firxpl,
                    speed: 8,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(56),
                    mass: 50,
                    damage: 3,
                    activesound: SfxName::Dmact,
                    flags: MobjFlags::SOLID
                        | MobjFlags::SHOOTABLE
                        | MobjFlags::FLOAT
                        | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 7,
                    spawnstate: StateNum::SpidStnd,
                    spawnhealth: 3000,
                    seestate: StateNum::SpidRun1,
                    seesound: SfxName::Spisit,
                    attacksound: SfxName::Shotgn,
                    painstate: StateNum::SpidPain,
                    painchance: 40,
                    painsound: SfxName::Dmpain,
                    missilestate: StateNum::SpidAtk1,
                    deathstate: StateNum::SpidDie1,
                    deathsound: SfxName::Spidth,
                    speed: 12,
                    radius: Fixed::from_int(128),
                    height: Fixed::from_int(100),
                    mass: 1000,
                    activesound: SfxName::Dmact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 68,
                    spawnstate: StateNum::BspiStnd,
                    spawnhealth: 500,
                    seestate: StateNum::BspiSight,
                    seesound: SfxName::Bspsit,
                    painstate: StateNum::BspiPain,
                    painchance: 128,
                    painsound: SfxName::Dmpain,
                    missilestate: StateNum::BspiAtk1,
                    deathstate: StateNum::BspiDie1,
                    deathsound: SfxName::Bspdth,
                    speed: 12,
                    radius: Fixed::from_int(64),
                    height: Fixed::from_int(64),
                    mass: 600,
                    activesound: SfxName::Bspact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    raisestate: StateNum::BspiRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 16,
                    spawnstate: StateNum::CyberStnd,
                    spawnhealth: 4000,
                    seestate: StateNum::CyberRun1,
                    seesound: SfxName::Cybsit,
                    painstate: StateNum::CyberPain,
                    painchance: 20,
                    painsound: SfxName::Dmpain,
                    missilestate: StateNum::CyberAtk1,
                    deathstate: StateNum::CyberDie1,
                    deathsound: SfxName::Cybdth,
                    speed: 16,
                    radius: Fixed::from_int(40),
                    height: Fixed::from_int(110),
                    mass: 1000,
                    activesound: SfxName::Dmact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 71,
                    spawnstate: StateNum::PainStnd,
                    spawnhealth: 400,
                    seestate: StateNum::PainRun1,
                    seesound: SfxName::Pesit,
                    painstate: StateNum::PainPain,
                    painchance: 128,
                    painsound: SfxName::Pepain,
                    missilestate: StateNum::PainAtk1,
                    deathstate: StateNum::PainDie1,
                    deathsound: SfxName::Pedth,
                    speed: 8,
                    radius: Fixed::from_int(31),
                    height: Fixed::from_int(56),
                    mass: 400,
                    activesound: SfxName::Dmact,
                    flags: MobjFlags::SOLID
                        | MobjFlags::SHOOTABLE
                        | MobjFlags::FLOAT
                        | MobjFlags::NOGRAVITY
                        | MobjFlags::COUNTKILL,
                    raisestate: StateNum::PainRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 84,
                    spawnstate: StateNum::SswvStnd,
                    spawnhealth: 50,
                    seestate: StateNum::SswvRun1,
                    seesound: SfxName::Sssit,
                    painstate: StateNum::SswvPain,
                    painchance: 170,
                    painsound: SfxName::Popain,
                    missilestate: StateNum::SswvAtk1,
                    deathstate: StateNum::SswvDie1,
                    xdeathstate: StateNum::SswvXdie1,
                    deathsound: SfxName::Ssdth,
                    speed: 8,
                    height: Fixed::from_int(56),
                    activesound: SfxName::Posact,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::COUNTKILL,
                    raisestate: StateNum::SswvRaise1,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 72,
                    spawnstate: StateNum::Keenstnd,
                    spawnhealth: 100,
                    painstate: StateNum::Keenpain,
                    painchance: 256,
                    painsound: SfxName::Keenpn,
                    deathstate: StateNum::Commkeen,
                    deathsound: SfxName::Keendt,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(72),
                    mass: 10000000,
                    flags: MobjFlags::SOLID
                        | MobjFlags::SPAWNCEILING
                        | MobjFlags::NOGRAVITY
                        | MobjFlags::SHOOTABLE
                        | MobjFlags::COUNTKILL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 88,
                    spawnstate: StateNum::Brain,
                    spawnhealth: 250,
                    painstate: StateNum::BrainPain,
                    painchance: 255,
                    painsound: SfxName::Bospn,
                    deathstate: StateNum::BrainDie1,
                    deathsound: SfxName::Bosdth,
                    radius: Fixed::from_int(16),
                    mass: 10000000,
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 89,
                    spawnstate: StateNum::Braineye,
                    seestate: StateNum::Braineyesee,
                    height: Fixed::from_int(32),
                    flags: MobjFlags::NOBLOCKMAP | MobjFlags::NOSECTOR,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 87,
                    height: Fixed::from_int(32),
                    flags: MobjFlags::NOBLOCKMAP | MobjFlags::NOSECTOR,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Spawn1,
                    seesound: SfxName::Bospit,
                    deathsound: SfxName::Firxpl,
                    speed: 10 * FRACUNIT.to_bits(),
                    radius: Fixed::from_int(6),
                    height: Fixed::from_int(32),
                    damage: 3,
                    flags: MobjFlags::NOBLOCKMAP
                        | MobjFlags::MISSILE
                        | MobjFlags::DROPOFF
                        | MobjFlags::NOGRAVITY
                        | MobjFlags::NOCLIP,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Spawnfire1,
                    flags: MobjFlags::NOBLOCKMAP | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2035,
                    spawnstate: StateNum::Bar1,
                    spawnhealth: 20,
                    deathstate: StateNum::Bexp,
                    deathsound: SfxName::Barexp,
                    radius: Fixed::from_int(10),
                    height: Fixed::from_int(42),
                    flags: MobjFlags::SOLID | MobjFlags::SHOOTABLE | MobjFlags::NOBLOOD,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Tball1,
                    seesound: SfxName::Firsht,
                    deathstate: StateNum::Tballx1,
                    deathsound: SfxName::Firxpl,
                    speed: 10 * FRACUNIT.to_bits(),
                    radius: Fixed::from_int(6),
                    height: Fixed::from_int(8),
                    damage: 3,
                    flags: MobjFlags::NOBLOCKMAP
                        | MobjFlags::MISSILE
                        | MobjFlags::DROPOFF
                        | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Rball1,
                    seesound: SfxName::Firsht,
                    deathstate: StateNum::Rballx1,
                    deathsound: SfxName::Firxpl,
                    speed: 10 * FRACUNIT.to_bits(),
                    radius: Fixed::from_int(6),
                    height: Fixed::from_int(8),
                    damage: 5,
                    flags: MobjFlags::NOBLOCKMAP
                        | MobjFlags::MISSILE
                        | MobjFlags::DROPOFF
                        | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Rocket,
                    seesound: SfxName::Rlaunc,
                    deathstate: StateNum::Explode1,
                    deathsound: SfxName::Barexp,
                    speed: 20 * FRACUNIT.to_bits(),
                    radius: Fixed::from_int(11),
                    height: Fixed::from_int(8),
                    damage: 20,
                    flags: MobjFlags::NOBLOCKMAP
                        | MobjFlags::MISSILE
                        | MobjFlags::DROPOFF
                        | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Plasball,
                    seesound: SfxName::Plasma,
                    deathstate: StateNum::Plasexp,
                    deathsound: SfxName::Firxpl,
                    speed: 25 * FRACUNIT.to_bits(),
                    radius: Fixed::from_int(13),
                    height: Fixed::from_int(8),
                    damage: 5,
                    flags: MobjFlags::NOBLOCKMAP
                        | MobjFlags::MISSILE
                        | MobjFlags::DROPOFF
                        | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Bfgshot,
                    deathstate: StateNum::Bfgland,
                    deathsound: SfxName::Rxplod,
                    speed: 25 * FRACUNIT.to_bits(),
                    radius: Fixed::from_int(13),
                    height: Fixed::from_int(8),
                    damage: 100,
                    flags: MobjFlags::NOBLOCKMAP
                        | MobjFlags::MISSILE
                        | MobjFlags::DROPOFF
                        | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::ArachPlaz,
                    seesound: SfxName::Plasma,
                    deathstate: StateNum::ArachPlex,
                    deathsound: SfxName::Firxpl,
                    speed: 25 * FRACUNIT.to_bits(),
                    radius: Fixed::from_int(13),
                    height: Fixed::from_int(8),
                    damage: 5,
                    flags: MobjFlags::NOBLOCKMAP
                        | MobjFlags::MISSILE
                        | MobjFlags::DROPOFF
                        | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Puff1,
                    flags: MobjFlags::NOBLOCKMAP | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Blood1,
                    flags: MobjFlags::NOBLOCKMAP,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Tfog,
                    flags: MobjFlags::NOBLOCKMAP | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Ifog,
                    flags: MobjFlags::NOBLOCKMAP | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 14,
                    flags: MobjFlags::NOBLOCKMAP | MobjFlags::NOSECTOR,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    spawnstate: StateNum::Bfgexp,
                    flags: MobjFlags::NOBLOCKMAP | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2018,
                    spawnstate: StateNum::Arm1,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2019,
                    spawnstate: StateNum::Arm2,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2014,
                    spawnstate: StateNum::Bon1,
                    flags: MobjFlags::SPECIAL | MobjFlags::COUNTITEM,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2015,
                    spawnstate: StateNum::Bon2,
                    flags: MobjFlags::SPECIAL | MobjFlags::COUNTITEM,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 5,
                    spawnstate: StateNum::Bkey,
                    flags: MobjFlags::SPECIAL | MobjFlags::NOTDMATCH,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 13,
                    spawnstate: StateNum::Rkey,
                    flags: MobjFlags::SPECIAL | MobjFlags::NOTDMATCH,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 6,
                    spawnstate: StateNum::Ykey,
                    flags: MobjFlags::SPECIAL | MobjFlags::NOTDMATCH,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 39,
                    spawnstate: StateNum::Yskull,
                    flags: MobjFlags::SPECIAL | MobjFlags::NOTDMATCH,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 38,
                    spawnstate: StateNum::Rskull,
                    flags: MobjFlags::SPECIAL | MobjFlags::NOTDMATCH,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 40,
                    spawnstate: StateNum::Bskull,
                    flags: MobjFlags::SPECIAL | MobjFlags::NOTDMATCH,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2011,
                    spawnstate: StateNum::Stim,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2012,
                    spawnstate: StateNum::Medi,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2013,
                    spawnstate: StateNum::Soul,
                    flags: MobjFlags::SPECIAL | MobjFlags::COUNTITEM,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2022,
                    spawnstate: StateNum::Pinv,
                    flags: MobjFlags::SPECIAL | MobjFlags::COUNTITEM,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2023,
                    spawnstate: StateNum::Pstr,
                    flags: MobjFlags::SPECIAL | MobjFlags::COUNTITEM,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2024,
                    spawnstate: StateNum::Pins,
                    flags: MobjFlags::SPECIAL | MobjFlags::COUNTITEM,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2025,
                    spawnstate: StateNum::Suit,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2026,
                    spawnstate: StateNum::Pmap,
                    flags: MobjFlags::SPECIAL | MobjFlags::COUNTITEM,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2045,
                    spawnstate: StateNum::Pvis,
                    flags: MobjFlags::SPECIAL | MobjFlags::COUNTITEM,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 83,
                    spawnstate: StateNum::Mega,
                    flags: MobjFlags::SPECIAL | MobjFlags::COUNTITEM,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2007,
                    spawnstate: StateNum::Clip,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2048,
                    spawnstate: StateNum::Ammo,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2010,
                    spawnstate: StateNum::Rock,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2046,
                    spawnstate: StateNum::Brok,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2047,
                    spawnstate: StateNum::Cell,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 17,
                    spawnstate: StateNum::Celp,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2008,
                    spawnstate: StateNum::Shel,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2049,
                    spawnstate: StateNum::Sbox,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 8,
                    spawnstate: StateNum::Bpak,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2006,
                    spawnstate: StateNum::Bfug,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2002,
                    spawnstate: StateNum::Mgun,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2005,
                    spawnstate: StateNum::Csaw,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2003,
                    spawnstate: StateNum::Laun,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2004,
                    spawnstate: StateNum::Plas,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2001,
                    spawnstate: StateNum::Shot,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 82,
                    spawnstate: StateNum::Shot2,
                    flags: MobjFlags::SPECIAL,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 85,
                    spawnstate: StateNum::Techlamp,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 86,
                    spawnstate: StateNum::Tech2lamp,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 2028,
                    spawnstate: StateNum::Colu,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 30,
                    spawnstate: StateNum::Tallgrncol,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 31,
                    spawnstate: StateNum::Shrtgrncol,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 32,
                    spawnstate: StateNum::Tallredcol,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 33,
                    spawnstate: StateNum::Shrtredcol,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 37,
                    spawnstate: StateNum::Skullcol,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 36,
                    spawnstate: StateNum::Heartcol,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 41,
                    spawnstate: StateNum::Evileye,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 42,
                    spawnstate: StateNum::Floatskull,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 43,
                    spawnstate: StateNum::Torchtree,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 44,
                    spawnstate: StateNum::Bluetorch,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 45,
                    spawnstate: StateNum::Greentorch,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 46,
                    spawnstate: StateNum::Redtorch,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 55,
                    spawnstate: StateNum::Btorchshrt,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 56,
                    spawnstate: StateNum::Gtorchshrt,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 57,
                    spawnstate: StateNum::Rtorchshrt,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 47,
                    spawnstate: StateNum::Stalagtite,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 48,
                    spawnstate: StateNum::Techpillar,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 34,
                    spawnstate: StateNum::Candlestik,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 35,
                    spawnstate: StateNum::Candelabra,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 49,
                    spawnstate: StateNum::Bloodytwitch,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(68),
                    flags: MobjFlags::SOLID | MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 50,
                    spawnstate: StateNum::Meat2,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(84),
                    flags: MobjFlags::SOLID | MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 51,
                    spawnstate: StateNum::Meat3,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(84),
                    flags: MobjFlags::SOLID | MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 52,
                    spawnstate: StateNum::Meat4,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(68),
                    flags: MobjFlags::SOLID | MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 53,
                    spawnstate: StateNum::Meat5,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(52),
                    flags: MobjFlags::SOLID | MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 59,
                    spawnstate: StateNum::Meat2,
                    height: Fixed::from_int(84),
                    flags: MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 60,
                    spawnstate: StateNum::Meat4,
                    height: Fixed::from_int(68),
                    flags: MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 61,
                    spawnstate: StateNum::Meat3,
                    height: Fixed::from_int(52),
                    flags: MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 62,
                    spawnstate: StateNum::Meat5,
                    height: Fixed::from_int(52),
                    flags: MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 63,
                    spawnstate: StateNum::Bloodytwitch,
                    height: Fixed::from_int(68),
                    flags: MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 22,
                    spawnstate: StateNum::HeadDie6,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 15,
                    spawnstate: StateNum::PlayDie7,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 18,
                    spawnstate: StateNum::PossDie5,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 21,
                    spawnstate: StateNum::SargDie6,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 23,
                    spawnstate: StateNum::SkullDie6,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 20,
                    spawnstate: StateNum::TrooDie5,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 19,
                    spawnstate: StateNum::SposDie5,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 10,
                    spawnstate: StateNum::PlayXdie9,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 12,
                    spawnstate: StateNum::PlayXdie9,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 28,
                    spawnstate: StateNum::Headsonstick,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 24,
                    spawnstate: StateNum::Gibs,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 27,
                    spawnstate: StateNum::Headonastick,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 29,
                    spawnstate: StateNum::Headcandles,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 25,
                    spawnstate: StateNum::Deadstick,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 26,
                    spawnstate: StateNum::Livestick,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 54,
                    spawnstate: StateNum::Bigtree,
                    radius: Fixed::from_int(32),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 70,
                    spawnstate: StateNum::Bbar1,
                    radius: Fixed::from_int(16),
                    flags: MobjFlags::SOLID,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 73,
                    spawnstate: StateNum::Hangnoguts,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(88),
                    flags: MobjFlags::SOLID | MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 74,
                    spawnstate: StateNum::Hangbnobrain,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(88),
                    flags: MobjFlags::SOLID | MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 75,
                    spawnstate: StateNum::Hangtlookdn,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(64),
                    flags: MobjFlags::SOLID | MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 76,
                    spawnstate: StateNum::Hangtskull,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(64),
                    flags: MobjFlags::SOLID | MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 77,
                    spawnstate: StateNum::Hangtlookup,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(64),
                    flags: MobjFlags::SOLID | MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 78,
                    spawnstate: StateNum::Hangtnobrain,
                    radius: Fixed::from_int(16),
                    height: Fixed::from_int(64),
                    flags: MobjFlags::SOLID | MobjFlags::SPAWNCEILING | MobjFlags::NOGRAVITY,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 79,
                    spawnstate: StateNum::Colongibs,
                    flags: MobjFlags::NOBLOCKMAP,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 80,
                    spawnstate: StateNum::Smallpool,
                    flags: MobjFlags::NOBLOCKMAP,
                    ..MobjInfo::DEFAULT
                },
                MobjInfo {
                    doomednum: 81,
                    spawnstate: StateNum::Brainstem,
                    flags: MobjFlags::NOBLOCKMAP,
                    ..MobjInfo::DEFAULT
                },
            ],
        }
    }
}
