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
        loop {
            let mut prompt = "> ";
            let mut line = String::new();
    
            loop {
                match self.editor.readline(prompt) {
                    Ok(input) => line.push_str(&input),
                    Err(_) => return Ok(()),
                }
    
                match self.lua.load(&line).eval::<MultiValue>() {
                    Ok(values) => {
                        self.editor.add_history_entry(line).unwrap();
                        println!(
                            "{}",
                            values
                                .iter()
                                .map(|value| format!("{:#?}", value))
                                .collect::<Vec<_>>()
                                .join("\t")
                        );
                        break;
                    }
                    Err(mlua::Error::SyntaxError {
                        incomplete_input: true,
                        ..
                    }) => {
                        // continue reading input and append it to `line`
                        line.push_str("\n"); // separate input lines
                        prompt = ">> ";
                    }
                    Err(e) => {
                        eprintln!("error: {}", e);
                        break;
                    }
                }
            }
        }
        Ok(())
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

    pub fn register_value<'lua, T>(&'lua self, name: &str, value: T) -> LuaResult<()>
    where
        T: IntoLua<'lua>,
    {
        self.lua.globals().set(name, value)?;
        Ok(())
    }

}