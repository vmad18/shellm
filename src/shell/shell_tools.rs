use std::collections::LinkedList;
use crate::utils::model_tool::{ChatRole, ChatWrapper, ModelContainer, ModelInstance};
use crate::utils::utils::get_sys_threads;

pub enum ModelMode {
    CMD,
    CODE,
    GENERAL
}

impl ModelMode {

    fn get_system_prompt(&self) -> String {
        match *self {
            ModelMode::CMD => "".to_string(),
            ModelMode::CODE => "".to_string(),
            ModelMode::GENERAL => "".to_string()
        }
    }

}

pub struct ShellCreationError;

pub struct ShellLM<'a> {
    instance: ModelInstance<'a>,
    container: ModelContainer,
    model_mode: ModelMode,
    shell_mode: bool,
    init_query: ChatWrapper,
    save_path: Option<String>,
    program_out_file: Option<String>,
}

impl <'a> ShellLM<'a> {

    pub fn new(query: Option<String>,
               model_mode: ModelMode,
               shell_mode: bool,
               load_session: Option<String>,
               save_path: Option<String>,
               program_out_file: Option<String>,
               model_path: String,
               ctx_window: u32) -> Result<Self, ShellCreationError> {
        let container = ModelContainer::new(model_path);
        let threads = Some((get_sys_threads() / 2) as i32);
        let instance =
            if let Some(load_path) = load_session
            {
                match ModelInstance::load_from_session(&container, threads, None, ctx_window, load_path) {
                    Ok(instance) => instance,
                    Err(_e) => {
                        eprintln!("Could not load session {}!", load_path);
                        return Err(ShellCreationError)
                    }
                }
            } else { ModelInstance::new(&container, threads, None, ctx_window) };

        let mut init_query = ChatWrapper::new();
        init_query.add_dialogue(ChatRole::System, model_mode.get_system_prompt());

        if let Some(query) = query {
            init_query.add_dialogue(ChatRole::User, query);
        }

        Ok(ShellLM { instance, container, model_mode, shell_mode, init_query, save_path, program_out_file } )
    }

    pub fn process_cmd(&self) -> String {

    }

    pub fn exec_bash_cmd(&self) -> String {

    }

    pub fn get_files(&self) -> String {

    }

    pub fn get_wd(&self) -> String {

    }

    pub fn run_shell(&self) {
        loop {

        }
    }

    pub fn run(&self) {

    }

}