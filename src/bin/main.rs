use clap::{arg, Parser};
use shellm::shell::shell_tools::{ModelMode, Shellm};
use shellm::utils::model_tool::{ChatRole, ChatWrapper, ModelContainer, ModelInstance};

#[derive(Parser, Debug)]
#[command(name = "shellm", about, long_about = None)]
struct Args {
    /// shellm query
    #[arg(short, long)]
    query: Option<String>,

    /// max number of tokens to generate
    #[arg(long, default_value="10000", value_name="LENGTH")]
    max: i32,

    /// shellm will produce bash commands for you
    #[arg(short, long)]
    bash: bool,

    /// shellm will produce code for you
    #[arg(short, long)]
    code: bool,

    /// shellm will help you with math questions
    #[arg(short, long)]
    math: bool,

    /// shellm will help you with writing based questions
    #[arg(short, long)]
    writing: bool,

    /// shellm will answer general questions/request
    #[arg(short, long)]
    general: bool,

    /// enter shellm environment
    #[arg(short, long)]
    shell: bool,

    /// load from a past session
    #[arg(long, value_name = "NAME")]
    load: Option<String>,

    /// save the current session
    #[arg(long, value_name = "NAME")]
    save: Option<String>,

    /// when in coding mode, will save generated code to file of <NAME>
    #[arg(short, long, value_name = "NAME")]
    prog_out: Option<String>
}

fn main() {
    let arguments = Args::parse();

    let model_mode = if arguments.bash {
        ModelMode::CMD
    } else if arguments.code {
        ModelMode::CODE
    } else if arguments.ma {
        ModelMode::MATH
    } else if arguments.writing {
        ModelMode::WRITING
    } else {
        ModelMode::GENERAL
    };

    let container = ModelContainer::new(
        "/home/v18/Documents/Code/ml/gguf_models/qwen2.5-coder-7b-instruct-q4_k_m.gguf",
    );

    let max_gen = if arguments.max < 30000 { arguments.max } else { 30000 };

    let mut shellm = Shellm::new(
        arguments.query,
        model_mode,
        max_gen,
        arguments.shell,
        arguments.load,
        arguments.save,
        arguments.prog_out,
        &container,
        30000,
    )
    .unwrap();
    shellm.run();
}