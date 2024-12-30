use crate::utils::color::{animate_text, colorify};
use crate::utils::model_tool::{ChatRole, ChatWrapper, ModelContainer, ModelInstance};
use crate::utils::utils::get_sys_threads;
use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs, thread};

pub enum ModelMode {
    CMD,
    CODE,
    MATH,
    WRITING,
    GENERAL,
}

impl ModelMode {
    fn get_system_prompt(&self) -> &str {
        match *self {
            ModelMode::CMD => "You are a bash command generator assistant for a linux systems. Output only raw bash commands without any explanations, markdown formatting, code blocks, or backticks - each response should be immediately executable in a terminal. Chain multiple commands with && when steps need to be sequential, use ; for independent commands that can run in any order, and default to absolute paths unless working directory is specified. Prefer single-line solutions over multiple lines when possible, using proper command escaping and quoting when needed. When provided, context will appear as 'WD: {path} FILES: {file1, file2, ...}' - use this information only when relevant to command construction. For directory-wide operations, use '.' instead of iterating through files, and respect the current working directory when provided. When details are missing, choose the most common/logical default options, use sudo when operations require elevated privileges, prefer widely available core utilities over optional packages, and include necessary package installation commands if specialized tools are required. Include basic error checking in critical operations, use -e flag with shell commands when appropriate, and add safeguards for destructive operations. Example context format: WD: /home/user/documents FILES: report.pdf, notes.txt, images/",
            ModelMode::CODE => "
You are a highly intelligent and capable coding assistant. Your responses must contain only code, with no surrounding text, explanations, or formatting. Follow these rules strictly:
1. Provide only working, executable code that directly solves the user's request
2. Include helpful comments within the code to explain key functionality and important logic
3. If no programming language is specified, use Python as the default language unless another language would clearly be more appropriate for the task (e.g., JavaScript for frontend web functionality, SQL for database queries)
4. Do not add any text before or after the code
5. Do not include markdown formatting or code block markers
6. Do not ask follow-up questions or provide additional explanations
7. Structure your code following best practices for readability and maintainability
8. Include error handling where appropriate
9. Use clear, descriptive variable names and consistent formatting
10. If multiple files are needed, separate them with a single line containing the filename in comments
11. Begin coding immediately when receiving a request, with no preamble
12. End your response as soon as the code is complete, with no concluding remarks
13. MAKE SURE TO NOT provide markdown formatting such as ```.",
            ModelMode::MATH => "",
            ModelMode::WRITING => "",
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
        query: Option<String>,
        model_mode: ModelMode,
        shell_mode: bool,
        load_session: Option<String>,
        save_path: Option<String>,
        program_out_file: Option<String>,
        container: &'a ModelContainer,
        ctx_window: u32,
    ) -> Result<Self, ShellCreationError> {
        let threads = Some((get_sys_threads() * 3 / 4) as i32);
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

        if let Some(mut query) = query {
            query = match model_mode {
                ModelMode::CMD => Self::augment_query(query),
                _ => query,
            };
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
            colorify(" ____  _  _  ____  __    __    _  _ ", 129., 59., 235.)
        );
        println!(
            "{}",
            colorify("/ ___)/ )( \\(  __)(  )  (  )  ( \\/ )", 201., 168., 255.)
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
            colorify("(____/\\_)(_/(____)\\____/\\____/\\_)(_/", 201., 168., 255.)
        );
        println!();
    }

    fn process_query(&mut self) -> String {
        let model_status = ModelStatus(false);
        let state = Arc::new(Mutex::new(model_status));
        self.loading_indicator(Arc::clone(&state));

        let toks = self
            .instance
            .chat_query(&self.query, 50000, false, false, ||{})
            .unwrap();
        let result = self.instance.decode_tokens(toks, false);

        state.lock().unwrap().0 = true;
        sleep(Duration::from_millis(50));

        result
    }

    fn stream_query(&mut self) -> () {
        let model_status = ModelStatus(false);
        let state = Arc::new(Mutex::new(model_status));
        self.loading_indicator(Arc::clone(&state));

        let toks = self
            .instance
            .chat_query(&self.query, 50000, true, true, move || { state.clone().lock().unwrap().0 = true; });

/*        let result = self.instance.decode_tokens(toks, false);

        result*/
    }

    fn exec_bash_cmd(cmd: String) {
        let mut output = String::new();

        output.push_str(&format!("{}\n", colorify("Generated command:", 150., 150., 150.)));
        output.push_str("\n");
        output.push_str(&format!("      {}\n", colorify(&cmd, 59., 235., 115.)));
        output.push_str("\n");
        output.push_str(&format!("{}", colorify("Cannot guarantee that the command is 'safe.'\nVerify the command if you're uncertain.\n", 150., 150., 150.)));

        let split = output.split(" ");
        let len = split.clone().count();
        for (i, w) in split.enumerate() {
            if i != len - 1 {
                print!("{} ", w);
            } else {
                print!("{}", w);
            }
            std::io::stdout().flush().unwrap();
            if !w.trim().is_empty() {
                sleep(Duration::from_millis(125));
            }
        }

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
                eprintln!(
                    "{}",
                    colorify("Command could not execute successfully", 247., 89., 89.)
                );
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

            let name = path
                .file_name()
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
        println!("{}", colorify("🔮 Bye", 201., 168., 255.))
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
            let mut buffer = String::new();
            if self.query.len() != 2 {
                print!("{} {} ", shell_tag, tilda);
                std::io::stdout().flush().unwrap(); // flush to stdout
                std::io::stdin().read_line(&mut buffer).unwrap();

                if buffer == "exit\n" {
                    self.exit_shell();
                    break;
                } else if buffer.is_empty() || buffer.trim().is_empty() {
                    continue;
                }

                buffer = match self.model_mode {
                    ModelMode::CMD => Self::augment_query(buffer),
                    _ => buffer,
                };

                self.query.add_dialogue(ChatRole::User, &buffer);
            }


            match self.model_mode {
                ModelMode::CMD => {
                    let result = self.process_query();
                    Self::exec_bash_cmd(result)
                },
                ModelMode::CODE =>  {
                    self.stream_query();
                },
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
            // process a single query
            if self.query.len() < 2 {
                eprintln!("{}", colorify("No query provided", 247., 89., 89.));
                return;
            }

            match self.model_mode {
                ModelMode::CMD =>  {
                    let result = self.process_query();
                    Self::exec_bash_cmd(result)
                },
                ModelMode::CODE => {
                    self.stream_query();
                }
                _ => {}
            }
            // TODO check if the curr mode is CMD or what not you
        }
    }
}
