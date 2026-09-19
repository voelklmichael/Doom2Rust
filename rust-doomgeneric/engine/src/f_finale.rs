use crate::d_event::event_t;
use crate::d_event::EvType;
use crate::d_event::GameAction;
use crate::d_event::GameScreenState;
use crate::d_mode::GameMission_t;
use crate::d_mode::GameMode_t;
use crate::d_mode::GameVersion;
use crate::doomdef::MAXPLAYERS;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::hu_stuff::HU_FONTSIZE;
use crate::hu_stuff::HU_FONTSTART;
use crate::i_video::IVideoState;
use crate::info::StateId;
use crate::patch::Patch;
use crate::v_video::Screen;
use crate::v_video::V_CachePatchName;

use crate::p_mobj::MobjType;
use crate::p_mobj::StateNum;

use crate::r_things::FF_FRAMEMASK;
use crate::s_sound::S_ChangeMusic;
use crate::s_sound::S_StartMusic;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::MusicName;
use crate::sounds::SfxName;

use crate::v_video::V_CachePatchNum;
use crate::v_video::V_DrawPatch;
use crate::v_video::V_DrawPatchFlipped;
use crate::v_video::V_MarkRect;
use crate::w_wad::W_LumpBytesName;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FinaleStage {
    F_STAGE_TEXT = 0,
    F_STAGE_ARTSCREEN = 1,
    F_STAGE_CAST = 2,
}
#[derive(Copy, Clone)]
pub struct textscreen_t {
    pub mission: GameMission_t,
    pub episode: i32,
    pub level: i32,
    pub background: &'static str,
    pub text: &'static str,
}
#[derive(Copy, Clone)]
pub struct castinfo_t {
    pub name: Option<&'static str>,
    pub kind: MobjType,
}
pub const E1TEXT: &str = "Once you beat the big badasses and\nclean out the moon base you're supposed\nto win, aren't you? Aren't you? Where's\nyour fat reward and ticket home? What\nthe hell is this? It's not supposed to\nend this way!\n\nIt stinks like rotten meat, but looks\nlike the lost Deimos base.  Looks like\nyou're stuck on The Shores of Hell.\nThe only way out is through.\n\nTo continue the DOOM experience, play\nThe Shores of Hell and its amazing\nsequel, Inferno!\n";
pub const E2TEXT: &str = "You've done it! The hideous cyber-\ndemon lord that ruled the lost Deimos\nmoon base has been slain and you\nare triumphant! But ... where are\nyou? You clamber to the edge of the\nmoon and look down to see the awful\ntruth.\n\nDeimos floats above Hell itself!\nYou've never heard of anyone escaping\nfrom Hell, but you'll make the bastards\nsorry they ever heard of you! Quickly,\nyou rappel down to  the surface of\nHell.\n\nNow, it's on to the final chapter of\nDOOM! -- Inferno.";
pub const E3TEXT: &str = "The loathsome spiderdemon that\nmasterminded the invasion of the moon\nbases and caused so much death has had\nits ass kicked for all time.\n\nA hidden doorway opens and you enter.\nYou've proven too tough for Hell to\ncontain, and now Hell at last plays\nfair -- for you emerge from the door\nto see the green fields of Earth!\nHome at last.\n\nYou wonder what's been happening on\nEarth while you were battling evil\nunleashed. It's good that no Hell-\nspawn could have come through that\ndoor with you ...";
pub const E4TEXT: &str = "the spider mastermind must have sent forth\nits legions of hellspawn before your\nfinal confrontation with that terrible\nbeast from hell.  but you stepped forward\nand brought forth eternal damnation and\nsuffering upon the horde as a true hero\nwould in the face of something so evil.\n\nbesides, someone was gonna pay for what\nhappened to daisy, your pet rabbit.\n\nbut now, you see spread before you more\npotential pain and gibbitude as a nation\nof demons run amok among our cities.\n\nnext stop, hell on earth!";
pub const C1TEXT: &str = "YOU HAVE ENTERED DEEPLY INTO THE INFESTED\nSTARPORT. BUT SOMETHING IS WRONG. THE\nMONSTERS HAVE BROUGHT THEIR OWN REALITY\nWITH THEM, AND THE STARPORT'S TECHNOLOGY\nIS BEING SUBVERTED BY THEIR PRESENCE.\n\nAHEAD, YOU SEE AN OUTPOST OF HELL, A\nFORTIFIED ZONE. IF YOU CAN GET PAST IT,\nYOU CAN PENETRATE INTO THE HAUNTED HEART\nOF THE STARBASE AND FIND THE CONTROLLING\nSWITCH WHICH HOLDS EARTH'S POPULATION\nHOSTAGE.";
pub const C2TEXT: &str = "YOU HAVE WON! YOUR VICTORY HAS ENABLED\nHUMANKIND TO EVACUATE EARTH AND ESCAPE\nTHE NIGHTMARE.  NOW YOU ARE THE ONLY\nHUMAN LEFT ON THE FACE OF THE PLANET.\nCANNIBAL MUTATIONS, CARNIVOROUS ALIENS,\nAND EVIL SPIRITS ARE YOUR ONLY NEIGHBORS.\nYOU SIT BACK AND WAIT FOR DEATH, CONTENT\nTHAT YOU HAVE SAVED YOUR SPECIES.\n\nBUT THEN, EARTH CONTROL BEAMS DOWN A\nMESSAGE FROM SPACE: \"SENSORS HAVE LOCATED\nTHE SOURCE OF THE ALIEN INVASION. IF YOU\nGO THERE, YOU MAY BE ABLE TO BLOCK THEIR\nENTRY.  THE ALIEN BASE IS IN THE HEART OF\nYOUR OWN HOME CITY, NOT FAR FROM THE\nSTARPORT.\" SLOWLY AND PAINFULLY YOU GET\nUP AND RETURN TO THE FRAY.";
pub const C3TEXT: &str = "YOU ARE AT THE CORRUPT HEART OF THE CITY,\nSURROUNDED BY THE CORPSES OF YOUR ENEMIES.\nYOU SEE NO WAY TO DESTROY THE CREATURES'\nENTRYWAY ON THIS SIDE, SO YOU CLENCH YOUR\nTEETH AND PLUNGE THROUGH IT.\n\nTHERE MUST BE A WAY TO CLOSE IT ON THE\nOTHER SIDE. WHAT DO YOU CARE IF YOU'VE\nGOT TO GO THROUGH HELL TO GET TO IT?";
pub const C4TEXT: &str = "THE HORRENDOUS VISAGE OF THE BIGGEST\nDEMON YOU'VE EVER SEEN CRUMBLES BEFORE\nYOU, AFTER YOU PUMP YOUR ROCKETS INTO\nHIS EXPOSED BRAIN. THE MONSTER SHRIVELS\nUP AND DIES, ITS THRASHING LIMBS\nDEVASTATING UNTOLD MILES OF HELL'S\nSURFACE.\n\nYOU'VE DONE IT. THE INVASION IS OVER.\nEARTH IS SAVED. HELL IS A WRECK. YOU\nWONDER WHERE BAD FOLKS WILL GO WHEN THEY\nDIE, NOW. WIPING THE SWEAT FROM YOUR\nFOREHEAD YOU BEGIN THE LONG TREK BACK\nHOME. REBUILDING EARTH OUGHT TO BE A\nLOT MORE FUN THAN RUINING IT WAS.\n";
pub const C5TEXT: &str = "CONGRATULATIONS, YOU'VE FOUND THE SECRET\nLEVEL! LOOKS LIKE IT'S BEEN BUILT BY\nHUMANS, RATHER THAN DEMONS. YOU WONDER\nWHO THE INMATES OF THIS CORNER OF HELL\nWILL BE.";
pub const C6TEXT: &str = "CONGRATULATIONS, YOU'VE FOUND THE\nSUPER SECRET LEVEL!  YOU'D BETTER\nBLAZE THROUGH THIS ONE!\n";
pub const P1TEXT: &str = "You gloat over the steaming carcass of the\nGuardian.  With its death, you've wrested\nthe Accelerator from the stinking claws\nof Hell.  You relax and glance around the\nroom.  Damn!  There was supposed to be at\nleast one working prototype, but you can't\nsee it. The demons must have taken it.\n\nYou must find the prototype, or all your\nstruggles will have been wasted. Keep\nmoving, keep fighting, keep killing.\nOh yes, keep living, too.";
pub const P2TEXT: &str = "Even the deadly Arch-Vile labyrinth could\nnot stop you, and you've gotten to the\nprototype Accelerator which is soon\nefficiently and permanently deactivated.\n\nYou're good at that kind of thing.";
pub const P3TEXT: &str = "You've bashed and battered your way into\nthe heart of the devil-hive.  Time for a\nSearch-and-Destroy mission, aimed at the\nGatekeeper, whose foul offspring is\ncascading to Earth.  Yeah, he's bad. But\nyou know who's worse!\n\nGrinning evilly, you check your gear, and\nget ready to give the bastard a little Hell\nof your own making!";
pub const P4TEXT: &str = "The Gatekeeper's evil face is splattered\nall over the place.  As its tattered corpse\ncollapses, an inverted Gate forms and\nsucks down the shards of the last\nprototype Accelerator, not to mention the\nfew remaining demons.  You're done. Hell\nhas gone back to pounding bad dead folks \ninstead of good live ones.  Remember to\ntell your grandkids to put a rocket\nlauncher in your coffin. If you go to Hell\nwhen you die, you'll need it for some\nfinal cleaning-up ...";
pub const P5TEXT: &str = "You've found the second-hardest level we\ngot. Hope you have a saved game a level or\ntwo previous.  If not, be prepared to die\naplenty. For master marines only.";
pub const P6TEXT: &str = "Betcha wondered just what WAS the hardest\nlevel we had ready for ya?  Now you know.\nNo one gets out alive.";
pub const T1TEXT: &str = "You've fought your way out of the infested\nexperimental labs.   It seems that UAC has\nonce again gulped it down.  With their\nhigh turnover, it must be hard for poor\nold UAC to buy corporate health insurance\nnowadays..\n\nAhead lies the military complex, now\nswarming with diseased horrors hot to get\ntheir teeth into you. With luck, the\ncomplex still has some warlike ordnance\nlaying around.";
pub const T2TEXT: &str = "You hear the grinding of heavy machinery\nahead.  You sure hope they're not stamping\nout new hellspawn, but you're ready to\nream out a whole herd if you have to.\nThey might be planning a blood feast, but\nyou feel about as mean as two thousand\nmaniacs packed into one mad killer.\n\nYou don't plan to go down easy.";
pub const T3TEXT: &str = "The vista opening ahead looks real damn\nfamiliar. Smells familiar, too -- like\nfried excrement. You didn't like this\nplace before, and you sure as hell ain't\nplanning to like it now. The more you\nbrood on it, the madder you get.\nHefting your gun, an evil grin trickles\nonto your face. Time to take some names.";
pub const T4TEXT: &str = "Suddenly, all is silent, from one horizon\nto the other. The agonizing echo of Hell\nfades away, the nightmare sky turns to\nblue, the heaps of monster corpses start \nto evaporate along with the evil stench \nthat filled the air. Jeeze, maybe you've\ndone it. Have you really won?\n\nSomething rumbles in the distance.\nA blue light begins to glow inside the\nruined skull of the demon-spitter.";
pub const T5TEXT: &str = "What now? Looks totally different. Kind\nof like King Tut's condo. Well,\nwhatever's here can't be any worse\nthan usual. Can it?  Or maybe it's best\nto let sleeping gods lie..";
pub const T6TEXT: &str = "Time for a vacation. You've burst the\nbowels of hell and by golly you're ready\nfor a break. You mutter to yourself,\nMaybe someone else can kick Hell's ass\nnext time around. Ahead lies a quiet town,\nwith peaceful flowing water, quaint\nbuildings, and presumably no Hellspawn.\n\nAs you step off the transport, you hear\nthe stomp of a cyberdemon's iron shoe.";
pub const CC_ZOMBIE: &str = "ZOMBIEMAN";
pub const CC_SHOTGUN: &str = "SHOTGUN GUY";
pub const CC_HEAVY: &str = "HEAVY WEAPON DUDE";
pub const CC_IMP: &str = "IMP";
pub const CC_DEMON: &str = "DEMON";
pub const CC_LOST: &str = "LOST SOUL";
pub const CC_CACO: &str = "CACODEMON";
pub const CC_HELL: &str = "HELL KNIGHT";
pub const CC_BARON: &str = "BARON OF HELL";
pub const CC_ARACH: &str = "ARACHNOTRON";
pub const CC_PAIN: &str = "PAIN ELEMENTAL";
pub const CC_REVEN: &str = "REVENANT";
pub const CC_MANCU: &str = "MANCUBUS";
pub const CC_ARCH: &str = "ARCH-VILE";
pub const CC_SPIDER: &str = "THE SPIDER MASTERMIND";
pub const CC_CYBER: &str = "THE CYBERDEMON";
pub const CC_HERO: &str = "OUR HERO";
pub const TEXTSPEED: i32 = 3;
pub const TEXTWAIT: i32 = 250;
const INITIAL_TEXTSCREENS: [textscreen_t; 22] = [
    textscreen_t {
        mission: GameMission_t::doom,
        episode: 1,
        level: 8,
        background: "FLOOR4_8",
        text: E1TEXT,
    },
    textscreen_t {
        mission: GameMission_t::doom,
        episode: 2,
        level: 8,
        background: "SFLR6_1",
        text: E2TEXT,
    },
    textscreen_t {
        mission: GameMission_t::doom,
        episode: 3,
        level: 8,
        background: "MFLR8_4",
        text: E3TEXT,
    },
    textscreen_t {
        mission: GameMission_t::doom,
        episode: 4,
        level: 8,
        background: "MFLR8_3",
        text: E4TEXT,
    },
    textscreen_t {
        mission: GameMission_t::doom2,
        episode: 1,
        level: 6,
        background: "SLIME16",
        text: C1TEXT,
    },
    textscreen_t {
        mission: GameMission_t::doom2,
        episode: 1,
        level: 11,
        background: "RROCK14",
        text: C2TEXT,
    },
    textscreen_t {
        mission: GameMission_t::doom2,
        episode: 1,
        level: 20,
        background: "RROCK07",
        text: C3TEXT,
    },
    textscreen_t {
        mission: GameMission_t::doom2,
        episode: 1,
        level: 30,
        background: "RROCK17",
        text: C4TEXT,
    },
    textscreen_t {
        mission: GameMission_t::doom2,
        episode: 1,
        level: 15,
        background: "RROCK13",
        text: C5TEXT,
    },
    textscreen_t {
        mission: GameMission_t::doom2,
        episode: 1,
        level: 31,
        background: "RROCK19",
        text: C6TEXT,
    },
    textscreen_t {
        mission: GameMission_t::pack_tnt,
        episode: 1,
        level: 6,
        background: "SLIME16",
        text: T1TEXT,
    },
    textscreen_t {
        mission: GameMission_t::pack_tnt,
        episode: 1,
        level: 11,
        background: "RROCK14",
        text: T2TEXT,
    },
    textscreen_t {
        mission: GameMission_t::pack_tnt,
        episode: 1,
        level: 20,
        background: "RROCK07",
        text: T3TEXT,
    },
    textscreen_t {
        mission: GameMission_t::pack_tnt,
        episode: 1,
        level: 30,
        background: "RROCK17",
        text: T4TEXT,
    },
    textscreen_t {
        mission: GameMission_t::pack_tnt,
        episode: 1,
        level: 15,
        background: "RROCK13",
        text: T5TEXT,
    },
    textscreen_t {
        mission: GameMission_t::pack_tnt,
        episode: 1,
        level: 31,
        background: "RROCK19",
        text: T6TEXT,
    },
    textscreen_t {
        mission: GameMission_t::pack_plut,
        episode: 1,
        level: 6,
        background: "SLIME16",
        text: P1TEXT,
    },
    textscreen_t {
        mission: GameMission_t::pack_plut,
        episode: 1,
        level: 11,
        background: "RROCK14",
        text: P2TEXT,
    },
    textscreen_t {
        mission: GameMission_t::pack_plut,
        episode: 1,
        level: 20,
        background: "RROCK07",
        text: P3TEXT,
    },
    textscreen_t {
        mission: GameMission_t::pack_plut,
        episode: 1,
        level: 30,
        background: "RROCK17",
        text: P4TEXT,
    },
    textscreen_t {
        mission: GameMission_t::pack_plut,
        episode: 1,
        level: 15,
        background: "RROCK13",
        text: P5TEXT,
    },
    textscreen_t {
        mission: GameMission_t::pack_plut,
        episode: 1,
        level: 31,
        background: "RROCK19",
        text: P6TEXT,
    },
];

pub struct FFinaleState {
    finalestage: FinaleStage,
    pub(crate) finalecount: u32,
    textscreens: [textscreen_t; 22],
    finaletext: &'static str,
    finaleflat: &'static str,
    pub(crate) castorder: [castinfo_t; 18],
    pub(crate) castnum: i32,
    pub(crate) casttics: i32,
    pub(crate) caststate: Option<StateId>,
    pub(crate) castdeath: bool,
    pub(crate) castframes: i32,
    pub(crate) castonmelee: i32,
    pub(crate) castattacking: bool,
    laststage: i32,
}

impl Default for FFinaleState {
    fn default() -> Self {
        Self::new()
    }
}

impl FFinaleState {
    pub const fn new() -> Self {
        FFinaleState {
            finalestage: FinaleStage::F_STAGE_TEXT,
            finalecount: 0,
            textscreens: INITIAL_TEXTSCREENS,
            finaletext: "",
            finaleflat: "",
            castorder: INITIAL_CASTORDER,
            castnum: 0,
            casttics: 0,
            caststate: None,
            castdeath: false,
            castframes: 0,
            castonmelee: 0,
            castattacking: false,
            laststage: 0,
        }
    }
}

pub fn F_StartFinale(state: &mut GameState) {
    state.g_game.gameaction = GameAction::ga_nothing;
    state.g_game.gamestate = GameScreenState::GS_FINALE;
    state.g_game.viewactive = false;
    state.am_map.automapactive = false;
    if (if state.doomstat.gamemission as u32 == GameMission_t::pack_chex as i32 as u32 {
        GameMission_t::doom as i32 as u32
    } else if state.doomstat.gamemission as u32 == GameMission_t::pack_hacx as i32 as u32 {
        GameMission_t::doom2 as i32 as u32
    } else {
        state.doomstat.gamemission as u32
    }) == GameMission_t::doom as i32 as u32
    {
        S_ChangeMusic(state, MusicName::mus_victor as i32, true);
    } else {
        S_ChangeMusic(state, MusicName::mus_read_m as i32, true);
    }
    let gamemission = if state.doomstat.gamemission == GameMission_t::pack_chex {
        GameMission_t::doom
    } else if state.doomstat.gamemission == GameMission_t::pack_hacx {
        GameMission_t::doom2
    } else {
        state.doomstat.gamemission
    };
    for screen in state.f_finale.textscreens.iter_mut() {
        if state.doomstat.gameversion == GameVersion::chex && screen.mission == GameMission_t::doom
        {
            screen.level = 5;
        }
        if gamemission == screen.mission
            && (gamemission != GameMission_t::doom || state.g_game.gameepisode == screen.episode)
            && state.g_game.gamemap == screen.level
        {
            state.f_finale.finaletext = screen.text;
            state.f_finale.finaleflat = screen.background;
        }
    }
    state.f_finale.finalestage = FinaleStage::F_STAGE_TEXT;
    state.f_finale.finalecount = 0;
}
pub fn F_Responder(state: &mut GameState, event: &event_t) -> bool {
    if state.f_finale.finalestage == FinaleStage::F_STAGE_CAST {
        return F_CastResponder(state, event);
    }
    false
}
pub fn F_Ticker(state: &mut GameState) {
    let mut i: usize;
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32
        && state.f_finale.finalecount > 50
    {
        i = 0_usize;
        while i < MAXPLAYERS as usize {
            if state.g_game.players[i].cmd.buttons != 0 {
                break;
            }
            i = i.wrapping_add(1);
        }
        if i < MAXPLAYERS as usize {
            if state.g_game.gamemap == 30 {
                F_StartCast(state);
            } else {
                state.g_game.gameaction = GameAction::ga_worlddone;
            }
        }
    }
    state.f_finale.finalecount = state.f_finale.finalecount.wrapping_add(1);
    if state.f_finale.finalestage == FinaleStage::F_STAGE_CAST {
        F_CastTicker(state);
        return;
    }
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        return;
    }
    if state.f_finale.finalestage == FinaleStage::F_STAGE_TEXT
        && state.f_finale.finalecount as usize
            > state
                .f_finale
                .finaletext
                .len()
                .wrapping_mul(TEXTSPEED as usize)
                .wrapping_add(TEXTWAIT as usize)
    {
        state.f_finale.finalecount = 0;
        state.f_finale.finalestage = FinaleStage::F_STAGE_ARTSCREEN;
        state.d_main.wipegamestate = GameScreenState::GS_WIPPED;
        if state.g_game.gameepisode == 3 {
            S_StartMusic(state, MusicName::mus_bunny as i32);
        }
    }
}
pub fn F_TextWrite(state: &mut GameState) {
    let mut w: i32;
    let mut count: i32;
    let mut c: i32;
    let mut cx: i32;
    let mut cy: i32;
    let flat = W_LumpBytesName(state, state.f_finale.finaleflat);
    let video = &mut state.i_video.I_VideoBuffer;
    for y in 0..SCREENHEIGHT as usize {
        let row = &flat[(y & 63) << 6..][..64];
        let line = &mut video[y * SCREENWIDTH as usize..][..SCREENWIDTH as usize];
        for chunk in line.chunks_mut(64) {
            chunk.copy_from_slice(&row[..chunk.len()]);
        }
    }
    let dest_screen = Screen::Video;
    V_MarkRect(state, dest_screen, 0, 0, SCREENWIDTH, SCREENHEIGHT);
    cx = 10;
    cy = 10;
    let mut chars = state.f_finale.finaletext.bytes();
    count = (state.f_finale.finalecount as i32 - 10) / TEXTSPEED;
    if count < 0 {
        count = 0;
    }
    while count != 0 {
        c = match chars.next() {
            Some(b) => b as i32,
            None => break,
        };
        if c == '\n' as i32 {
            cx = 10;
            cy += 11;
        } else {
            c = (c as u8).to_ascii_uppercase() as i32 - HU_FONTSTART;
            if !(0..=HU_FONTSIZE).contains(&c) {
                cx += 4;
            } else {
                let font_patch = V_CachePatchNum(state, state.hu_stuff.hu_font[c as usize]);
                w = font_patch.width();
                if cx + w > SCREENWIDTH {
                    break;
                }
                let dest_screen = Screen::Video;
                V_DrawPatch(state, dest_screen, cx, cy, &font_patch);
                cx += w;
            }
        }
        count -= 1;
    }
}
const INITIAL_CASTORDER: [castinfo_t; 18] = [
    castinfo_t {
        name: Some(CC_ZOMBIE),
        kind: MobjType::MT_POSSESSED,
    },
    castinfo_t {
        name: Some(CC_SHOTGUN),
        kind: MobjType::MT_SHOTGUY,
    },
    castinfo_t {
        name: Some(CC_HEAVY),
        kind: MobjType::MT_CHAINGUY,
    },
    castinfo_t {
        name: Some(CC_IMP),
        kind: MobjType::MT_TROOP,
    },
    castinfo_t {
        name: Some(CC_DEMON),
        kind: MobjType::MT_SERGEANT,
    },
    castinfo_t {
        name: Some(CC_LOST),
        kind: MobjType::MT_SKULL,
    },
    castinfo_t {
        name: Some(CC_CACO),
        kind: MobjType::MT_HEAD,
    },
    castinfo_t {
        name: Some(CC_HELL),
        kind: MobjType::MT_KNIGHT,
    },
    castinfo_t {
        name: Some(CC_BARON),
        kind: MobjType::MT_BRUISER,
    },
    castinfo_t {
        name: Some(CC_ARACH),
        kind: MobjType::MT_BABY,
    },
    castinfo_t {
        name: Some(CC_PAIN),
        kind: MobjType::MT_PAIN,
    },
    castinfo_t {
        name: Some(CC_REVEN),
        kind: MobjType::MT_UNDEAD,
    },
    castinfo_t {
        name: Some(CC_MANCU),
        kind: MobjType::MT_FATSO,
    },
    castinfo_t {
        name: Some(CC_ARCH),
        kind: MobjType::MT_VILE,
    },
    castinfo_t {
        name: Some(CC_SPIDER),
        kind: MobjType::MT_SPIDER,
    },
    castinfo_t {
        name: Some(CC_CYBER),
        kind: MobjType::MT_CYBORG,
    },
    castinfo_t {
        name: Some(CC_HERO),
        kind: MobjType::MT_PLAYER,
    },
    castinfo_t {
        name: None,
        kind: MobjType::MT_PLAYER,
    },
];
pub fn F_StartCast(state: &mut GameState) {
    state.d_main.wipegamestate = GameScreenState::GS_WIPPED;
    state.f_finale.castnum = 0;
    let cast_type = state.f_finale.castorder[state.f_finale.castnum as usize].kind;
    state.f_finale.caststate = Some(StateId(
        state.info.mobjinfo[cast_type as usize].seestate as u32,
    ));
    state.f_finale.casttics = state.info.state_mut(state.f_finale.caststate.unwrap()).tics;
    state.f_finale.castdeath = false;
    state.f_finale.finalestage = FinaleStage::F_STAGE_CAST;
    state.f_finale.castframes = 0;
    state.f_finale.castonmelee = 0;
    state.f_finale.castattacking = false;
    S_ChangeMusic(state, MusicName::mus_evil as i32, true);
}
pub fn F_CastTicker(state: &mut GameState) {
    state.f_finale.casttics -= 1;
    if state.f_finale.casttics > 0 {
        return;
    }
    let mut stop_attack = false;
    let cur_caststate = state.info.state_mut(state.f_finale.caststate.unwrap());
    if cur_caststate.tics == -1 || cur_caststate.nextstate as u32 == StateNum::S_NULL as i32 as u32
    {
        state.f_finale.castnum += 1;
        state.f_finale.castdeath = false;
        if state.f_finale.castorder[state.f_finale.castnum as usize]
            .name
            .is_none()
        {
            state.f_finale.castnum = 0;
        }
        if state.info.mobjinfo
            [state.f_finale.castorder[state.f_finale.castnum as usize].kind as usize]
            .seesound
            != 0
        {
            S_StartSound(
                state,
                SoundOrigin::None,
                state.info.mobjinfo
                    [state.f_finale.castorder[state.f_finale.castnum as usize].kind as usize]
                    .seesound,
            );
        }
        let cast_type = state.f_finale.castorder[state.f_finale.castnum as usize].kind;
        state.f_finale.caststate = Some(StateId(
            state.info.mobjinfo[cast_type as usize].seestate as u32,
        ));
        state.f_finale.castframes = 0;
    } else if state.f_finale.caststate == Some(StateId(StateNum::S_PLAY_ATK1 as u32)) {
        // The player's attack frame ends the attack at once.
        stop_attack = true;
    } else {
        let st = cur_caststate.nextstate as i32;
        state.f_finale.caststate = Some(StateId(st as u32));
        state.f_finale.castframes += 1;
        let sfx = match st {
            154 => SfxName::sfx_dshtgn as i32,
            185 => SfxName::sfx_pistol as i32,
            218 => SfxName::sfx_shotgn as i32,
            256 => SfxName::sfx_vilatk as i32,
            336 => SfxName::sfx_skeswg as i32,
            338 => SfxName::sfx_skepch as i32,
            340 => SfxName::sfx_skeatk as i32,
            383 | 380 | 377 => SfxName::sfx_firsht as i32,
            417..=419 => SfxName::sfx_shotgn as i32,
            454 => SfxName::sfx_claw as i32,
            486 => SfxName::sfx_sgtatk as i32,
            538 | 567 | 505 => SfxName::sfx_firsht as i32,
            590 => SfxName::sfx_sklatk as i32,
            616 | 617 => SfxName::sfx_shotgn as i32,
            648 => SfxName::sfx_plasma as i32,
            685 | 687 | 689 => SfxName::sfx_rlaunc as i32,
            710 => SfxName::sfx_sklatk as i32,
            _ => 0,
        };
        if sfx != 0 {
            S_StartSound(state, SoundOrigin::None, sfx);
        }
    }
    if !stop_attack {
        let cast_type = state.f_finale.castorder[state.f_finale.castnum as usize].kind;
        let cast_info = state.info.mobjinfo[cast_type as usize];
        if state.f_finale.castframes == 12 {
            state.f_finale.castattacking = true;
            if state.f_finale.castonmelee != 0 {
                state.f_finale.caststate = Some(StateId(cast_info.meleestate as u32));
            } else {
                state.f_finale.caststate = Some(StateId(cast_info.missilestate as u32));
            }
            state.f_finale.castonmelee ^= 1;
            if state.f_finale.caststate == Some(StateId(StateNum::S_NULL as u32)) {
                if state.f_finale.castonmelee != 0 {
                    state.f_finale.caststate = Some(StateId(cast_info.meleestate as u32));
                } else {
                    state.f_finale.caststate = Some(StateId(cast_info.missilestate as u32));
                }
            }
        }
        stop_attack = state.f_finale.castattacking
            && (state.f_finale.castframes == 24
                || state.f_finale.caststate == Some(StateId(cast_info.seestate as u32)));
    }
    if stop_attack {
        state.f_finale.castattacking = false;
        state.f_finale.castframes = 0;
        let cast_type = state.f_finale.castorder[state.f_finale.castnum as usize].kind;
        state.f_finale.caststate = Some(StateId(
            state.info.mobjinfo[cast_type as usize].seestate as u32,
        ));
    }
    state.f_finale.casttics = state.info.state_mut(state.f_finale.caststate.unwrap()).tics;
    if state.f_finale.casttics == -1 {
        state.f_finale.casttics = 15;
    }
}
pub fn F_CastResponder(state: &mut GameState, ev: &event_t) -> bool {
    if ev.kind != EvType::ev_keydown {
        return false;
    }
    if state.f_finale.castdeath {
        return true;
    }
    state.f_finale.castdeath = true;
    let cast_type = state.f_finale.castorder[state.f_finale.castnum as usize].kind;
    state.f_finale.caststate = Some(StateId(
        state.info.mobjinfo[cast_type as usize].deathstate as u32,
    ));
    state.f_finale.casttics = state.info.state_mut(state.f_finale.caststate.unwrap()).tics;
    state.f_finale.castframes = 0;
    state.f_finale.castattacking = false;
    if state.info.mobjinfo[state.f_finale.castorder[state.f_finale.castnum as usize].kind as usize]
        .deathsound
        != 0
    {
        S_StartSound(
            state,
            SoundOrigin::None,
            state.info.mobjinfo
                [state.f_finale.castorder[state.f_finale.castnum as usize].kind as usize]
                .deathsound,
        );
    }
    true
}
pub fn F_CastPrint(state: &mut GameState, text: &str) {
    let mut c: i32;
    let mut cx: i32;
    let mut w: i32;
    let mut width: i32 = 0;
    for b in text.bytes() {
        c = b.to_ascii_uppercase() as i32 - HU_FONTSTART;
        if !(0..=HU_FONTSIZE).contains(&c) {
            width += 4;
        } else {
            w = V_CachePatchNum(state, state.hu_stuff.hu_font[c as usize]).width();
            width += w;
        }
    }
    cx = 160 - width / 2;
    for b in text.bytes() {
        c = b.to_ascii_uppercase() as i32 - HU_FONTSTART;
        if !(0..=HU_FONTSIZE).contains(&c) {
            cx += 4;
        } else {
            let font_patch = V_CachePatchNum(state, state.hu_stuff.hu_font[c as usize]);
            w = font_patch.width();
            let dest_screen = Screen::Video;
            V_DrawPatch(state, dest_screen, cx, 180, &font_patch);
            cx += w;
        }
    }
}
pub fn F_CastDrawer(state: &mut GameState) {
    let __wcache865_4 = V_CachePatchName(state, "BOSSBACK");
    let dest_screen = Screen::Video;
    V_DrawPatch(state, dest_screen, 0, 0, &__wcache865_4);
    let cast_name = state.f_finale.castorder[state.f_finale.castnum as usize]
        .name
        .unwrap();
    F_CastPrint(state, cast_name);
    let cur_caststate = state.info.state_mut(state.f_finale.caststate.unwrap());
    let sprframe = &state.r_things.sprites[cur_caststate.sprite as usize].spriteframes
        [(cur_caststate.frame & FF_FRAMEMASK) as usize];
    let lump: i32 = sprframe.lump[0] as i32;
    let flip: bool = sprframe.flip[0] != 0;
    let patch: Patch = V_CachePatchNum(state, lump + state.r_data.firstspritelump);
    if flip {
        let dest_screen = Screen::Video;
        V_DrawPatchFlipped(state, dest_screen, 160, 170, &patch);
    } else {
        let dest_screen = Screen::Video;
        V_DrawPatch(state, dest_screen, 160, 170, &patch);
    };
}
fn F_DrawPatchCol(state: &mut IVideoState, x: i32, patch: &Patch, col: i32) {
    for post in patch.posts(col) {
        let mut dest = post.topdelta * SCREENWIDTH as usize + x as usize;
        for &pixel in post.pixels {
            state.I_VideoBuffer[dest] = pixel;
            dest += SCREENWIDTH as usize;
        }
    }
}
pub fn F_BunnyScroll(state: &mut GameState) {
    let mut scrolled: i32;

    let mut stage: i32;
    let p1: Patch = V_CachePatchName(state, "PFUB2");
    let p2: Patch = V_CachePatchName(state, "PFUB1");
    let dest_screen = Screen::Video;
    V_MarkRect(state, dest_screen, 0, 0, SCREENWIDTH, SCREENHEIGHT);
    scrolled = 320 - (state.f_finale.finalecount as i32 - 230) / 2;
    scrolled = scrolled.clamp(0, 320);
    for x in 0..SCREENWIDTH {
        if x + scrolled < 320 {
            F_DrawPatchCol(&mut state.i_video, x, &p1, x + scrolled);
        } else {
            F_DrawPatchCol(&mut state.i_video, x, &p2, x + scrolled - 320);
        }
    }
    if state.f_finale.finalecount < 1130 {
        return;
    }
    if state.f_finale.finalecount < 1180 {
        let __wcache963_3 = V_CachePatchName(state, "END0");
        let dest_screen = Screen::Video;
        V_DrawPatch(
            state,
            dest_screen,
            (SCREENWIDTH - 13 * 8) / 2,
            (SCREENHEIGHT - 8 * 8) / 2,
            &__wcache963_3,
        );
        state.f_finale.laststage = 0;
        return;
    }
    stage = state
        .f_finale
        .finalecount
        .wrapping_sub(1180)
        .wrapping_div(5) as i32;
    if stage > 6 {
        stage = 6;
    }
    if stage > state.f_finale.laststage {
        S_StartSound(state, SoundOrigin::None, SfxName::sfx_pistol as i32);
        state.f_finale.laststage = stage;
    }
    let name = format!("END{}", stage);
    let __wcache990_2 = V_CachePatchName(state, &name);
    let dest_screen = Screen::Video;
    V_DrawPatch(
        state,
        dest_screen,
        (SCREENWIDTH - 13 * 8) / 2,
        (SCREENHEIGHT - 8 * 8) / 2,
        &__wcache990_2,
    );
}
fn F_ArtScreenDrawer(state: &mut GameState) {
    let lumpname: &str;
    if state.g_game.gameepisode == 3 {
        F_BunnyScroll(state);
    } else {
        match state.g_game.gameepisode {
            1 => {
                if state.doomstat.gamemode as u32 == GameMode_t::retail as i32 as u32 {
                    lumpname = "CREDIT";
                } else {
                    lumpname = "HELP2";
                }
            }
            2 => {
                lumpname = "VICTORY2";
            }
            4 => {
                lumpname = "ENDPIC";
            }
            _ => return,
        }
        let __wcache1026_1 = V_CachePatchName(state, lumpname);
        let dest_screen = Screen::Video;
        V_DrawPatch(state, dest_screen, 0, 0, &__wcache1026_1);
    };
}
pub fn F_Drawer(state: &mut GameState) {
    match state.f_finale.finalestage {
        FinaleStage::F_STAGE_CAST => {
            F_CastDrawer(state);
        }
        FinaleStage::F_STAGE_TEXT => {
            F_TextWrite(state);
        }
        FinaleStage::F_STAGE_ARTSCREEN => {
            F_ArtScreenDrawer(state);
        }
    };
}
