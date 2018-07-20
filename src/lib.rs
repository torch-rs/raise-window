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

use xcb::Connection;
use failure::{Error, err_msg};

pub fn raise_app(app_name: String) -> Result<(), Error> {
    let condition = &format!("class = \"{}\"", app_name);
    let cond = condition.parse().map_err(|_| err_msg("Invalid condition"))?;

    let (conn, screen_num) = Connection::connect(None)?;
    let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();

    match windows::find_matching_window(&conn, &screen, &cond)? {
        Some(win) => windows::set_active_window(&conn, &screen, win)?,
        None => return Err(err_msg("No matching window found")),
    }
    conn.flush();

    Ok(())
}

#[cfg(test)]
mod tests {

    use raise_app;

    #[test]
    fn raise_app_test() {
        assert!(raise_app(String::from("Firefox")).is_ok());
    }

}
