pub mod model_tool;

pub mod color {
    use std::io::Write;
    use std::thread::sleep;
    use std::time::Duration;

    const CLEAR_LINE: &str = "\x1b[2K\x1b[G";
    const RESET_COLOR: &str = "\x1b[0m";
    const HIDE_CURSOR: &str = "\x1b[?25l";
    const SHOW_CURSOR: &str = "\x1b[?25h";
    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
        let h = h - h.floor();

        let c = v * s;
        let h_prime = h * 6.0;
        let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
        let m = v - c;

        let (r1, g1, b1) = match h_prime.floor() as i32 {
            0 => (c, x, 0.0),  // Red to yellow
            1 => (x, c, 0.0),  // Yellow to green
            2 => (0.0, c, x),  // Green to cyan
            3 => (0.0, x, c),  // Cyan to blue
            4 => (x, 0.0, c),  // Blue to magenta
            5 => (c, 0.0, x),  // Magenta to red
            _ => (0.0, 0.0, 0.0)
        };
        (r1 + m, g1 + m, b1 + m)
    }

    fn rgb_to_ansi(r: f32, g: f32, b: f32) -> String {
        format!("\x1b[38;2;{};{};{}m",
                (r * 255.0) as i32,
                (g * 255.0) as i32,
                (b * 255.0) as i32
        )
    }

    pub fn colorify(content: &str, r: f32, g: f32, b: f32) -> String {
        format!("{}{}{}", rgb_to_ansi(r / 255., g / 255., b / 255.), content, RESET_COLOR)
    }

    pub fn color_gradient_text(content: &String, offset: f32) -> String {
        let mut result = String::new();
        let length = content.len() as f32;

        for (i, c) in content.chars().enumerate() {
            let hue = (i as f32 / (2.5 * length) + offset) % 1.0;
            let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
            result.push_str(&rgb_to_ansi(r, g, b));
            result.push(c);
        }

        format!("{}{}", RESET_COLOR, result)
    }

    pub fn animate_text<F>(content: &str, speed: f32, cond: F)
        where F: Fn() -> bool
    {
        let mut offset: f32 = 0.0;
        write!( std::io::stdout(), "{}", HIDE_CURSOR).unwrap();

        while cond() {
            write!(std::io::stdout(), "{}{}", CLEAR_LINE, color_gradient_text(&content.to_string(), offset)).expect("panik!");
            std::io::stdout().flush().expect("panik2");

            offset = (speed + offset) % 1.0;

            sleep(Duration::from_millis(50));
        }

        let mut handle = std::io::stdout().lock();
        write!(handle, "{}{}{}", CLEAR_LINE, RESET_COLOR, SHOW_CURSOR).unwrap();
    }

}

pub mod utils {
    pub fn get_sys_threads() -> usize {
        num_cpus::get()
    }

}