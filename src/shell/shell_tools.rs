use crate::utils::color::{animate_text, colorify};
use crate::utils::model_tool::{ChatRole, ChatWrapper, ModelContainer, ModelInstance};
use crate::utils::utils::get_sys_threads;
use std::io::{Read, Write};
use std::process::{exit, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs, thread};
use std::error::Error;

pub enum ModelMode {
    CMD,
    CODE,
    MATH,
    GENERAL,
}

impl ModelMode {
    fn get_system_prompt(&self) -> &str {
        match *self {
            ModelMode::CMD => "Provide only bash commands for linux without any description. If there is a lack of details, provide most logical solution. Ensure the output is a valid shell command. If multiple steps required try to combine them together using &&. Provide only plain text without Markdown formatting. MAKE SURE TO NOT provide markdown formatting such as ```. Certain user requests will provide the current working directory and the list of files in the working directory. Use this information to inform the creation of the bash commands ONLY if it's necessary. It will be formatted as WD: {} FILES: {}.",
            ModelMode::CODE => "",
            ModelMode::MATH => "",
            ModelMode::GENERAL => "",
        }
    }
}

pub struct ModelStatus(pub bool);

#[derive(Debug)]
pub struct ShellCreationError;

pub struct Shellm<'a> {
    instance: ModelInstance<'a>,
    model_mode: ModelMode,
    shell_mode: bool,
    query: ChatWrapper,
    save_path: Option<String>,
    program_out_file: Option<String>,
}

impl<'a> Shellm<'a> {
    pub fn new(
        query: Option<&str>,
        model_mode: ModelMode,
        shell_mode: bool,
        load_session: Option<String>,
        save_path: Option<String>,
        program_out_file: Option<String>,
        container: &'a ModelContainer,
        ctx_window: u32,
    ) -> Result<Self, ShellCreationError> {
        let threads = Some((get_sys_threads() / 2) as i32);
        let instance = if let Some(load_path) = load_session {
            match ModelInstance::load_from_session(
                container,
                threads,
                None,
                ctx_window,
                load_path.clone(),
            ) {
                Ok(instance) => instance,
                Err(_e) => {
                    eprintln!("Could not load session {}!", load_path);
                    return Err(ShellCreationError);
                }
            }
        } else {
            ModelInstance::new(container, threads, None, ctx_window)
        };

        let mut init_query = ChatWrapper::new();
        init_query.add_dialogue(ChatRole::System, model_mode.get_system_prompt());

        if let Some(query) = query {
            let mut query = query.to_string();

            query = match model_mode {
                ModelMode::CMD => Self::augment_query(query),
                _ => query
            };

            println!("{}", query);

            init_query.add_dialogue(ChatRole::User, &query);
        }

        Ok(Shellm {
            instance,
            model_mode,
            shell_mode,
            query: init_query,
            save_path,
            program_out_file,
        })
    }

    fn print_shell_start_msg() {
        println!(
            "{}",
            colorify(" ____  _  _  ____  __    __    _  _ ",
                     129.,
                     59.,
                     235.)
        );
        println!(
            "{}",
            colorify("/ ___)/ )( \\(  __)(  )  (  )  ( \\/ )",  201., 168., 255.)
        );
        println!(
            "{}",
            colorify(
                "\\___ \\) __ ( ) _) / (_/\\/ (_/\\/ \\/ \\",
                129.,
                59.,
                235.
            )
        );
        println!(
            "{}",
            colorify("(____/\\_)(_/(____)\\____/\\____/\\_)(_/",  201., 168., 255.)
        );
    }

    pub fn process_query(&mut self) -> String {
        let model_status = ModelStatus(false);
        let state = Arc::new(Mutex::new(model_status));
        self.loading_indicator(Arc::clone(&state));

        let toks = self
            .instance
            .chat_query(&self.query, 500, false, false)
            .unwrap();
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

    fn get_wd() -> String {
        env::current_dir().unwrap().display().to_string()
    }

    fn get_files() -> Result<String, Box<dyn Error>> {
        let mut entries = Vec::new();

        let dir_entries = fs::read_dir(Self::get_wd())?;
        for entry_result in dir_entries {
            let entry = entry_result?;
            let path = entry.path();

            let name = path.file_name()
                .and_then(|n| n.to_str())
                .ok_or("Invalid file name")?
                .to_string();

            if path.is_dir() {
                entries.push(format!("{}/", name));
            } else {
                entries.push(name);
            }
        }

        entries.sort();
        Ok(entries.join(", "))
    }

    fn augment_query(mut query: String) -> String {
        query.push_str(" WD: ");
        query.push_str(&Self::get_wd());
        query.push_str(" FILES: ");
        query.push_str(&Self::get_files().unwrap());
        query

    }

    fn exit_shell(&self) {
        let save = self.save_path.clone();
        if let Some(save_path) = save {
            self.instance
                .save_curr_session(Some(save_path))
                .expect("Could not save session!");
        }
        println!("{}", colorify("🔮 Exiting ✨", 201., 168., 255.))
    }

    fn loading_indicator(&self, model_status: Arc<Mutex<ModelStatus>>) {
        thread::spawn(move || {
            animate_text(
                "✨ ─────── running magik ─────── ✨",
                -0.009,
                || {
                    let status = {
                        let model_status_lock = model_status.lock().unwrap();
                        !model_status_lock.0
                    };
                    status
                },
            )
        });
    }

    fn run_shell(&mut self) {
        let shell_tag = colorify("🔮", 129., 59., 235.);
        let tilda = colorify("~", 59., 150., 235.);
        loop {
            // Self::clear_stdin();
            let mut buffer = String::new();
            if self.query.len() != 2 {
                print!("{} {} ", shell_tag, tilda);
                std::io::stdout().flush().unwrap(); // flush to stdout
                std::io::stdin().read_line(&mut buffer).unwrap();

                if buffer == "exit\n" {
                    self.exit_shell();
                    break;
                }

                buffer = match self.model_mode {
                    ModelMode::CMD => Self::augment_query(buffer),
                    _ => buffer
                };

                self.query.add_dialogue(ChatRole::User, &buffer);
            }

            let result = self.process_query();

            match self.model_mode {
                ModelMode::CMD => Self::exec_bash_cmd(result),
                _ => {}
            }

            self.query.clear();
        }
    }

    pub fn run(&mut self) {
        if self.shell_mode {
            Self::print_shell_start_msg();
            self.run_shell();
        } else {
            if self.query.len() < 2 {
                eprintln!("{}", colorify("No query provided", 247., 89., 89.));
                return;
            }

            let result = self.process_query();
            match self.model_mode {
                ModelMode::CMD => Self::exec_bash_cmd(result),
                _ => {}
            }
            // TODO check if the curr mode is CMD or what not you
        }
    }
}
