use crate::windowmanager::WindowManager;
use crate::xlibwrapper::core::*;
use crate::xlibwrapper::event::*;
use std::rc::Rc;
use crate::xlibwrapper::masks::*;

pub fn button_press(xlib: Rc<XlibWrapper>, wm: &mut WindowManager, event: Event) {

    let (window, x_root, y_root, state) =
        match event {
            Event {
                event_type: EventType::ButtonPress,
                payload: Some(EventPayload::ButtonPress(window, _sub_window, _button, x_root, y_root, state))
            } => (window, x_root, y_root, state),
            _ => { return; }
        };


    if !wm.clients.contains_key(&window) || window == xlib.get_root() {
        return
    }

    println!("Button pressed from: {}", window);

    let ww = wm.clients.get(&window).expect("ButtonPressed: No such window in client list");
    let geometry = xlib.get_geometry(ww.window());

    wm.drag_start_pos = (x_root as i32 , y_root as i32);
    wm.drag_start_frame_pos = (geometry.x,geometry.y);
    wm.drag_start_frame_size = (geometry.width, geometry.height);
    
    match ww.get_dec() {
        Some(dec) => {
            xlib.raise_window(dec);
            xlib.raise_window(ww.window());
        },
        None => xlib.raise_window(ww.window())
    }
}
