use crate::game_state::GameState;
use crate::m_config::M_BindVariable;

pub const NUM_VIRTUAL_BUTTONS: i32 = 10;

pub struct IJoystickState {
    usejoystick: i32,
    joystick_index: i32,
    joystick_x_axis: i32,
    joystick_x_invert: i32,
    joystick_y_axis: i32,
    joystick_y_invert: i32,
    joystick_strafe_axis: i32,
    joystick_strafe_invert: i32,
    joystick_physical_buttons: [i32; 10],
}

impl IJoystickState {
    pub const fn new() -> Self {
        IJoystickState {
            usejoystick: 0,
            joystick_index: -1,
            joystick_x_axis: 0,
            joystick_x_invert: 0,
            joystick_y_axis: 1,
            joystick_y_invert: 0,
            joystick_strafe_axis: -1,
            joystick_strafe_invert: 0,
            joystick_physical_buttons: [
                0 as i32, 1 as i32, 2 as i32, 3 as i32, 4 as i32, 5 as i32, 6 as i32, 7 as i32,
                8 as i32, 9 as i32,
            ],
        }
    }
}

pub unsafe fn I_BindJoystickVariables(state: &mut GameState) {
    let mut i: i32 = 0;
    M_BindVariable(
        &mut state.m_config,
        "use_joystick",
        &raw mut state.i_joystick.usejoystick as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "joystick_index",
        &raw mut state.i_joystick.joystick_index as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "joystick_x_axis",
        &raw mut state.i_joystick.joystick_x_axis as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "joystick_y_axis",
        &raw mut state.i_joystick.joystick_y_axis as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "joystick_strafe_axis",
        &raw mut state.i_joystick.joystick_strafe_axis as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "joystick_x_invert",
        &raw mut state.i_joystick.joystick_x_invert as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "joystick_y_invert",
        &raw mut state.i_joystick.joystick_y_invert as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "joystick_strafe_invert",
        &raw mut state.i_joystick.joystick_strafe_invert as *mut ::core::ffi::c_void,
    );
    i = 0 as i32;
    while i < NUM_VIRTUAL_BUTTONS {
        let name = format!("joystick_physical_button{}", i);
        M_BindVariable(
            &mut state.m_config,
            &name,
            (&raw mut state.i_joystick.joystick_physical_buttons as *mut i32).offset(i as isize)
                as *mut i32 as *mut ::core::ffi::c_void,
        );
        i += 1;
    }
}
