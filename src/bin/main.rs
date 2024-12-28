use shell_rs::utils::color::{animate_text};
use shell_rs::utils::model_tool::{ChatRole, ChatWrapper, ModelContainer, ModelInstance};
use shell_rs::utils::utils::get_sys_threads;

fn test_one() {
    let model = ModelContainer::new("/home/v18/Documents/Code/shell_rs2/qwen2.5-coder-7b-instruct-q4_k_m.gguf".to_string());
    let mut instance = ModelInstance::new(&model, None, None, 4096);
    let mut chat = ChatWrapper::new();

    chat.add_dialogue(ChatRole::System, "Provide only bash commands for linux without any description. If there is a lack of details, provide most logical solution. Ensure the output is a valid shell command. If multiple steps required try to combine them together using &&. Provide only plain text without Markdown formatting. MAKE SURE TO NOT provide markdown formatting such as ```.".to_string());
    chat.add_dialogue(ChatRole::User, "delete the contents in the Documents directory then give me a list of random numbers and put it into a file named 'file.txt' and finally give me the weather.".to_string());

    instance.chat_query(&chat, 500, true, true);
    instance.save_curr_session(Some("curr_session.bin".to_string())).unwrap();
}

fn test_two() {
    let model = ModelContainer::new("/home/v18/Documents/Code/shell_rs2/qwen2.5-coder-7b-instruct-q4_k_m.gguf".to_string());
    let mut instance = ModelInstance::load_from_session(&model, None, None, 4096, "curr_session.bin".to_string()).unwrap();
    instance.user_query("find all files that have the word 'hello' and 'bobby' in them.".to_string(), 500, true, true);

}

fn test_three() {
    let model = ModelContainer::new("/home/v18/Documents/Code/shell_rs2/qwen2.5-coder-7b-instruct-q4_k_m.gguf".to_string());
    let mut instance = ModelInstance::new(&model, Some(12),  None, 10000);

    let mut chat = ChatWrapper::new();
    chat.add_dialogue(ChatRole::System, "You are a highly capable and adaptive mathematics assistant designed to help users solve math-based problems effectively. Your primary goal is to provide clear, accurate, and concise solutions while fostering understanding of the underlying concepts. Approach every problem with a focus on clarity and precision, breaking down solutions into logical, easy-to-follow steps tailored to the user's level of expertise. Always verify your calculations and ensure your explanations are thorough yet accessible. Encourage users to ask follow-up questions, explore alternative methods, and deepen their understanding of the subject. Whether the problem involves basic arithmetic, advanced calculus, or abstract mathematical theory, provide guidance that is both technically correct and intuitively understandable, aiming to empower users to solve problems confidently on their own. Address the user by the name they provide them".to_string());
    chat.add_dialogue(ChatRole::User, "My name is Bobby Durk".to_string());
    instance.chat_query(&chat, 500, false, true);
    chat.clear();
    chat.add_dialogue(ChatRole::User, "explain a trip to france using emojis".to_string());
    instance.chat_query(&chat, 500, false, true);
}

fn main() {

    animate_text("██████████████████████████████████████████████████████████".to_string(), -0.009);

/*    println!("{}", get_sys_threads());
    let colored_text = rgb_to_ansi(1.0, 0.0, 0.5); // bright purple
    println!("{}Hello World{}", colored_text, "\x1b[0m"); // with reset code at the end*/


    /*    let model = ModelContainer::new("/home/v18/Documents/Code/shell_rs2/qwen2.5-coder-7b-instruct-q4_k_m.gguf".to_string());
    let mut instance = ModelInstance::new(&model, Some(12),  None, 10000);
    let mut chat = ChatWrapper::new();
    chat.add_dialogue(ChatRole::System, "You are a highly capable and adaptive mathematics assistant designed to help users solve math-based problems effectively. Your primary goal is to provide clear, accurate, and concise solutions while fostering understanding of the underlying concepts. Approach every problem with a focus on clarity and precision, breaking down solutions into logical, easy-to-follow steps tailored to the user's level of expertise. Always verify your calculations and ensure your explanations are thorough yet accessible. Encourage users to ask follow-up questions, explore alternative methods, and deepen their understanding of the subject. Whether the problem involves basic arithmetic, advanced calculus, or abstract mathematical theory, provide guidance that is both technically correct and intuitively understandable, aiming to empower users to solve problems confidently on their own. Address the user by the name they provide. KEEP ANSWERS CONCISE AND BRIEF.".to_string());

    loop {
        let mut buffer = String::new();
        print!("shellm> ");
        io::stdout().flush().unwrap(); // flush to stdout
        io::stdin().read_line(&mut buffer).unwrap();

        chat.add_dialogue(ChatRole::User, buffer);
        println!();
        println!("Processing query...");
        instance.chat_query(&chat, 5000, true, true);
        println!();
        println!();
        chat.clear();
    }*/
    // test_three();
}