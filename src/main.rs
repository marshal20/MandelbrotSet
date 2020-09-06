use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use std::time;
use rand::Rng;

type Real = f32;

#[derive(Copy, Clone)]
struct Complex {
    r: Real,
    i: Real,
}

impl Complex {
    fn squared(&self) -> Complex {
        Complex {r: self.r*self.r - self.i*self.i, i: 2.0*self.r*self.i}
    }

    fn add(&self, rhs: &Complex) -> Complex {
        Complex {r: self.r + rhs.r, i: self.i + rhs.i}
    }

    fn length(&self) -> Real {
        (self.r*self.r + self.i*self.i).sqrt()
    }
}

#[derive(Copy, Clone)]
struct Color {
    r: Real,
    g: Real,
    b: Real,
    a: Real,
}

impl Color {
    fn new() -> Color {
        Color {r:0.0, g:0.0, b:0.0, a:0.0}
    }

    fn add(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.a += rhs.a;
    }

    fn divide(&mut self, value: Real) {
        self.r /= value;
        self.g /= value;
        self.b /= value;
        self.a /= value;
    }
}

fn save_image(color_buffer: &[Color], width: usize, height: usize, path: &str) {
    let path = Path::new(path);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);
    
    let mut encoder = png::Encoder::new(w, width as u32, height as u32);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    
    let mut rgba_data = vec![0u8; 4 * width * height];
    for y in 0..height {
        for x in 0..width {
            rgba_data[4 * (y * width + x) + 0] = (color_buffer[y * width + x].r * 255.0) as u8;
            rgba_data[4 * (y * width + x) + 1] = (color_buffer[y * width + x].g * 255.0) as u8;
            rgba_data[4 * (y * width + x) + 2] = (color_buffer[y * width + x].b * 255.0) as u8;
            rgba_data[4 * (y * width + x) + 3] = (color_buffer[y * width + x].a * 255.0) as u8;
        }
    }

    writer.write_image_data(&rgba_data).unwrap();
}

// Prints the progress [0:100] as a bar in the console
fn print_progress(progress: u32) {
    let mut progress_bar = String::from("[");
    for i in 0..50 {
        if i <= progress/2 {
            progress_bar.push('=');
        }
        else {
            progress_bar.push(' ');
        }
    }
    progress_bar.push_str("]");
    print!("\r{} {}%  ", progress_bar, progress);
}

fn main() {
    const BUFFER_WIDTH: usize = 1366;
    const BUFFER_HEIGHT: usize = 768;
    const BUFFER_ASPECT_RATIO: Real = (BUFFER_WIDTH as Real) / (BUFFER_HEIGHT as Real);
    const SAMPLE_COUNT: usize = 16;
    const CENTER_X: Real = -0.7453;
    const CENTER_Y: Real = 0.1127;
    const VIEW_WIDTH: Real = 6.5E-4 * BUFFER_ASPECT_RATIO;
    const VIEW_HEIGHT: Real = 6.5E-4;
    const MAX_ITERATIONS: u32 = 250;
    const MAX_LENGTH: Real = 2.0;

    // COLOR PALLETE SOURCE: https://stackoverflow.com/a/16505538/9218594
    const COLOR_PALETTE: [Color; 16] = [
        Color {r:0.26, g:0.1,  b:0.06, a:1.0}, 
        Color {r:0.1,  g:0.03, b:0.1,  a:1.0}, 
        Color {r:0.3,  g:0.01, b:0.18, a:1.0}, 
        Color {r:0.02, g:0.02, b:0.28, a:1.0}, 
        Color {r:0.0,  g:0.03, b:0.4,  a:1.0}, 
        Color {r:0.05, g:0.17, b:0.54, a:1.0}, 
        Color {r:0.1,  g:0.3,  b:0.7,  a:1.0}, 
        Color {r:0.25, g:0.5,  b:0.82, a:1.0}, 
        Color {r:0.53, g:0.71, b:0.9,  a:1.0}, 
        Color {r:0.83, g:0.93, b:0.97, a:1.0}, 
        Color {r:0.95, g:0.91, b:0.75, a:1.0}, 
        Color {r:0.97, g:0.78, b:0.37, a:1.0}, 
        Color {r:1.0,  g:0.67, b:0.0,  a:1.0}, 
        Color {r:0.8,  g:0.5,  b:0.0,  a:1.0}, 
        Color {r:0.6,  g:0.34, b:0.0,  a:1.0}, 
        Color {r:0.42, g:0.2,  b:0.02, a:1.0} 
        ];

    let mut color_buffer = vec![Color::new(); BUFFER_WIDTH * BUFFER_HEIGHT]; // Row major

    let mut rng = rand::thread_rng();
    println!("Drawing the buffer...");
    let start_time = time::Instant::now();

    for y in 0..BUFFER_HEIGHT {
        for x in 0..BUFFER_WIDTH {
            let mut pixel_color = Color::new();
            for _ in 0..SAMPLE_COUNT {
                let norm_pos_x = ((x as Real) + rng.gen_range(-0.5, 0.5))/(BUFFER_WIDTH as Real) * 2.0 - 1.0; // [-1:1]
                let norm_pos_y = ((y as Real) + rng.gen_range(-0.5, 0.5))/(BUFFER_HEIGHT as Real) * 2.0 - 1.0; // [-1:1]
                let mut pos = Complex {r: 0.0, i: 0.0};
                pos.r = CENTER_X + norm_pos_x * VIEW_WIDTH / 2.0; // real axis
                pos.i = CENTER_Y + norm_pos_y * VIEW_HEIGHT / 2.0; // imaginary axis
                let mut iterations: u32 = 0;
                let mut temp = Complex {r: 0.0, i: 0.0};
                while temp.length() <= MAX_LENGTH && iterations < MAX_ITERATIONS {
                    temp = temp.squared().add(&pos);
                    iterations += 1;
                }
                pixel_color.add(COLOR_PALETTE[(iterations%16) as usize]);
            }
            pixel_color.divide(SAMPLE_COUNT as Real);
            color_buffer[y * BUFFER_WIDTH + x] = pixel_color;
        }
        let progress = (y * 100) / BUFFER_HEIGHT; // [0:100]
        print_progress(progress as u32);
    }
    print_progress(100);
    let duration = time::Instant::now().duration_since(start_time).as_secs();
    println!("\nFinished rendering in {}h{}m{}s", (duration/60/60), (duration/60)%60, duration%(60*60));

    save_image(&color_buffer, BUFFER_WIDTH, BUFFER_HEIGHT, "output/image.png");
    println!("Saved buffer to image.png");
}
