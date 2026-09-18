use crate::game_state::GameState;
use crate::m_config::M_BindVariable_int;

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

impl Default for IJoystickState {
    fn default() -> Self {
        Self::new()
    }
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
                0_i32, 1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32, 7_i32, 8_i32, 9_i32,
            ],
        }
    }
}

pub fn I_BindJoystickVariables(state: &mut GameState) {
    let mut i: i32;
    M_BindVariable_int(&mut state.m_config, "use_joystick", |s| {
        &mut s.i_joystick.usejoystick
    });
    M_BindVariable_int(&mut state.m_config, "joystick_index", |s| {
        &mut s.i_joystick.joystick_index
    });
    M_BindVariable_int(&mut state.m_config, "joystick_x_axis", |s| {
        &mut s.i_joystick.joystick_x_axis
    });
    M_BindVariable_int(&mut state.m_config, "joystick_y_axis", |s| {
        &mut s.i_joystick.joystick_y_axis
    });
    M_BindVariable_int(&mut state.m_config, "joystick_strafe_axis", |s| {
        &mut s.i_joystick.joystick_strafe_axis
    });
    M_BindVariable_int(&mut state.m_config, "joystick_x_invert", |s| {
        &mut s.i_joystick.joystick_x_invert
    });
    M_BindVariable_int(&mut state.m_config, "joystick_y_invert", |s| {
        &mut s.i_joystick.joystick_y_invert
    });
    M_BindVariable_int(&mut state.m_config, "joystick_strafe_invert", |s| {
        &mut s.i_joystick.joystick_strafe_invert
    });
    i = 0_i32;
    while i < NUM_VIRTUAL_BUTTONS {
        let name = format!("joystick_physical_button{}", i);
        M_BindVariable_int(&mut state.m_config, &name, move |s| {
            &mut s.i_joystick.joystick_physical_buttons[i as usize]
        });
        i += 1;
    }
}
