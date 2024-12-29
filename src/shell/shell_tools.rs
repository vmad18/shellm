use std::io::{Read, Write};
use std::process::{Command, exit, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use crate::utils::color::{animate_text, colorify};
use crate::utils::model_tool::{ChatRole, ChatWrapper, ModelContainer, ModelInstance};
use crate::utils::utils::get_sys_threads;

pub enum ModelMode {
    CMD,
    CODE,
    MATH,
    GENERAL
}

impl ModelMode {
    fn get_system_prompt(&self) -> &str {
        match *self {
            ModelMode::CMD => "Provide only bash commands for linux without any description. If there is a lack of details, provide most logical solution. Ensure the output is a valid shell command. If multiple steps required try to combine them together using &&. Provide only plain text without Markdown formatting. MAKE SURE TO NOT provide markdown formatting such as ```.",
            ModelMode::CODE => "",
            ModelMode::MATH => "",
            ModelMode::GENERAL => "",
        }
    }
}

pub struct ModelStatus(pub bool);

#[derive(Debug)]
pub struct ShellCreationError;

pub struct ShellLM<'a> {
    instance: ModelInstance<'a>,
    model_mode: ModelMode,
    shell_mode: bool,
    query: ChatWrapper,
    save_path: Option<String>,
    program_out_file: Option<String>,
}

impl <'a> ShellLM<'a> {
    pub fn new(query: Option<&str>,
               model_mode: ModelMode,
               shell_mode: bool,
               load_session: Option<String>,
               save_path: Option<String>,
               program_out_file: Option<String>,
               container: &'a ModelContainer,
               ctx_window: u32) -> Result<Self, ShellCreationError> {
        let threads = Some((get_sys_threads() / 2) as i32);
        let instance =
            if let Some(load_path) = load_session
            {
                match ModelInstance::load_from_session(container, threads, None, ctx_window, load_path.clone()) {
                    Ok(instance) => instance,
                    Err(_e) => {
                        eprintln!("Could not load session {}!", load_path);
                        return Err(ShellCreationError)
                    }
                }
            } else {
                ModelInstance::new(container, threads, None, ctx_window)
            };

        let mut init_query = ChatWrapper::new();
        init_query.add_dialogue(ChatRole::System, model_mode.get_system_prompt());

        if let Some(query) = query {
            init_query.add_dialogue(ChatRole::User, query);
        }

        Ok(ShellLM { instance, model_mode, shell_mode, query: init_query, save_path, program_out_file } )
    }

    pub fn process_query(&mut self) -> String {
        let model_status = ModelStatus(false);
        let state = Arc::new(Mutex::new(model_status));
        self.loading_indicator(Arc::clone(&state));

        let toks = self.instance.chat_query(&self.query, 500, false, false).unwrap();
        let result = self.instance.decode_tokens(toks, false);

        state.lock().unwrap().0 = true;
        sleep(Duration::from_millis(50));

        result
    }


    fn exec_bash_cmd(cmd: String) {
        println!("{}", colorify("Generated command:", 150., 150., 150.));
        println!();
        println!("      {}", colorify(&cmd, 59., 235., 115.));
        println!();
        println!("{}", colorify("Cannot guarantee that the command is 'safe.'\nVerify the command if you're uncertain.", 150., 150., 150.));
        print!("     [E]xecute [A]bort (default) ");
        std::io::stdout().flush().unwrap();

        let mut buffer = String::new();

        std::io::stdin().read_line(&mut buffer).unwrap();

        if buffer.to_lowercase() == "e\n" {
            let mut child = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Failed to execute command");

            let status = child.wait().expect("Something went wrong!");
            if !status.success() {
                eprintln!("Command could not execute successfully");
            }
        } else {
            println!("{}", colorify("Aborted", 247., 89., 89.))
        }

    }


    // pub fn get_files(&self) -> String {
    //
    // }
    //
    // pub fn get_wd(&self) -> String {
    //
    // }

    fn exit_event(&self) {
        let save = self.save_path.clone();
        if let Some(save_path) = save {
            self.instance.save_curr_session(Some(save_path)).expect("Could not save session!");
        }
        println!("{}", colorify("🔮 Exiting", 129., 59., 235.))
    }

    fn loading_indicator(&self, model_status: Arc<Mutex<ModelStatus>>) {
        thread::spawn(move || {
                                animate_text("✨ ─────── running magik ─────── ✨", -0.009, || {
                                    let status = {
                                        let model_status_lock = model_status.lock().unwrap();
                                        !model_status_lock.0
                                    };
                                    status
                                }) });
    }

    fn run_shell(&mut self) {
        let shell_tag = colorify("shellm 🔮", 129., 59., 235.);
        let tilda = colorify("~", 59., 150., 235.);
        loop {
            // Self::clear_stdin();
            let mut buffer = String::new();
            if self.query.len() != 2 {
                print!("{} {} ", shell_tag, tilda);
                std::io::stdout().flush().unwrap(); // flush to stdout
                std::io::stdin().read_line(&mut buffer).unwrap();

                if buffer == "exit\n" {
                    self.exit_event();
                    break;
                }

                self.query.add_dialogue(ChatRole::User, &buffer);
            }

            let result = self.process_query();

            match self.model_mode {
                ModelMode::CMD => { Self::exec_bash_cmd(result) },
                _ => {}
            }

            self.query.clear();

        }
    }

    pub fn run(&mut self) {
        if self.shell_mode {
            self.run_shell();
        } else {

            if self.query.len() < 2 {
                eprintln!("{}", colorify("No query provided", 247., 89., 89.));
                return;
            }

            let result = self.process_query();
            match self.model_mode {
                ModelMode::CMD => { Self::exec_bash_cmd(result) },
                _ => {}
            }
            // TODO check if the curr mode is CMD or what not you
        }
    }

}