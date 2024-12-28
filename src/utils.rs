pub mod model_tool;


pub mod utils {

    pub fn get_sys_threads() -> usize {
        num_cpus::get()
    }

}