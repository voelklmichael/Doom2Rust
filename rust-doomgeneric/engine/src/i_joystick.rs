use crate::m_config::bind_variable_int;
use crate::m_config::MConfigState;

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
        Self {
            usejoystick: 0,
            joystick_index: -1,
            joystick_x_axis: 0,
            joystick_x_invert: 0,
            joystick_y_axis: 1,
            joystick_y_invert: 0,
            joystick_strafe_axis: -1,
            joystick_strafe_invert: 0,
            joystick_physical_buttons: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        }
    }
}

pub fn bind_joystick_variables(m_config: &mut MConfigState) {
    bind_variable_int(m_config, "use_joystick", |s| {
        &mut s.io.i_joystick.usejoystick
    });
    bind_variable_int(m_config, "joystick_index", |s| {
        &mut s.io.i_joystick.joystick_index
    });
    bind_variable_int(m_config, "joystick_x_axis", |s| {
        &mut s.io.i_joystick.joystick_x_axis
    });
    bind_variable_int(m_config, "joystick_y_axis", |s| {
        &mut s.io.i_joystick.joystick_y_axis
    });
    bind_variable_int(m_config, "joystick_strafe_axis", |s| {
        &mut s.io.i_joystick.joystick_strafe_axis
    });
    bind_variable_int(m_config, "joystick_x_invert", |s| {
        &mut s.io.i_joystick.joystick_x_invert
    });
    bind_variable_int(m_config, "joystick_y_invert", |s| {
        &mut s.io.i_joystick.joystick_y_invert
    });
    bind_variable_int(m_config, "joystick_strafe_invert", |s| {
        &mut s.io.i_joystick.joystick_strafe_invert
    });
    for i in 0..NUM_VIRTUAL_BUTTONS as usize {
        let name = format!("joystick_physical_button{i}");
        bind_variable_int(m_config, &name, move |s| {
            &mut s.io.i_joystick.joystick_physical_buttons[i]
        });
    }
}
