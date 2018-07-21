extern crate xcb;
extern crate regex;
extern crate encoding;
extern crate failure;

#[macro_use]
extern crate nom;
#[macro_use]
extern crate lazy_static;

mod parsing;
mod windows;
mod conditions;

use xcb::{Atom, Connection, Screen, Window};
use failure::{Error, err_msg};

fn get_property(conn: &Connection, window: Window, atom: Atom) -> Option<Vec<u8>> {
    let cookie = xcb::get_property(conn,
                                   false,
                                   window,
                                   atom,
                                   xcb::GET_PROPERTY_TYPE_ANY,
                                   0,
                                   u32::max_value());
    if let Ok(reply) = cookie.get_reply() {
        let value: &[u8] = reply.value();
        return Some(value.iter().cloned().collect());
    }
    None
}

pub fn raise_window(conn: &Connection, screen: &Screen, win: Window) -> Result<(), Error> {
    let net_wm_desktop = windows::get_atom(&conn, "_NET_WM_DESKTOP")?;
    if let Some(value) = get_property(&conn, win, net_wm_desktop) {
        let desktop_index = value[0] as u32;
        let data = xcb::ClientMessageData::from_data32([desktop_index,
                                                        xcb::CURRENT_TIME,
                                                        0,
                                                        0,
                                                        0]);
        let net_current_desktop = windows::get_atom(&conn, "_NET_CURRENT_DESKTOP")?;
        let ev = xcb::ClientMessageEvent::new(32, screen.root(), net_current_desktop, data);
        xcb::send_event(&conn,
                        false,
                        screen.root(),
                        xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
                        &ev).request_check()?;
        windows::set_active_window(&conn, &screen, win)?;
        conn.flush();
    }

    Ok(())
}

pub fn raise_window_by_class(name: String) -> Result<(), Error> {
    let condition = &format!("class = \"{}\"", name);
    let cond = condition.parse().map_err(|_| err_msg("Invalid condition"))?;

    let (conn, screen_num) = Connection::connect(None)?;
    let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();

    match windows::find_matching_window(&conn, &screen, &cond)? {
        Some(win) => raise_window(&conn, &screen, win),
        None => Err(err_msg("No matching window found")),
    }
}

#[cfg(test)]
mod tests {

    use raise_window_by_class;

    #[test]
    fn raise_window_by_class_test() {
        assert!(raise_window_by_class(String::from("Caprine")).is_ok());
    }

}
