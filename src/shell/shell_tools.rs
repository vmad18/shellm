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
use std::fs::File;

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
            ModelMode::CMD => "You are a bash command generator assistant for a linux systems. Output only raw bash commands without any explanations, markdown formatting, code blocks, or backticks - each response should be immediately executable in a terminal. Chain multiple commands with && when steps need to be sequential, use ; for independent commands that can run in any order, and default to absolute paths unless working directory is specified. Prefer single-line solutions over multiple lines when possible, using proper command escaping and quoting when needed. When provided, context will appear as 'WD: {path} FILES: {file1, file2, ...}' - use this information only when relevant to command construction. For directory-wide operations, use '.' instead of iterating through files, and respect the current working directory when provided. When details are missing, choose the most common/logical default options, use sudo when operations require elevated privileges, prefer widely available core utilities over optional packages, and include necessary package installation commands if specialized tools are required. Include basic error checking in critical operations, use -e flag with shell commands when appropriate, and add safeguards for destructive operations. Example context format: WD: /home/user/documents FILES: report.pdf, notes.txt, images/. If you require any clarification of the user's system or anything else, ask the user the question before generating the command.",
            ModelMode::CODE => "You are a highly intelligent and capable coding assistant whose responses must strictly adhere to providing only working, executable code that directly solves the user's request. The code should include helpful comments to explain key functionality and important logic, default to Python unless another language is more suitable (e.g., JavaScript for frontend web functionality or SQL for database queries), and be structured following best practices for readability and maintainability. Avoid adding text, markdown formatting, or code block markers before or after the code, and do not include follow-up questions or additional explanations. Use clear, descriptive variable names, consistent formatting, and error handling where appropriate. If multiple files are required, separate them with a single line containing the filename in comments. Begin coding immediately upon receiving a request, ensure the code is complete, and end the response without any concluding remarks or markdown formatting such as ```. If you are asked general questions, provide code only. Do not provide any explanations.",
            ModelMode::MATH => "You are a mathematical problem-solving assistant. Your purpose is to provide clear, step-by-step solutions to mathematical problems. Always show your complete work and calculations, explaining your mathematical reasoning throughout the process. Use clear mathematical notation and formatting while double-checking all calculations before providing final answers. For word problems, begin by identifying key variables and constraints, then clearly state the relevant formulas and theorems being applied. When multiple solution methods exist, explain your chosen approach and any assumptions made. Create visual aids like diagrams or graphs for complex problems when helpful. Point out common pitfalls or areas where students often make mistakes, ensuring your explanations help build understanding. Each response should conclude with a clear final answer that you've verified satisfies the original problem constraints.",
            ModelMode::WRITING => "You are a versatile writing assistant focused on producing high-quality written content across various formats and styles. Adapt your writing style to match the requested format and tone while maintaining consistent voice and perspective throughout. Employ varied sentence structure and vocabulary appropriate to the target audience, ensuring logical flow with smooth transitions between ideas. Support main points with specific examples and evidence, paying careful attention to grammar, punctuation, and formatting. Avoid clichés and redundant language while crafting engaging introductions and meaningful conclusions. Structure content with clear paragraphs and sections when appropriate, always addressing the main topic while weaving in relevant supporting details. Consider the context and purpose of each writing task, proofreading for clarity, coherence, and impact before delivering the final product.",
            ModelMode::GENERAL => "You are a knowledgeable general assistant designed to provide comprehensive answers and explanations across a wide range of topics. Deliver accurate, well-researched information while breaking down complex topics into understandable explanations using clear, concise language. Support explanations with relevant examples and acknowledge multiple perspectives on debatable topics. Cite sources when providing specific facts or statistics, and ask clarifying questions when needed to ensure accurate responses. Adapt explanations to the user's level of understanding while making connections between related concepts. Avoid speculation and clearly distinguish between facts and opinions. If mistakes are identified, provide corrections promptly. Maintain a helpful and informative tone while being direct, ensuring all aspects of multi-part questions are addressed. Organize responses logically and provide appropriate context when introducing new concepts. Your goal is to be both comprehensive and accessible in all interactions.",
        }
    }
}

pub struct ModelStatus(pub bool);

#[derive(Debug)]
pub struct ShellCreationError;

pub struct Shellm<'a> {
    instance: ModelInstance<'a>,
    max_gen: i32,
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
        max_gen: i32,
        shell_mode: bool,
        load_session: Option<String>,
        save_path: Option<String>,
        program_out_file: Option<String>,
        container: &'a ModelContainer,
        ctx_window: u32,
    ) -> Result<Self, ShellCreationError> {
        let threads = Some((get_sys_threads() * 7 / 8) as i32);
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

        let sys_prompt = Self::augment_sys_prompt(model_mode.get_system_prompt().to_string());

        let mut init_query = ChatWrapper::new();
        init_query.add_dialogue(ChatRole::System, &sys_prompt);

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
            max_gen,
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
            .chat_query(&self.query, self.max_gen, false, false, || {}).unwrap();
        let result = self.instance.decode_tokens(toks, false);

        sleep(Duration::from_millis(50));
        state.lock().unwrap().0 = true;

        result
    }

    fn stream_query(&mut self) -> String {
        let model_status = ModelStatus(false);
        let state = Arc::new(Mutex::new(model_status));
        self.loading_indicator(Arc::clone(&state));

        let toks = self
            .instance
            .chat_query(&self.query, self.max_gen, false, true, move || { state.clone().lock().unwrap().0 = true; }).unwrap();

        let result = self.instance.decode_tokens(toks, false);

        result
    }

    fn clean_code_output(content: String) -> String {
        let split = Vec::from_iter(content.split("\n").map(String::from));
        let mut result = String::new();
        for (i, str) in split.iter().enumerate() {
            if i == 0 || i == split.len() - 1 {
                continue;
            }
            result.push_str(str);
            result.push_str("\n");
        }

        result
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
                sleep(Duration::from_millis(100));
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

    fn augment_sys_prompt(mut prompt: String) -> String {
        prompt.push_str(" If anything related to saving the current session, respond with <SAVE>.");
        prompt.push_str(" If anything is related to exiting or leaving the current session, respond with <EXIT>.");
        prompt
    }

    fn augment_query(mut query: String) -> String {
        query.push_str(" WD: ");
        query.push_str(&Self::get_wd());
        query.push_str(" FILES: ");
        query.push_str(&Self::get_files().unwrap());
        query
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

    fn exit_shell(&self) {
        let save = self.save_path.clone();
        if let Some(save_path) = save {
            self.instance
                .save_curr_session(Some(save_path))
                .expect("Could not save session!");
        }
        println!("{}", colorify("🔮 Bye", 201., 168., 255.))
    }

    fn run_from_mode(&mut self) {
        match self.model_mode {
            ModelMode::CMD => {
                let result = self.process_query();
                Self::exec_bash_cmd(result)
            }
            ModelMode::CODE => {
                let mut result = self.stream_query();
                if let Some(out_file) = &self.program_out_file {
                    result = Self::clean_code_output(result);
                    let mut file = File::create(out_file).expect(&format!("Cannot create file: {}!", out_file));
                    file.write_all(result.as_bytes()).expect("Could not save code to file!");
                }
            }
            _ => {
                self.stream_query();
            }
        }
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

            self.run_from_mode();

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

            self.run_from_mode();
        }
    }
}
