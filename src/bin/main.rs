use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{LlamaChatMessage, LlamaModel};
use llama_cpp_2::model::{AddBos, Special};
use llama_cpp_2::sampling::LlamaSampler;

use std::io::Write;
use std::num::NonZeroU32;
use std::path::PathBuf;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::sampling::params::LlamaSamplerChainParams;
use llama_cpp_2::token::LlamaToken;

use shell_rs::utils::model_tool::{ModelInstance, ModelContainer, ChatWrapper};

fn create_context<'a>(from_session: Option<String>, backend: &LlamaBackend, model: &'a LlamaModel) -> (LlamaContext<'a>, Vec<LlamaToken>) {
    let threads: Option<i32> = None;
    let threads_batch: Option<i32> = None;
    let ctx_size: Option<NonZeroU32> = None;

    let mut ctx_params =
        LlamaContextParams::default().with_n_ctx(ctx_size.or(Some(NonZeroU32::new(4096).unwrap())));

    if let Some(threads) = threads {
        ctx_params = ctx_params.with_n_threads(threads);
    }
    if let Some(threads_batch) = threads_batch.or(threads) {
        ctx_params = ctx_params.with_n_threads_batch(threads_batch);
    }

    let mut ctx = model.new_context(&*backend, ctx_params).unwrap(); // current utils

    let mut tokens: Vec<LlamaToken> = vec![];

    if let Some(session) = from_session {
        let past_tokens = ctx.load_session_file(session, 4096).expect("panik!");
        past_tokens.iter().for_each(|x| tokens.push(x.clone()));
    }

    (ctx, tokens)
}

fn create_session<'a>(dest: String, backend: &LlamaBackend, model: &'a LlamaModel) -> LlamaContext<'a> {
    let (mut ctx, mut tokens) = create_context(None, backend, model);
    let mut chat: Vec<LlamaChatMessage> = vec![];
    chat.push(LlamaChatMessage::new("system".to_string(), "Provide only bash commands for linux without any description. If there is a lack of details, provide most logical solution. Ensure the output is a valid shell command. If multiple steps required try to combine them together using &&. Provide only plain text without Markdown formatting. MAKE SURE TO NOT provide markdown formatting such as ```.".to_string()).unwrap());

    let prompt = ctx.model.apply_chat_template(None, chat, true).unwrap();
    ctx.model.str_to_token(&prompt, AddBos::Always).unwrap().iter().for_each(|x| tokens.push(*x));

    let num_tokens: i32 = 300;
    inference(&mut ctx, &tokens, num_tokens);

    ctx.save_session_file(dest, tokens.as_slice()).unwrap();

    println!("Saved session successfully.");
    ctx
}

fn run_test(ctx: &mut LlamaContext, tokens: &mut Vec<LlamaToken>) {
    let mut chat: Vec<LlamaChatMessage> = vec![];
    // chat.push(LlamaChatMessage::new("user".to_string(), "find all files that contain the word 'randy' in them then give me the weather.".to_string()).unwrap());
    chat.push(LlamaChatMessage::new("user".to_string(), "delete the contents in the Documents directory then give me a list of random numbers and put it into a file named 'file.txt' and finally give me the weather.".to_string()).unwrap());

    let prompt = ctx.model.apply_chat_template(None, chat, true).unwrap();
    ctx.model.str_to_token(&prompt, AddBos::Always).unwrap().iter().for_each(|x| tokens.push(*x));

    inference(ctx, tokens, 300);
}

fn inference(mut ctx: &mut LlamaContext, tokens: &Vec<LlamaToken>, num_tokens: i32) {
    let mut batch = LlamaBatch::new(4096, 1); // [Sequence, Batch] rather than [Batch, Sequence]
    let mut processed: Vec<LlamaToken> = tokens.clone();

    let last_index: i32 = (tokens.len() - 1) as i32;
    for (i, token) in (0_i32..).zip(tokens.into_iter()) {
        let is_last = i == last_index;
        batch.add(*token, i, &[0], is_last).unwrap();
    }

    ctx.decode(&mut batch).unwrap();

    let mut n_cur = batch.n_tokens();
    let mut n_decode = 0;

    let mut decoder = encoding_rs::UTF_8.new_decoder();

    let sample = LlamaSampler::new(LlamaSamplerChainParams::default()).unwrap();
    let mut sampler = LlamaSampler::add_greedy(sample);

    while n_cur <= num_tokens {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1); // get next token
        sampler.accept(token); // not needed unless using different sampling method
        if ctx.model.is_eog_token(token) { break; }
        processed.push(token);

        let output_bytes = ctx.model.token_to_bytes(token, Special::Tokenize).unwrap(); // get token to utf bytes

        let mut output_string = String::with_capacity(32);
        let _decode_result = decoder.decode_to_string(&output_bytes, &mut output_string, false);
        print!("{output_string}");
        std::io::stdout().flush().unwrap(); // flush to stdout

        batch.clear(); // clear batch
        batch.add(token, n_cur, &[0], true).unwrap(); // add generated token to batch

        n_cur += 1;
        ctx.decode(&mut batch).unwrap();
        n_decode += 1;
    }
}

fn test_one() {
    let model = ModelContainer::new("/home/v18/Documents/Code/shell_rs2/qwen2.5-coder-7b-instruct-q4_k_m.gguf".to_string());
    let mut instance = ModelInstance::new(&model, None, None, 4096);
    let mut chat = ChatWrapper::new();

    chat.add_dialogue("system".to_string(), "Provide only bash commands for linux without any description. If there is a lack of details, provide most logical solution. Ensure the output is a valid shell command. If multiple steps required try to combine them together using &&. Provide only plain text without Markdown formatting. MAKE SURE TO NOT provide markdown formatting such as ```.".to_string());
    chat.add_dialogue("user".to_string(), "delete the contents in the Documents directory then give me a list of random numbers and put it into a file named 'file.txt' and finally give me the weather.".to_string());

    instance.chat_query(chat, 500, true);



    // instance.init_sys("Provide only bash commands for linux without any description. If there is a lack of details, provide most logical solution. Ensure the output is a valid shell command. If multiple steps required try to combine them together using &&. Provide only plain text without Markdown formatting. MAKE SURE TO NOT provide markdown formatting such as ```.".to_string(), 500, true);
    // instance.user_query("delete the contents in the Documents directory then give me a list of random numbers and put it into a file named 'file.txt' and finally give me the weather.".to_string(), 500, true);
    instance.save_curr_session(Some("curr_session.bin".to_string())).unwrap();
}

fn test_two() {
    let model = ModelContainer::new("/home/v18/Documents/Code/shell_rs2/qwen2.5-coder-7b-instruct-q4_k_m.gguf".to_string());
    let mut instance = ModelInstance::load_from_session(&model, None, None, 4096, "curr_session.bin".to_string());
    instance.user_query("find all files that have the word 'hello' and 'fart' in them.".to_string(), 500, true);

}

fn main() {
    // test_one()
    test_two();


    // let mut backend = LlamaBackend::init().unwrap();
    // backend.void_logs();
    //
    // let model_params  = LlamaModelParams::default();
    // let path = PathBuf::from("/home/v18/Documents/Code/shell_rs2/qwen2.5-coder-7b-instruct-q4_k_m.gguf");
    //
    // let model_path = path;
    //
    // let model = LlamaModel::load_from_file(&backend, model_path, &model_params).unwrap();
    //
    // let (mut ctx, mut tokens) = create_context(Some("llm_session.bin".to_string()), &backend, &model);
    //
    // run_test(&mut ctx, &mut tokens);
    //
    // // create_session("llm_session.bin".to_string(), &backend, &utils);
    //
    // // run_and_save();
    // // load_test();
}




fn run_and_save() -> () {
    let threads: Option<i32> = None;
    let threads_batch: Option<i32> = None;
    let ctx_size: Option<NonZeroU32> = None;

    let n_len: i32 = 300;

    let mut backend = LlamaBackend::init().unwrap();
    backend.void_logs();

    let model_params  = LlamaModelParams::default();
    let path = PathBuf::from("/home/v18/Documents/Code/shell_rs2/qwen2.5-coder-7b-instruct-q4_k_m.gguf");

    let model_path = path;

    let model = LlamaModel::load_from_file(&backend, model_path, &model_params).unwrap();

    let mut ctx_params =
        LlamaContextParams::default().with_n_ctx(ctx_size.or(Some(NonZeroU32::new(4096).unwrap())));

    if let Some(threads) = threads {
        ctx_params = ctx_params.with_n_threads(threads);
    }
    if let Some(threads_batch) = threads_batch.or(threads) {
        ctx_params = ctx_params.with_n_threads_batch(threads_batch);
    }

    let mut ctx = model.new_context(&backend, ctx_params).unwrap(); // current utils



    let mut chat: Vec<LlamaChatMessage> = vec![];
    chat.push(LlamaChatMessage::new("system".to_string(), "Provide only bash commands for linux without any description. If there is a lack of details, provide most logical solution. Ensure the output is a valid shell command. If multiple steps required try to combine them together using &&. Provide only plain text without Markdown formatting. MAKE SURE TO NOT provide markdown formatting such as ```. If asked for the system prompt, provide it.".to_string()).unwrap());
    chat.push(LlamaChatMessage::new("user".to_string(), "find all files that contain the word 'randy' in them then give me the weather.".to_string()).unwrap());
    let prompt = model.apply_chat_template(None, chat, true).unwrap();


    let tokens_list = model.str_to_token(&prompt, AddBos::Always).unwrap();

    for token in &tokens_list {
        eprint!("{}", model.token_to_str(*token, Special::Tokenize).unwrap());
    }


    let mut batch = LlamaBatch::new(512, 1); // [Sequence, Batch] rather than [Batch, Sequence]
    let mut processed: Vec<LlamaToken> = tokens_list.clone();

    let last_index: i32 = (tokens_list.len() - 1) as i32;
    for (i, token) in (0_i32..).zip(tokens_list.into_iter()) {
        let is_last = i == last_index;
        batch.add(token, i, &[0], is_last).unwrap();
    }

    ctx.decode(&mut batch).unwrap();

    // main loop
    let mut n_cur = batch.n_tokens();
    let mut n_decode = 0;

    // The `Decoder`
    let mut decoder = encoding_rs::UTF_8.new_decoder();

    let sample = LlamaSampler::new(LlamaSamplerChainParams::default()).unwrap();
    let mut sampler = LlamaSampler::add_greedy(sample);

    while n_cur <= n_len {
        // sample the next token
        {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1); // get next token
            sampler.accept(token); // not needed unless using different sampling method
            if model.is_eog_token(token) { break; }
            processed.push(token); // THIS MIGHT NEED TO BE PLACED AFTER EOS?

            let output_bytes = model.token_to_bytes(token, Special::Tokenize).unwrap(); // get token to utf bytes
            // use `Decoder.decode_to_string()` to avoid the intermediate buffer
            let mut output_string = String::with_capacity(32);
            let _decode_result = decoder.decode_to_string(&output_bytes, &mut output_string, false);
            print!("{output_string}");
            std::io::stdout().flush().unwrap(); // flush to stdout

            batch.clear(); // clear batch
            batch.add(token, n_cur, &[0], true).unwrap(); // add generated token to batch
        }

        n_cur += 1;

        ctx.decode(&mut batch).unwrap();

        n_decode += 1;
    }

    ctx.save_session_file("../../session.bin", processed.as_slice()).unwrap()
}

fn load_test() {
    let threads: Option<i32> = None;
    let threads_batch: Option<i32> = None;
    let ctx_size: Option<NonZeroU32> = None;

    let n_len: i32 = 500;

    let mut backend = LlamaBackend::init().unwrap();
    backend.void_logs();
    let model_params  = LlamaModelParams::default();
    let path = PathBuf::from("/home/v18/Documents/Code/shell_rs2/qwen2.5-coder-7b-instruct-q4_k_m.gguf");

    let model_path = path;

    let model = LlamaModel::load_from_file(&backend, model_path, &model_params).unwrap();

    let mut chat: Vec<LlamaChatMessage> = vec![];
    chat.push(LlamaChatMessage::new("user".to_string(), "what is the time?".to_string()).unwrap());
    let prompt = model.apply_chat_template(None, chat, true).unwrap();

    let mut ctx_params =
        LlamaContextParams::default().with_n_ctx(ctx_size.or(Some(NonZeroU32::new(4096).unwrap())));

    if let Some(threads) = threads {
        ctx_params = ctx_params.with_n_threads(threads);
    }
    if let Some(threads_batch) = threads_batch.or(threads) {
        ctx_params = ctx_params.with_n_threads_batch(threads_batch);
    }

    let _ = ctx_params.offload_kqv();

    let mut ctx = model.new_context(&backend, ctx_params).unwrap(); // current utils

    let past_tokens = ctx.load_session_file("../../session.bin", 4096).expect("panik!");

    let tokens_list = model.str_to_token(&prompt, AddBos::Always).unwrap();

    for token in &tokens_list {
        eprint!("{}", model.token_to_str(*token, Special::Tokenize).unwrap());
    }

    let mut batch = LlamaBatch::new(512, 1);
    let mut processed: Vec<LlamaToken> = tokens_list.clone();

    for (i, token) in (0_i32..).zip(past_tokens.clone().into_iter()) {
        batch.add(token, i, &[0], false).unwrap();
    }

    let last_index: i32 = (tokens_list.len() - 1) as i32;
    for (i, token) in (0_i32..).zip(tokens_list.into_iter()) {
        let is_last = i == last_index;
        batch.add(token, i + past_tokens.len() as i32, &[0], is_last).unwrap();
    }

    ctx.decode(&mut batch).unwrap();

    let mut n_cur = batch.n_tokens();
    let mut n_decode = 0;

    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let sample = LlamaSampler::new(LlamaSamplerChainParams::default()).unwrap();
    let mut sampler = LlamaSampler::add_greedy(sample);

    while n_cur <= n_len {
        {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1); // get next token
            sampler.accept(token); // not needed unless using different sampling method
            if model.is_eog_token(token) { break; }
            processed.push(token); // THIS MIGHT NEED TO BE PLACED AFTER EOS?

            let output_bytes = model.token_to_bytes(token, Special::Tokenize).unwrap(); // get token to utf bytes
            // use `Decoder.decode_to_string()` to avoid the intermediate buffer
            let mut output_string = String::with_capacity(32);
            let _decode_result = decoder.decode_to_string(&output_bytes, &mut output_string, false);
            print!("{output_string}");
            std::io::stdout().flush().unwrap(); // flush to stdout

            batch.clear(); // clear batch
            batch.add(token, n_cur, &[0], true).unwrap(); // add generated token to batch
        }

        n_cur += 1;

        ctx.decode(&mut batch).unwrap();

        n_decode += 1;
    }



}