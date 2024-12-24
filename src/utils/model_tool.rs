use std::io::Write;
use std::num::NonZeroU32;
use std::path::PathBuf;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel, Special};
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::sampling::params::LlamaSamplerChainParams;
use llama_cpp_2::token::LlamaToken;

#[derive(Debug)]
pub struct SaveInstanceError;

pub struct ModelContainer {
    model: LlamaModel,
    backend: LlamaBackend
}

impl ModelContainer {

    pub fn new(model_path: String) -> Self {
        let mut backend = LlamaBackend::init().unwrap();
        backend.void_logs();

        let model_params  = LlamaModelParams::default();
        let model =
            LlamaModel::load_from_file(&backend, PathBuf::from(model_path), &model_params).unwrap();

        ModelContainer {
            model,
            backend
        }
    }

}

pub struct ChatWrapper {
    chat: Vec<LlamaChatMessage>
}

impl ChatWrapper {
    pub fn new() -> Self {
        ChatWrapper { chat: vec![] }
    }

    pub fn add_dialogue(&mut self, role: String, content: String) {
        self.chat.push(LlamaChatMessage::new(role, content).unwrap());
    }

    pub fn to_tokens(&self, ctx: &LlamaContext) -> Vec<LlamaToken> {
        let prompt = ctx.model.apply_chat_template(None, self.chat.clone(), true).unwrap();
        ctx.model.str_to_token(&prompt, AddBos::Always).unwrap()
    }

}

pub struct ModelInstance<'a> {
    ctx_window: u32,
    ctx: LlamaContext<'a>,
    tokens: Vec<LlamaToken>,
}

impl <'a>ModelInstance<'a> {
    pub fn new(model_storage: &'a ModelContainer,
               threads: Option<i32>,
               threads_batch: Option<i32>,
               ctx_window: u32) -> Self {


        let mut ctx_params =
            LlamaContextParams::default().with_n_ctx(Some(NonZeroU32::new(ctx_window).unwrap()));

        if let Some(threads) = threads {
            ctx_params = ctx_params.with_n_threads(threads);
        }
        if let Some(threads_batch) = threads_batch.or(threads) {
            ctx_params = ctx_params.with_n_threads_batch(threads_batch);
        }

        let ctx = model_storage.model.new_context(&model_storage.backend, ctx_params).unwrap(); // current utils
        ModelInstance {
            ctx_window,
            ctx,
            tokens: vec![]
        }
    }

    fn create_chat_dialogue(&self, chat: Vec<LlamaChatMessage>) -> Vec<LlamaToken> {
        let prompt = self.ctx.model.apply_chat_template(None, chat, true).unwrap();
        self.ctx.model.str_to_token(&prompt, AddBos::Always).unwrap()
    }

    pub fn load_from_session(
        model_storage: &'a ModelContainer,
        threads: Option<i32>,
        threads_batch: Option<i32>,
        ctx_window: u32,
        session_path: String) -> Self {
        let mut model_instance = ModelInstance::new(model_storage, threads, threads_batch, ctx_window);
        let past_tokens = model_instance.ctx.load_session_file(session_path, ctx_window as usize).expect("panik!");
        past_tokens.iter().for_each(|x| model_instance.tokens.push(x.clone()));

        model_instance
    }

    pub fn save_curr_session(&self, dest: Option<String>) -> Result<(), SaveInstanceError> {
        let path = if let Some(path) = dest { path } else { "session.bin".to_string() };

        match self.ctx.save_session_file(path, self.tokens.as_slice()) {
            Err(e) => Err(SaveInstanceError),
            _ => Ok(())
        }
    }

    fn stream_tokens(&mut self, tokens: &Vec<LlamaToken>) {
        tokens.iter().for_each(|x| self.tokens.push(*x));
    }

    fn decode_tokens(&self, tokens: Vec<LlamaToken>) -> String {
        let mut decoded: String = "".to_owned();
        let mut decoder = encoding_rs::UTF_8.new_decoder();

        for token in tokens {
            let output_bytes = self.ctx.model.token_to_bytes(token, Special::Tokenize).unwrap(); // get token to utf bytes

            let mut output_string = String::with_capacity(32);
            let _decode_result = decoder.decode_to_string(&output_bytes, &mut output_string, false);

            decoded.push_str(output_string.as_str());

            std::io::stdout().flush().unwrap(); // flush to stdout
        }

        decoded
    }
    pub fn init_sys(&mut self, content: String, max_gen: i32, output: bool) -> Option<Vec<LlamaToken>> {
        let mut chat: Vec<LlamaChatMessage> = vec![];
        chat.push(LlamaChatMessage::new("system".to_string(), content).unwrap());

        if output {
            self.print_inference(self.create_chat_dialogue(chat), max_gen);
            None
        } else {
            Some(self.inference(self.create_chat_dialogue(chat), max_gen))
        }
    }

    pub fn user_query(&mut self, content: String, max_gen: i32, output: bool) -> Option<Vec<LlamaToken>> {
        let mut chat: Vec<LlamaChatMessage> = vec![];
        chat.push(LlamaChatMessage::new("user".to_string(), content).unwrap());

        if output {
            self.print_inference(self.create_chat_dialogue(chat), max_gen);
            None
        } else {
            Some(self.inference(self.create_chat_dialogue(chat), max_gen))
        }
    }

    pub fn chat_query(&mut self, chat: ChatWrapper, max_gen: i32, output: bool) -> Option<Vec<LlamaToken>> {
        if output {
            self.print_inference(chat.to_tokens(&self.ctx), max_gen);
            None
        } else {
            Some(self.inference(chat.to_tokens(&self.ctx), max_gen))
        }
    }

    pub fn print_inference(&mut self, query: Vec<LlamaToken>, max_gen: i32) {
        let result = self.inference(query, max_gen);
        println!("{}", self.decode_tokens(result));
    }

    pub fn inference(&mut self, query: Vec<LlamaToken>, max_gen: i32) -> Vec<LlamaToken> {
        let mut result: Vec<LlamaToken> = vec![];
        self.stream_tokens(&query);

        let mut batch = LlamaBatch::new(self.ctx_window as usize, 1); // [S, B]

        let mut tokens: &Vec<LlamaToken> = &self.tokens;

        let last_index: i32 = (tokens.len() - 1) as i32;
        for (i, token) in (0_i32..).zip(tokens.into_iter()) {
            let is_last = i == last_index;
            batch.add(*token, i, &[0], is_last).unwrap();
        }

        self.ctx.decode(&mut batch).unwrap();

        let mut n_cur = batch.n_tokens();

        let sample = LlamaSampler::new(LlamaSamplerChainParams::default()).unwrap();
        let mut sampler = LlamaSampler::add_greedy(sample);

        while n_cur <= max_gen {
            let token = sampler.sample(&self.ctx, batch.n_tokens() - 1); // get next token
            sampler.accept(token); // not needed unless using different sampling method
            if self.ctx.model.is_eog_token(token) { break; }
            result.push(token);
            self.tokens.push(token);

            batch.clear(); // clear batch
            batch.add(token, n_cur, &[0], true).unwrap(); // add generated token to batch

            self.ctx.decode(&mut batch).unwrap();

            n_cur += 1;
        }

        result
    }

}