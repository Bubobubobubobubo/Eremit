//use mlua::{Lua, MultiValue, Function as LuaFunction, FromLuaMulti};
use mlua::prelude::*;
use mlua::MultiValue;
use rustyline::DefaultEditor;
use mlua::Result as LuaResult;
use std::sync::{Arc, Mutex};

pub struct Interpreter {
    pub lua: Lua,
    exit: Arc<Mutex<bool>>,
    editor: DefaultEditor
}

impl Interpreter {
    pub fn new() -> Self {
        let exit = Arc::new(Mutex::new(false));
        let lua = Lua::new();
        let editor = DefaultEditor::new().expect("Failed to create editor");
        Interpreter {
            lua,
            exit,
            editor,
        }
    }

    pub fn run(&mut self) -> LuaResult<()> {
        let globals = self.lua.globals();
        let exit_clone = self.exit.clone();

        let quit = self.lua.create_function(move |_, ()| -> LuaResult<()> {
            let mut exit = exit_clone.lock().unwrap();
            *exit = true;
            Ok(())
        })?;
        globals.set("quit", quit)?;
        loop {
             let prompt = "=> ";

             while !*self.exit.lock().unwrap() {
                 let mut line = String::new();
                 match self.editor.readline(prompt) {
                     Ok(input) => line.push_str(&input),
                     Err(_) => {},
                 }
                 match self.lua.load(&line).eval::<MultiValue>() {
                     Ok(values) => {
                         self.editor.add_history_entry(line.clone()).unwrap();
                         println!(
                             "{}",
                             values
                                 .iter()
                                 .map(|value| format!("{:#?}", value))
                                 .collect::<Vec<_>>()
                                 .join("\t")
                         );
                         continue;
                     }
                     Err(e) => {
                         eprintln!("error: {}", e);
                         continue;
                     }
                 }
             }
             if *self.exit.lock().unwrap() {
                 return Ok(());
             }
         }
    }

    pub fn register_function<'lua, F, A, R>(&'lua self, name: &str, function: F) -> LuaResult<()>
    where
        F: Fn(&'lua Lua, A) -> LuaResult<R>,
        F: 'static,
        A: FromLuaMulti<'lua>,
        R: IntoLuaMulti<'lua>,
    {
        let func = self.lua.create_function(function)?;
        self.lua.globals().set(name, func)?;
        Ok(())
    }

    pub fn register_void_function<'lua, F>(&'lua self, name: &str, function: F) -> LuaResult<()>
    where
        F: Fn() + 'static,
    {
        self.register_function(name, move |_lua, ()| {
            function();
            Ok(())
        })
    }
}