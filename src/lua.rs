extern crate ggez;

use crate::gamestate::GameState;
use rlua::{Lua, Result};

pub fn create_lua_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("GameState", GameState::new())?;

    let make_defaults = lua.create_function(|_, ()| {
        println!("This function will create a default gameplay screen");
        Ok(())
    })?;

    globals.set("make_defaults", make_defaults)?;

    let add_note_layout = lua.create_function(|_, ()| {
        println!("This function will create a Note Layout");
        Ok(())
    })?;

    globals.set("add_note_layout", add_note_layout)?;

    let add_notefield = lua.create_function(|_, ()| {
        println!("This function will create a Notefield");
        Ok(())
    })?;

    globals.set("add_notefield", add_notefield)?;

    let add_music = lua.create_function(|_, ()| {
        println!("This function will add music");
        Ok(())
    })?;

    globals.set("add_music", add_music)?;

    Ok(())
}
