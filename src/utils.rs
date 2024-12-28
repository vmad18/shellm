pub mod model_tool;

pub mod color {
    use std::io::Write;
    use std::thread::sleep;
    use std::time::Duration;

    const CLEAR_LINE: &str = "\x1b[2K\x1b[G";
    const RESET_COLOR: &str = "\x1b[0m";
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

    pub fn color_gradient_text(content: &String, offset: f32) -> String {
        let mut result = String::new();
        let length = content.len() as f32;

        for (i, c) in content.chars().enumerate() {
            let hue = (i as f32 / (5.0 * length) + offset) % 1.0;
            let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
            result.push_str(&rgb_to_ansi(r, g, b));
            result.push(c);
            // let colored = format!("{}{}", rgb_to_ansi(r, g, b), c);
            // result.push_str(colored.as_str());
        }

        result.push_str(RESET_COLOR);
        result
    }

    pub fn animate_text(content: String, speed: f32) {
        let mut offset: f32 = 0.0;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();

        loop {
            write!(handle, "{}{}", CLEAR_LINE, color_gradient_text(&content, offset)).expect("panik!");
            handle.flush().expect("panik2");

            offset = (speed + offset) % 1.0;

            sleep(Duration::from_millis(50));
        }
    }

}

pub mod utils {
    pub fn get_sys_threads() -> usize {
        num_cpus::get()
    }

}