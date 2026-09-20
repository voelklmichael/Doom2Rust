use crate::d_event::EvType;
use crate::d_event::Event;
use crate::d_event::GameAction;
use crate::d_event::GameScreenState;
use crate::d_mode::GameMission;
use crate::d_mode::GameMode;
use crate::d_mode::GameVersion;
use crate::d_player::PlayerId;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::hu_stuff::HU_FONTSIZE;
use crate::hu_stuff::HU_FONTSTART;
use crate::i_video::IVideoState;
use crate::info::StateId;
use crate::patch::Patch;
use crate::v_video::cache_patch_name;
use crate::v_video::Screen;

use crate::p_mobj::MobjType;
use crate::p_mobj::StateNum;

use crate::r_things::FF_FRAMEMASK;
use crate::s_sound::change_music;
use crate::s_sound::s_start_sound;
use crate::s_sound::start_music;
use crate::s_sound::SoundOrigin;
use crate::sounds::MusicName;
use crate::sounds::SfxName;

use crate::v_video::cache_patch_num;
use crate::v_video::draw_patch;
use crate::v_video::draw_patch_flipped;
use crate::v_video::mark_rect;
use crate::w_wad::lump_bytes_name;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FinaleStage {
    Text,
    ArtScreen,
    Cast,
}
#[derive(Copy, Clone)]
pub struct TextScreen {
    pub mission: GameMission,
    pub episode: i32,
    pub level: i32,
    pub background: &'static str,
    pub text: &'static str,
}
#[derive(Copy, Clone)]
pub struct CastInfo {
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
pub const TEXTWAIT: usize = 250;
const INITIAL_TEXTSCREENS: [TextScreen; 22] = [
    TextScreen {
        mission: GameMission::Doom,
        episode: 1,
        level: 8,
        background: "FLOOR4_8",
        text: E1TEXT,
    },
    TextScreen {
        mission: GameMission::Doom,
        episode: 2,
        level: 8,
        background: "SFLR6_1",
        text: E2TEXT,
    },
    TextScreen {
        mission: GameMission::Doom,
        episode: 3,
        level: 8,
        background: "MFLR8_4",
        text: E3TEXT,
    },
    TextScreen {
        mission: GameMission::Doom,
        episode: 4,
        level: 8,
        background: "MFLR8_3",
        text: E4TEXT,
    },
    TextScreen {
        mission: GameMission::Doom2,
        episode: 1,
        level: 6,
        background: "SLIME16",
        text: C1TEXT,
    },
    TextScreen {
        mission: GameMission::Doom2,
        episode: 1,
        level: 11,
        background: "RROCK14",
        text: C2TEXT,
    },
    TextScreen {
        mission: GameMission::Doom2,
        episode: 1,
        level: 20,
        background: "RROCK07",
        text: C3TEXT,
    },
    TextScreen {
        mission: GameMission::Doom2,
        episode: 1,
        level: 30,
        background: "RROCK17",
        text: C4TEXT,
    },
    TextScreen {
        mission: GameMission::Doom2,
        episode: 1,
        level: 15,
        background: "RROCK13",
        text: C5TEXT,
    },
    TextScreen {
        mission: GameMission::Doom2,
        episode: 1,
        level: 31,
        background: "RROCK19",
        text: C6TEXT,
    },
    TextScreen {
        mission: GameMission::PackTnt,
        episode: 1,
        level: 6,
        background: "SLIME16",
        text: T1TEXT,
    },
    TextScreen {
        mission: GameMission::PackTnt,
        episode: 1,
        level: 11,
        background: "RROCK14",
        text: T2TEXT,
    },
    TextScreen {
        mission: GameMission::PackTnt,
        episode: 1,
        level: 20,
        background: "RROCK07",
        text: T3TEXT,
    },
    TextScreen {
        mission: GameMission::PackTnt,
        episode: 1,
        level: 30,
        background: "RROCK17",
        text: T4TEXT,
    },
    TextScreen {
        mission: GameMission::PackTnt,
        episode: 1,
        level: 15,
        background: "RROCK13",
        text: T5TEXT,
    },
    TextScreen {
        mission: GameMission::PackTnt,
        episode: 1,
        level: 31,
        background: "RROCK19",
        text: T6TEXT,
    },
    TextScreen {
        mission: GameMission::PackPlut,
        episode: 1,
        level: 6,
        background: "SLIME16",
        text: P1TEXT,
    },
    TextScreen {
        mission: GameMission::PackPlut,
        episode: 1,
        level: 11,
        background: "RROCK14",
        text: P2TEXT,
    },
    TextScreen {
        mission: GameMission::PackPlut,
        episode: 1,
        level: 20,
        background: "RROCK07",
        text: P3TEXT,
    },
    TextScreen {
        mission: GameMission::PackPlut,
        episode: 1,
        level: 30,
        background: "RROCK17",
        text: P4TEXT,
    },
    TextScreen {
        mission: GameMission::PackPlut,
        episode: 1,
        level: 15,
        background: "RROCK13",
        text: P5TEXT,
    },
    TextScreen {
        mission: GameMission::PackPlut,
        episode: 1,
        level: 31,
        background: "RROCK19",
        text: P6TEXT,
    },
];

pub struct FFinaleState {
    finalestage: FinaleStage,
    pub(crate) finalecount: u32,
    textscreens: [TextScreen; 22],
    finaletext: &'static str,
    finaleflat: &'static str,
    pub(crate) castorder: [CastInfo; 18],
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
        Self {
            finalestage: FinaleStage::Text,
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

pub fn f_start_finale(state: &mut GameState) {
    state.game.g_game.gameaction = GameAction::Nothing;
    state.game.g_game.gamestate = GameScreenState::Finale;
    state.game.g_game.viewactive = false;
    state.ui.am_map.automapactive = false;
    if state.game.doomstat.gamemission.base() == GameMission::Doom {
        change_music(state, MusicName::Victor as i32, true);
    } else {
        change_music(state, MusicName::ReadM as i32, true);
    }
    let gamemission = if state.game.doomstat.gamemission == GameMission::PackChex {
        GameMission::Doom
    } else if state.game.doomstat.gamemission == GameMission::PackHacx {
        GameMission::Doom2
    } else {
        state.game.doomstat.gamemission
    };
    for screen in &mut state.ui.f_finale.textscreens {
        if state.game.doomstat.gameversion == GameVersion::Chex
            && screen.mission == GameMission::Doom
        {
            screen.level = 5;
        }
        if gamemission == screen.mission
            && (gamemission != GameMission::Doom || state.game.g_game.gameepisode == screen.episode)
            && state.game.g_game.gamemap == screen.level
        {
            state.ui.f_finale.finaletext = screen.text;
            state.ui.f_finale.finaleflat = screen.background;
        }
    }
    state.ui.f_finale.finalestage = FinaleStage::Text;
    state.ui.f_finale.finalecount = 0;
}
pub fn f_responder(state: &mut GameState, event: &Event) -> bool {
    if state.ui.f_finale.finalestage == FinaleStage::Cast {
        return cast_responder(state, event);
    }
    false
}
pub fn f_ticker(state: &mut GameState) {
    if state.game.doomstat.gamemode == GameMode::Commercial && state.ui.f_finale.finalecount > 50 {
        let anyone_pressing =
            PlayerId::all().any(|p| state.game.g_game.players[p].cmd.buttons != 0);
        if anyone_pressing {
            if state.game.g_game.gamemap == 30 {
                start_cast(state);
            } else {
                state.game.g_game.gameaction = GameAction::WorldDone;
            }
        }
    }
    state.ui.f_finale.finalecount = state.ui.f_finale.finalecount.wrapping_add(1);
    if state.ui.f_finale.finalestage == FinaleStage::Cast {
        cast_ticker(state);
        return;
    }
    if state.game.doomstat.gamemode == GameMode::Commercial {
        return;
    }
    if state.ui.f_finale.finalestage == FinaleStage::Text
        && state.ui.f_finale.finalecount as usize
            > state
                .ui
                .f_finale
                .finaletext
                .len()
                .wrapping_mul(TEXTSPEED as usize)
                .wrapping_add(TEXTWAIT)
    {
        state.ui.f_finale.finalecount = 0;
        state.ui.f_finale.finalestage = FinaleStage::ArtScreen;
        state.game.d_main.wipegamestate = GameScreenState::Wipped;
        if state.game.g_game.gameepisode == 3 {
            start_music(state, MusicName::Bunny as i32);
        }
    }
}
pub fn text_write(state: &mut GameState) {
    let flat = lump_bytes_name(
        &*state.assets.fs,
        &mut state.assets.w_wad,
        state.ui.f_finale.finaleflat,
    );
    let video = &mut state.io.i_video.i_video_buffer;
    for y in 0..SCREENHEIGHT as usize {
        let row = &flat[(y & 63) << 6..][..64];
        let line = &mut video[y * SCREENWIDTH as usize..][..SCREENWIDTH as usize];
        for chunk in line.chunks_mut(64) {
            chunk.copy_from_slice(&row[..chunk.len()]);
        }
    }
    let dest_screen = Screen::Video;
    mark_rect(
        &mut state.io.v_video,
        dest_screen,
        0,
        0,
        SCREENWIDTH,
        SCREENHEIGHT,
    );
    let mut cx: i32 = 10;
    let mut cy: i32 = 10;
    let mut chars = state.ui.f_finale.finaletext.bytes();
    let mut count: i32 = (state.ui.f_finale.finalecount as i32 - 10) / TEXTSPEED;
    if count < 0 {
        count = 0;
    }
    while count != 0 {
        let mut c: i32 = match chars.next() {
            Some(b) => i32::from(b),
            None => break,
        };
        if c == '\n' as i32 {
            cx = 10;
            cy += 11;
        } else {
            c = i32::from((c as u8).to_ascii_uppercase()) - HU_FONTSTART;
            if (0..=HU_FONTSIZE).contains(&c) {
                let font_patch = cache_patch_num(
                    &*state.assets.fs,
                    &mut state.assets.w_wad,
                    state.ui.hu_stuff.hu_font[c as usize],
                );
                let w: i32 = font_patch.width();
                if cx + w > SCREENWIDTH {
                    break;
                }
                let dest_screen = Screen::Video;
                draw_patch(state, dest_screen, cx, cy, &font_patch);
                cx += w;
            } else {
                cx += 4;
            }
        }
        count -= 1;
    }
}
const INITIAL_CASTORDER: [CastInfo; 18] = [
    CastInfo {
        name: Some(CC_ZOMBIE),
        kind: MobjType::Possessed,
    },
    CastInfo {
        name: Some(CC_SHOTGUN),
        kind: MobjType::Shotguy,
    },
    CastInfo {
        name: Some(CC_HEAVY),
        kind: MobjType::Chainguy,
    },
    CastInfo {
        name: Some(CC_IMP),
        kind: MobjType::Troop,
    },
    CastInfo {
        name: Some(CC_DEMON),
        kind: MobjType::Sergeant,
    },
    CastInfo {
        name: Some(CC_LOST),
        kind: MobjType::Skull,
    },
    CastInfo {
        name: Some(CC_CACO),
        kind: MobjType::Head,
    },
    CastInfo {
        name: Some(CC_HELL),
        kind: MobjType::Knight,
    },
    CastInfo {
        name: Some(CC_BARON),
        kind: MobjType::Bruiser,
    },
    CastInfo {
        name: Some(CC_ARACH),
        kind: MobjType::Baby,
    },
    CastInfo {
        name: Some(CC_PAIN),
        kind: MobjType::Pain,
    },
    CastInfo {
        name: Some(CC_REVEN),
        kind: MobjType::Undead,
    },
    CastInfo {
        name: Some(CC_MANCU),
        kind: MobjType::Fatso,
    },
    CastInfo {
        name: Some(CC_ARCH),
        kind: MobjType::Vile,
    },
    CastInfo {
        name: Some(CC_SPIDER),
        kind: MobjType::Spider,
    },
    CastInfo {
        name: Some(CC_CYBER),
        kind: MobjType::Cyborg,
    },
    CastInfo {
        name: Some(CC_HERO),
        kind: MobjType::Player,
    },
    CastInfo {
        name: None,
        kind: MobjType::Player,
    },
];
pub fn start_cast(state: &mut GameState) {
    state.game.d_main.wipegamestate = GameScreenState::Wipped;
    state.ui.f_finale.castnum = 0;
    let cast_type = state.ui.f_finale.castorder[state.ui.f_finale.castnum as usize].kind;
    state.ui.f_finale.caststate = Some(StateId(
        state.assets.info.mobjinfo[cast_type as usize].seestate as u32,
    ));
    state.ui.f_finale.casttics = state
        .assets
        .info
        .state_mut(state.ui.f_finale.caststate.unwrap())
        .tics;
    state.ui.f_finale.castdeath = false;
    state.ui.f_finale.finalestage = FinaleStage::Cast;
    state.ui.f_finale.castframes = 0;
    state.ui.f_finale.castonmelee = 0;
    state.ui.f_finale.castattacking = false;
    change_music(state, MusicName::Evil as i32, true);
}
pub fn cast_ticker(state: &mut GameState) {
    state.ui.f_finale.casttics -= 1;
    if state.ui.f_finale.casttics > 0 {
        return;
    }
    let mut stop_attack = false;
    let cur_caststate = state
        .assets
        .info
        .state_mut(state.ui.f_finale.caststate.unwrap());
    if cur_caststate.tics == -1 || cur_caststate.nextstate as u32 == StateNum::Null as i32 as u32 {
        state.ui.f_finale.castnum += 1;
        state.ui.f_finale.castdeath = false;
        if state.ui.f_finale.castorder[state.ui.f_finale.castnum as usize]
            .name
            .is_none()
        {
            state.ui.f_finale.castnum = 0;
        }
        if state.assets.info.mobjinfo
            [state.ui.f_finale.castorder[state.ui.f_finale.castnum as usize].kind as usize]
            .seesound
            != SfxName::SfxNone
        {
            s_start_sound(
                state,
                SoundOrigin::None,
                state.assets.info.mobjinfo
                    [state.ui.f_finale.castorder[state.ui.f_finale.castnum as usize].kind as usize]
                    .seesound,
            );
        }
        let cast_type = state.ui.f_finale.castorder[state.ui.f_finale.castnum as usize].kind;
        state.ui.f_finale.caststate = Some(StateId(
            state.assets.info.mobjinfo[cast_type as usize].seestate as u32,
        ));
        state.ui.f_finale.castframes = 0;
    } else if state.ui.f_finale.caststate == Some(StateId(StateNum::PlayAtk1 as u32)) {
        // The player's attack frame ends the attack at once.
        stop_attack = true;
    } else {
        let st = cur_caststate.nextstate as i32;
        state.ui.f_finale.caststate = Some(StateId(st as u32));
        state.ui.f_finale.castframes += 1;
        let sfx = match st {
            154 => SfxName::Dshtgn,
            185 => SfxName::Pistol,
            218 => SfxName::Shotgn,
            256 => SfxName::Vilatk,
            336 => SfxName::Skeswg,
            338 => SfxName::Skepch,
            340 => SfxName::Skeatk,
            383 | 380 | 377 => SfxName::Firsht,
            417..=419 => SfxName::Shotgn,
            454 => SfxName::Claw,
            486 => SfxName::Sgtatk,
            538 | 567 | 505 => SfxName::Firsht,
            590 => SfxName::Sklatk,
            616 | 617 => SfxName::Shotgn,
            648 => SfxName::Plasma,
            685 | 687 | 689 => SfxName::Rlaunc,
            710 => SfxName::Sklatk,
            _ => SfxName::SfxNone,
        };
        if sfx != SfxName::SfxNone {
            s_start_sound(state, SoundOrigin::None, sfx);
        }
    }
    if !stop_attack {
        let cast_type = state.ui.f_finale.castorder[state.ui.f_finale.castnum as usize].kind;
        let cast_info = state.assets.info.mobjinfo[cast_type as usize];
        if state.ui.f_finale.castframes == 12 {
            state.ui.f_finale.castattacking = true;
            if state.ui.f_finale.castonmelee != 0 {
                state.ui.f_finale.caststate = Some(StateId(cast_info.meleestate as u32));
            } else {
                state.ui.f_finale.caststate = Some(StateId(cast_info.missilestate as u32));
            }
            state.ui.f_finale.castonmelee ^= 1;
            if state.ui.f_finale.caststate == Some(StateId(StateNum::Null as u32)) {
                if state.ui.f_finale.castonmelee != 0 {
                    state.ui.f_finale.caststate = Some(StateId(cast_info.meleestate as u32));
                } else {
                    state.ui.f_finale.caststate = Some(StateId(cast_info.missilestate as u32));
                }
            }
        }
        stop_attack = state.ui.f_finale.castattacking
            && (state.ui.f_finale.castframes == 24
                || state.ui.f_finale.caststate == Some(StateId(cast_info.seestate as u32)));
    }
    if stop_attack {
        state.ui.f_finale.castattacking = false;
        state.ui.f_finale.castframes = 0;
        let cast_type = state.ui.f_finale.castorder[state.ui.f_finale.castnum as usize].kind;
        state.ui.f_finale.caststate = Some(StateId(
            state.assets.info.mobjinfo[cast_type as usize].seestate as u32,
        ));
    }
    state.ui.f_finale.casttics = state
        .assets
        .info
        .state_mut(state.ui.f_finale.caststate.unwrap())
        .tics;
    if state.ui.f_finale.casttics == -1 {
        state.ui.f_finale.casttics = 15;
    }
}
pub fn cast_responder(state: &mut GameState, ev: &Event) -> bool {
    if ev.kind != EvType::Keydown {
        return false;
    }
    if state.ui.f_finale.castdeath {
        return true;
    }
    state.ui.f_finale.castdeath = true;
    let cast_type = state.ui.f_finale.castorder[state.ui.f_finale.castnum as usize].kind;
    state.ui.f_finale.caststate = Some(StateId(
        state.assets.info.mobjinfo[cast_type as usize].deathstate as u32,
    ));
    state.ui.f_finale.casttics = state
        .assets
        .info
        .state_mut(state.ui.f_finale.caststate.unwrap())
        .tics;
    state.ui.f_finale.castframes = 0;
    state.ui.f_finale.castattacking = false;
    if state.assets.info.mobjinfo
        [state.ui.f_finale.castorder[state.ui.f_finale.castnum as usize].kind as usize]
        .deathsound
        != SfxName::SfxNone
    {
        s_start_sound(
            state,
            SoundOrigin::None,
            state.assets.info.mobjinfo
                [state.ui.f_finale.castorder[state.ui.f_finale.castnum as usize].kind as usize]
                .deathsound,
        );
    }
    true
}
pub fn cast_print(state: &mut GameState, text: &str) {
    let mut c: i32;
    let mut w: i32;
    let mut width: i32 = 0;
    for b in text.bytes() {
        c = i32::from(b.to_ascii_uppercase()) - HU_FONTSTART;
        if (0..=HU_FONTSIZE).contains(&c) {
            w = cache_patch_num(
                &*state.assets.fs,
                &mut state.assets.w_wad,
                state.ui.hu_stuff.hu_font[c as usize],
            )
            .width();
            width += w;
        } else {
            width += 4;
        }
    }
    let mut cx: i32 = 160 - width / 2;
    for b in text.bytes() {
        c = i32::from(b.to_ascii_uppercase()) - HU_FONTSTART;
        if (0..=HU_FONTSIZE).contains(&c) {
            let font_patch = cache_patch_num(
                &*state.assets.fs,
                &mut state.assets.w_wad,
                state.ui.hu_stuff.hu_font[c as usize],
            );
            w = font_patch.width();
            let dest_screen = Screen::Video;
            draw_patch(state, dest_screen, cx, 180, &font_patch);
            cx += w;
        } else {
            cx += 4;
        }
    }
}
pub fn cast_drawer(state: &mut GameState) {
    let __wcache865_4 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "BOSSBACK");
    let dest_screen = Screen::Video;
    draw_patch(state, dest_screen, 0, 0, &__wcache865_4);
    let cast_name = state.ui.f_finale.castorder[state.ui.f_finale.castnum as usize]
        .name
        .unwrap();
    cast_print(state, cast_name);
    let cur_caststate = state
        .assets
        .info
        .state_mut(state.ui.f_finale.caststate.unwrap());
    let sprframe = &state.render.r_things.sprites[cur_caststate.sprite as usize].spriteframes
        [(cur_caststate.frame & FF_FRAMEMASK) as usize];
    let lump: i32 = i32::from(sprframe.lump[0]);
    let flip: bool = sprframe.flip[0] != 0;
    let patch: Patch = cache_patch_num(
        &*state.assets.fs,
        &mut state.assets.w_wad,
        lump + state.render.r_data.firstspritelump,
    );
    if flip {
        let dest_screen = Screen::Video;
        draw_patch_flipped(state, dest_screen, 160, 170, &patch);
    } else {
        let dest_screen = Screen::Video;
        draw_patch(state, dest_screen, 160, 170, &patch);
    }
}
fn draw_patch_col(state: &mut IVideoState, x: i32, patch: &Patch, col: i32) {
    for post in patch.posts(col) {
        let mut dest = post.topdelta * SCREENWIDTH as usize + x as usize;
        for &pixel in post.pixels {
            state.i_video_buffer[dest] = pixel;
            dest += SCREENWIDTH as usize;
        }
    }
}
pub fn bunny_scroll(state: &mut GameState) {
    let p1: Patch = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "PFUB2");
    let p2: Patch = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "PFUB1");
    let dest_screen = Screen::Video;
    mark_rect(
        &mut state.io.v_video,
        dest_screen,
        0,
        0,
        SCREENWIDTH,
        SCREENHEIGHT,
    );
    let mut scrolled: i32 = 320 - (state.ui.f_finale.finalecount as i32 - 230) / 2;
    scrolled = scrolled.clamp(0, 320);
    for x in 0..SCREENWIDTH {
        if x + scrolled < 320 {
            draw_patch_col(&mut state.io.i_video, x, &p1, x + scrolled);
        } else {
            draw_patch_col(&mut state.io.i_video, x, &p2, x + scrolled - 320);
        }
    }
    if state.ui.f_finale.finalecount < 1130 {
        return;
    }
    if state.ui.f_finale.finalecount < 1180 {
        let __wcache963_3 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "END0");
        let dest_screen = Screen::Video;
        draw_patch(
            state,
            dest_screen,
            (SCREENWIDTH - 13 * 8) / 2,
            (SCREENHEIGHT - 8 * 8) / 2,
            &__wcache963_3,
        );
        state.ui.f_finale.laststage = 0;
        return;
    }
    let mut stage: i32 = state
        .ui
        .f_finale
        .finalecount
        .wrapping_sub(1180)
        .wrapping_div(5) as i32;
    if stage > 6 {
        stage = 6;
    }
    if stage > state.ui.f_finale.laststage {
        s_start_sound(state, SoundOrigin::None, SfxName::Pistol);
        state.ui.f_finale.laststage = stage;
    }
    let name = format!("END{stage}");
    let __wcache990_2 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, &name);
    let dest_screen = Screen::Video;
    draw_patch(
        state,
        dest_screen,
        (SCREENWIDTH - 13 * 8) / 2,
        (SCREENHEIGHT - 8 * 8) / 2,
        &__wcache990_2,
    );
}
fn art_screen_drawer(state: &mut GameState) {
    if state.game.g_game.gameepisode == 3 {
        bunny_scroll(state);
    } else {
        let lumpname: &str;

        match state.game.g_game.gameepisode {
            1 => {
                if state.game.doomstat.gamemode == GameMode::Retail {
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
        let __wcache1026_1 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, lumpname);
        let dest_screen = Screen::Video;
        draw_patch(state, dest_screen, 0, 0, &__wcache1026_1);
    }
}
pub fn f_drawer(state: &mut GameState) {
    match state.ui.f_finale.finalestage {
        FinaleStage::Cast => {
            cast_drawer(state);
        }
        FinaleStage::Text => {
            text_write(state);
        }
        FinaleStage::ArtScreen => {
            art_screen_drawer(state);
        }
    }
}
