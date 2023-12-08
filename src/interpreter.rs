use mlua::{Lua, MultiValue, Function};
use rustyline::DefaultEditor;
use mlua::Result as LuaResult;
use std::sync::{Arc, Mutex};

pub struct Interpreter {
    lua: Lua,
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

    pub fn add_function(&self, name: &str, func: Function) -> LuaResult<()> {
        let globals = self.lua.globals();
        globals.set(name, func)?;
        Ok(())
    }
}