include!("dummy.rs");

/*//use std::mem;

use State;

/*type Tchar = i16;
const MAXPNAMELEN: usize = 32;
const MAX_JOYSTICKOEMVXDNAME: usize = 260;

#[derive(Copy, Clone)] #[repr(C)]
struct JoyCaps {
    w_mid: u16,
    w_pid: u16,
    sz_pname: [Tchar; MAXPNAMELEN],
    x_min: u32,
    x_max: u32,
    y_min: u32,
    y_max: u32,
    z_min: u32,
    z_max: u32,
    num_buttons: u32,
    period_min: u32,
    period_max: u32,
    r_min: u32,
    r_max: u32,
    u_min: u32,
    u_max: u32,
    v_min: u32,
    v_max: u32,
    caps: u32,
    max_axes: u32,
    num_axes: u32,
    max_buttons: u32,
    sz_reg_key: [Tchar; MAXPNAMELEN],
    sz_oem_vxd: [Tchar; MAX_JOYSTICKOEMVXDNAME],
}

#[derive(Copy, Clone)] #[repr(C)]
struct JoyInfo {
    size: u32,
    flags: u32,
    x_pos: u32,
    y_pos: u32,
    z_pos: u32,
    r_pos: u32,
    u_pos: u32,
    v_pos: u32,
    buttons: u32,
    button_number: u32,
    pov: u32,
    reserved_1: u32,
    reserved_2: u32,
}

#[derive(Copy, Clone)]
struct Device {
    caps: JoyCaps,
    info: JoyInfo,
}

// Link to the windows multimedia library.
#[link(name = "winmm")]
extern "system" {
    // Get number of joysticks that the system supports.
    fn joyGetNumDevs() -> u32;
    //
    fn joyGetDevCapsW(joy_id: usize, caps: *mut JoyCaps, cbjc: u32) -> u32;
    //
    fn joyGetPosEx(joy_id: u32, pji: *mut JoyInfo) -> u32;
}*/

pub struct NativeManager {/*joy_caps: Vec<Option<Device>>*/}
impl NativeManager {
    pub fn new() -> NativeManager {
        /*		let supported = unsafe { joyGetNumDevs() } as usize;
        let mut joy_caps = Vec::new();

        joy_caps.resize(supported, None);*/

        NativeManager { /*joy_caps*/ }
    }

    /// Do a search for controllers.  Returns number of controllers.
    pub fn search(&mut self) -> (usize, usize) {
        /*		for dev in 0..self.joy_caps.len() {
            let mut pos = unsafe { mem::uninitialized() };

            // If Can't connect to joystick
            if unsafe { joyGetPosEx(dev as u32, &mut pos) } == 167 {
                println!("WIN: unplugged");
                self.disconnect(dev as i32);
                continue;
            }
            // Can connect & is already recorded so
            if !self.joy_caps[dev].is_none() {
                self.joy_caps[dev].as_mut().unwrap().info = pos;
            // If Can, but is recorded as unplugged
            } else {
                let mut joy_caps = unsafe { mem::uninitialized() };

                unsafe {
                    joyGetDevCapsW(0, &mut joy_caps,
                        mem::size_of::<JoyCaps>() as u32);
                }

                panic!("WIN: New joystick {}", dev);

                self.joy_caps[dev] = Some(Device {
                    caps: joy_caps,
                    info: pos,
                });

                return (self.joy_caps.len(), self.joy_caps.len());
            }
        }*/

        (0 /*self.joy_caps.len()*/, ::std::usize::MAX)
    }

    //	pub fn num_plugged_in(&self) -> usize {
    //		self.joy_caps.len()
    //		0
    //	}

    pub fn disconnect(&mut self, _fd: i32) -> () {
        //		self.joy_caps[fd as usize] = None;
    }

    pub fn get_fd(&self, id: usize) -> (i32, bool, bool) {
        /*		if id >= self.joy_caps.len() {
            (0, true, true)
        } else {*/
        (id as i32, false, false)
        //		}
    }

    pub fn get_id(&self, _id: usize) -> (i32, bool) {
        //		if id >= self.joy_caps.len() {
        //			(0, true)
        //		} else {
        (-1, false)
        //		}
    }

    pub fn get_abs(&self, _id: usize) -> (i32, i32, bool) {
        //		if id >= self.joy_caps.len() {
        (0, 0, true)
        //		} else {
        //			(self.joy_caps[id].unwrap().caps.x_min as i32,
        //			 self.joy_caps[id].unwrap().caps.x_max as i32, false)
        //		}
    }

    pub(crate) fn poll_event(&mut self, _i: usize, _state: &mut State) -> () {
        //		println!("POLL_EVENT");
        //		if self.joy_caps[i].is_none() {
        //			println!("NONE {}", i);
        //			{ let mut a = 0; a+=1; }
        return;
        /*		} else {
            println!("SOME {}", i);
        }

        let info = self.joy_caps[i].unwrap().info;

        println!("BT {}", info.buttons);

        state.execute = (info.buttons |
            0b_00000000_00000000_00000000_00000001) != 0;
        state.accept = (info.buttons |
            0b_00000000_00000000_00000000_00000010) != 0;
        state.cancel = (info.buttons |
            0b_00000000_00000000_00000000_00000100) != 0;
        state.trigger = (info.buttons |
            0b_00000000_00000000_00000000_00001000) != 0;
        state.l[0] = (info.buttons |
            0b_00000000_00000000_00000000_00010000) != 0;
        state.r[0] = (info.buttons |
            0b_00000000_00000000_00000000_00100000) != 0;
        state.r[1] = (info.buttons |
            0b_00000000_00000000_00000000_10000000) != 0;
        state.menu = (info.buttons |
            0b_00000000_00000000_00000010_00000000) != 0;

        state.move_xy.0 = info.x_pos as f32 / 100.0;
        state.move_xy.1 = info.y_pos as f32 / 100.0;
        state.cam_xy.1 = info.z_pos as f32 / 100.0;
        state.left_throttle = info.r_pos as f32 / 100.0;
        state.right_throttle = info.u_pos as f32 / 100.0;
        state.cam_xy.0 = info.v_pos as f32 / 100.0;

        match info.pov as u16 {
            ::std::u16::MAX => {
                state.right = false;
                state.left = false;
                state.up = false;
                state.down = false;
            }
            270_00 => {
                state.right = false;
                state.left = true;
                state.up = false;
                state.down = false;
            }
            90_00 => {
                state.right = true;
                state.left = false;
                state.up = false;
                state.down = false;
            }
            0 => {
                state.right = false;
                state.left = false;
                state.up = true;
                state.down = false;
            }
            180_00 => {
                state.right = false;
                state.left = false;
                state.up = false;
                state.down = true;
            }
            a => {
                println!("unknown {}", a);
            }
        }*/
    }
}
impl Drop for NativeManager {
    fn drop(&mut self) -> () {
        //		if self.native != -1 {
        //			destroy::joystick(self.native);
        //		}
    }
}*/
