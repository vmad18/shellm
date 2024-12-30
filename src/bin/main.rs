use clap::{arg, Parser};
use shellm::shell::shell_tools::{ModelMode, Shellm};
use shellm::utils::model_tool::{ChatRole, ChatWrapper, ModelContainer, ModelInstance};

#[derive(Parser, Debug)]
#[command(name = "shellm", about, long_about = None)]
struct Args {
    /// shellm query
    #[arg(short, long)]
    query: Option<String>,

    /// shellm will produce bash commands for you
    #[arg(short, long)]
    bash: bool,

    /// shellm will produce code for you
    #[arg(short, long)]
    code: bool,

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
}

fn main() {
    // Some("find all files that contain the word 'hello' in them")

    let arguments = Args::parse();

    let model_mode = if arguments.bash {
        ModelMode::CMD
    } else if arguments.code {
        ModelMode::CODE
    } else {
        ModelMode::GENERAL
    };

    let container = ModelContainer::new(
        "/home/v18/Documents/Code/ml/gguf_models/qwen2.5-coder-7b-instruct-q4_k_m.gguf",
    );
    let mut shellm = Shellm::new(
        arguments.query,
        model_mode,
        arguments.shell,
        arguments.load,
        arguments.save,
        None,
        &container,
        10000,
    )
    .unwrap();
    shellm.run();
}

// fn test_three() {
//     let model = ModelContainer::new(
//         "/home/v18/Documents/Code/shell_rs2/qwen2.5-coder-7b-instruct-q4_k_m.gguf",
//     );
//     let mut instance = ModelInstance::new(&model, Some(12), None, 50000);
//
//     let mut chat = ChatWrapper::new();
//     chat.add_dialogue(ChatRole::System, "You are a highly capable and adaptive mathematics assistant designed to help users solve math-based problems effectively. Your primary goal is to provide clear, accurate, and concise solutions while fostering understanding of the underlying concepts. Approach every problem with a focus on clarity and precision, breaking down solutions into logical, easy-to-follow steps tailored to the user's level of expertise. Always verify your calculations and ensure your explanations are thorough yet accessible. Encourage users to ask follow-up questions, explore alternative methods, and deepen their understanding of the subject. Whether the problem involves basic arithmetic, advanced calculus, or abstract mathematical theory, provide guidance that is both technically correct and intuitively understandable, aiming to empower users to solve problems confidently on their own. Address the user by the name they provide them");
//     chat.add_dialogue(ChatRole::User, "My name is Bobby Durk");
//     instance.chat_query(&chat, 500, false, true);
//     chat.clear();
//     chat.add_dialogue(ChatRole::User, "explain a trip to france using emojis");
//     instance.chat_query(&chat, 500, false, true);
// }
