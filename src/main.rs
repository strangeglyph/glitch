extern crate image;
extern crate rand;
extern crate noise;
extern crate clap;

use image::{GenericImage, ImageBuffer, Rgba, Pixel};
use rand::distributions::{Normal, Range, IndependentSample};
use rand::Rng;
use std::borrow::Borrow;
use std::str::FromStr;
use noise::*;
use clap::{App, Arg, SubCommand, ArgMatches, Shell};

type RgbaBuf = ImageBuffer<Rgba<u8>, Vec<<Rgba<u8> as Pixel>::Subpixel>>;

// Poor man's typedef
trait RgbaImage: GenericImage<Pixel=Rgba<u8>> {}

impl<T> RgbaImage for T where T: GenericImage<Pixel=Rgba<u8>> {}

#[derive(Clone)]
struct Options {
    // shift: ShiftOptions,
    shift: LimitedShiftOptions,
    scan: ScanlineOptions,
    wind: WindOptions,
    blocks: BlockShiftOptions,
}

impl Options {
    fn new(shift: LimitedShiftOptions, scan: ScanlineOptions, wind: WindOptions, blocks: BlockShiftOptions) -> Options {
        Options { shift, scan, wind, blocks }
    }

    fn step(&self) -> Options {
        Options {
            shift: self.shift,
            scan: self.scan.step(),
            wind: self.wind,
            blocks: self.blocks.step(),
        }
    }
}

/// Options for color channel offsetting
#[derive(Copy, Clone)]
struct ShiftOptions {
    r: ChannelShiftOptions,
    g: ChannelShiftOptions,
    b: ChannelShiftOptions,
}

#[derive(Copy, Clone)]
struct ChannelShiftOptions {
    base_shift_x: f64,
    base_shift_y: f64,
    current_shift_x: f64,
    current_shift_y: f64,
    radius: f64,
    max_move: f64,
}

impl ShiftOptions {
    fn random(offset: f64, radius: f64, max_move: f64) -> ShiftOptions {
        ShiftOptions {
            r: ChannelShiftOptions::random(offset, radius, max_move),
            g: ChannelShiftOptions::random(offset, radius, max_move),
            b: ChannelShiftOptions::random(offset, radius, max_move),
        }
    }

    fn step(&self) -> ShiftOptions {
        ShiftOptions {
            r: self.r.step(),
            g: self.g.step(),
            b: self.b.step(),
        }
    }
}

impl ChannelShiftOptions {
    fn random(offset: f64, radius: f64, max_move: f64) -> ChannelShiftOptions {
        // let distribution = Normal::new(0, offset/3.0);

        let distribution = Range::new(0.0, 2.0 * std::f64::consts::PI);
        let angle = distribution.ind_sample(&mut rand::thread_rng());
        let base_shift_x = f64::cos(angle) * offset;
        let base_shift_y = f64::sin(angle) * offset;

        ChannelShiftOptions {
            base_shift_x,
            base_shift_y,
            current_shift_x: base_shift_x,
            current_shift_y: base_shift_y,
            radius,
            max_move,
        }
    }

    fn step(&self) -> ChannelShiftOptions {
        // Chosen so ~99.7% of values will lie within radius
        let distribution = Normal::new(0.0, self.radius / 3.0f64);
        let new_x = self.base_shift_x + distribution.ind_sample(&mut rand::thread_rng());
        let new_y = self.base_shift_y + distribution.ind_sample(&mut rand::thread_rng());

        let mut dx = new_x - self.current_shift_x;
        let mut dy = new_y - self.current_shift_y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq > self.max_move * self.max_move {
            let scale = self.max_move / f64::sqrt(dist_sq);
            dx *= scale;
            dy *= scale;
        }

        ChannelShiftOptions {
            current_shift_x: self.current_shift_x + dx,
            current_shift_y: self.current_shift_y + dy,
            ..*self
        }
    }
}

/// Options for the red/cyan color offsetting
#[derive(Copy, Clone)]
struct LimitedShiftOptions {
    distance: i32
}

impl LimitedShiftOptions {
    fn new(distance: i32) -> LimitedShiftOptions {
        LimitedShiftOptions {
            distance
        }
    }
}

/// Options for the scanline and sinuoid desync effect
#[derive(Copy, Clone)]
struct ScanlineOptions {
    vert_dist: u32,
    gap_size: u32,
    desync_phase_shift: f64,
    desync_amplitude: f64,
    desync_frequency: f64,
}

impl ScanlineOptions {
    fn random(vert_dist: u32, gap_size: u32, desync_amplitude: f64, desync_frequency: f64) -> ScanlineOptions {
        let desync_phase_shift = rand::thread_rng().next_f64() * desync_frequency;
        ScanlineOptions {
            vert_dist,
            gap_size,
            desync_phase_shift,
            desync_amplitude,
            desync_frequency,
        }
    }

    fn step(&self) -> ScanlineOptions {
        let desync_phase_shift = self.desync_phase_shift + 0.1 * self.desync_frequency;
        ScanlineOptions {
            desync_phase_shift,
            ..*self
        }
    }
}

/// Options for the block movement effect
#[derive(Clone)]
struct BlockShiftOptions {
    blocks: Vec<Block>
}

#[derive(Copy, Clone)]
struct Block {
    min_y: u32,
    height: u32,
    shift: i32,
}

impl BlockShiftOptions {
    fn random(num_blocks: usize, max_y: u32) -> BlockShiftOptions {
        let mut rng = rand::thread_rng();
        let mut start_lines = vec!();
        for i in 0..num_blocks {
            start_lines.push(rng.gen_range(0, max_y));
        }
        start_lines.sort();
        let mut blocks = vec!();
        for i in 0..num_blocks {
            let min_y = start_lines[i];
            let max_end = if i == num_blocks - 1 { max_y } else { start_lines[i + 1] };
            let height = rng.gen_range(0, max_end - min_y);
            let shift = rng.gen_range(-20, 20);
            blocks.push(Block {
                min_y,
                height: u32::max(16, height),
                shift: if shift < 0 && shift >= -3 { -3 } else if shift >= 0 && shift <= 3 { 3 } else { shift },
            })
        }
        BlockShiftOptions {
            blocks
        }
    }

    fn step(&self) -> BlockShiftOptions {
        return BlockShiftOptions {
            blocks: self.blocks.iter().map(|&Block { min_y, height, shift }| Block { min_y: min_y + 3, height, shift }).collect::<Vec<_>>()
        };
    }
}

/// Options for the wind (horizontal line stretching) effect
#[derive(Copy, Clone)]
struct WindOptions {
    wind_onset_chance: f32,
    wind_stop_chance: f32,
}

impl WindOptions {
    fn new(wind_onset_chance: f32, wind_stop_chance: f32) -> WindOptions {
        WindOptions {
            wind_onset_chance,
            wind_stop_chance,
        }
    }
}

fn cli<'a, 'b>() -> App<'a, 'b> {
    App::new("Glitch")
        .version("1.0")
        .author("μ")
        .about("Apply a glitch effect to images")
        .subcommand(SubCommand::with_name("render")
            .about("Apply a glitch effect to images")
            .arg(Arg::with_name("file")
                .value_name("FILE")
                .help("Input image")
                .required(true)
                .index(1))
            .arg(Arg::with_name("number")
                .short("n")
                .long("number")
                .takes_value(true)
                .value_name("N")
                .validator(|n| validate::<usize>(n, "Expected an integer"))
                .default_value("1")
                .help("Number of images to generate. If generating multiple images, they will form a continuous animation")
                .display_order(1))
            .arg(Arg::with_name("color shift amount")
                .long("color-shift")
                .takes_value(true)
                .value_name("N")
                .validator(|n| validate::<i32>(n, "Expected an integer"))
                .default_value("4")
                .help("Amount of offset from original position of each color channel")
                .display_order(2))
            .arg(Arg::with_name("scanline height")
                .long("scan-height")
                .takes_value(true)
                .value_name("N")
                .validator(|n| validate::<u32>(n, "Expected an integer"))
                .default_value("6")
                .help("Height of each scanline")
                .display_order(3))
            .arg(Arg::with_name("scanline gap height")
                .long("scan-gap")
                .takes_value(true)
                .value_name("M")
                .validator(|n| validate::<u32>(n, "Expected an integer"))
                .default_value("3")
                .help("Height of the gap between scanlines")
                .display_order(4))
            .arg(Arg::with_name("desync amplitude")
                .long("desync-amp")
                .takes_value(true)
                .value_name("N")
                .validator(|n| validate::<f64>(n, "Expected a float"))
                .default_value("6.0")
                .help("Amplitude for the desync effect")
                .display_order(5))
            .arg(Arg::with_name("desync frequency")
                .long("desync-freq")
                .takes_value(true)
                .value_name("M")
                .validator(|n| validate::<f64>(n, "Expected a float"))
                .default_value("0.3")
                .help("Frequency for the desync effect")
                .display_order(6))
            .arg(Arg::with_name("wind onset chance")
                .long("wind-onset")
                .takes_value(true)
                .value_name("N")
                .validator(|n| validate::<f32>(n, "Expected a float"))
                .default_value("0.05")
                .help("Onset chance for wind effect")
                .display_order(7))
            .arg(Arg::with_name("wind continue chance")
                .long("wind-continue")
                .takes_value(true)
                .value_name("M")
                .validator(|n| validate::<f64>(n, "Expected a float"))
                .default_value("0.15")
                .help("Continue chance for wind effect")
                .display_order(8))
            .arg(Arg::with_name("block count")
                .long("blocks")
                .takes_value(true)
                .value_name("M")
                .validator(|n| validate::<usize>(n, "Expected an integer"))
                .default_value("5")
                .help("Number of blocks to shift")
                .display_order(9)))
        .subcommand(SubCommand::with_name("completion")
            .about("Generate completion scripts")
            .arg(Arg::with_name("zsh")
                .long("zsh")
                .help("Generate zsh completion")
                .display_order(1))
            .arg(Arg::with_name("bash")
                .long("bash")
                .help("Generate bash completion")
                .display_order(1))
            .arg(Arg::with_name("fish")
                .long("fish")
                .help("Generate fish completion")
                .display_order(1))
            .arg(Arg::with_name("powershell")
                .long("psh")
                .help("Generate powershell completion")
                .display_order(1)))
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        ("completion", Some(m)) => gen_completions(m),
        ("render", Some(m)) => render(m),
        _ => {
            cli().print_help().expect("Unable to print help");
            std::process::exit(1);
        },
    };
}

fn gen_completions(matches: &ArgMatches) {
    let mut cli = cli();
    if matches.is_present("zsh") {
        cli.gen_completions("glitch", Shell::Zsh, ".");
    }
    if matches.is_present("bash") {
        cli.gen_completions("glitch", Shell::Bash, ".");
    }
    if matches.is_present("fish") {
        cli.gen_completions("glitch", Shell::Fish, ".");
    }
    if matches.is_present("powershell") {
        cli.gen_completions("glitch", Shell::PowerShell, ".");
    }
}

fn validate<T: FromStr>(input: String, error: &str) -> Result<(), String> {
    if let Ok(val) = input.parse::<T>() {
        Ok(())
    } else {
        Err(String::from(error))
    }
}

fn unwrap_opt<T: FromStr>(matches: &ArgMatches, name: &str) -> T {
    matches.value_of(name).and_then(|n| n.parse::<T>().ok()).unwrap()
}

fn render(matches: &ArgMatches) {
    let img = image::open(matches.value_of("file").unwrap());
    let img = img.expect("Unable to load input image");
    println!("Size: {:?}", img.dimensions());
    println!("Color model: {:?}", img.color());
    println!("----------------------");

    let color_shift = unwrap_opt(matches, "color shift amount");
    let lim_shift_options = LimitedShiftOptions::new(color_shift);

    let scanline_vert_dist = unwrap_opt(matches, "scanline height");
    let scanline_gap_dist = unwrap_opt(matches, "scanline gap height");
    let desync_amplitude = unwrap_opt(matches, "desync amplitude");
    let desync_frequency = unwrap_opt(matches, "desync frequency");
    let scanline_options = ScanlineOptions::random(
        scanline_vert_dist,
        scanline_gap_dist,
        desync_amplitude,
        desync_frequency
    );

    let wind_onset_chance = unwrap_opt(matches, "wind onset chance");
    let wind_continue_chance = unwrap_opt(matches, "wind continue chance");
    let wind_options = WindOptions::new(wind_onset_chance, wind_continue_chance);

    let block_count = unwrap_opt(matches, "block count");
    let block_options = BlockShiftOptions::random(block_count, img.height());

    let mut opts = Options::new(
        lim_shift_options,
        scanline_options,
        wind_options,
        block_options
    );

    let n = unwrap_opt(matches, "number");
    for i in 0..n {
        println!();
        println!("Pass {}", i + 1);
        let derived_img = glitch_img(&img, &opts);
        opts = opts.step();
        derived_img.save(format!("glitch_{}.png", i)).expect("Unable to save output image");
    }
}

fn clamping_add(val: u32, offset: i32, max: u32) -> u32 {
    if -offset > val as i32 {
        return 0;
    }

    let result = (offset + val as i32) as u32;
    if result > max {
        max
    } else {
        result
    }
}

fn blend<T>(pixels: &[T]) -> Rgba<u8> where T: Borrow<Rgba<u8>> {
    let alpha_values = pixels.iter().map(|px| px.borrow().channels()[3]);
    let max_alpha = alpha_values.max().expect("Needs at least one pixel to blend");

    if max_alpha == 0 {
        return Rgba::from_channels(0, 0, 0, 0);
    }

    let max_alpha = (max_alpha as f64) / (u8::max_value() as f64);

    // To approximate light mixing, we set
    // final_rgb[color] = max_{k < n} rgb_k[color] * alpha[k] / (max alpha)

    let mut final_r = 0;
    let mut final_g = 0;
    let mut final_b = 0;
    let mut final_a = 0.0;
    for p in pixels {
        let (r, g, b, a) = p.borrow().channels4();

        let a = (a as f64) / (u8::max_value() as f64);
        let r = (r as f64) * a / max_alpha;
        let g = (g as f64) * a / max_alpha;
        let b = (b as f64) * a / max_alpha;
        final_r = u8::max(final_r, r as u8);
        final_g = u8::max(final_g, g as u8);
        final_b = u8::max(final_b, b as u8);

        final_a += (1.0 - final_a) * a;
    }

    Rgba::from_channels(final_r, final_g, final_b, (final_a * 255.0) as u8)
}

fn blend_alpha_one_minus_alpha(a: f64, col1: u8, col2: u8) -> u8 {
    let col1 = col1 as f64;
    let col2 = col2 as f64;

    f64::min(u8::max_value() as f64, a * col1 + (1.0 - a) * col2) as u8
}

fn glitch_img<T: RgbaImage>(img: &T, config: &Options) -> RgbaBuf {
    let img = scanlines(img, &config.scan);
    let img = offset_red_cyan(&img, &config.shift);
    //let img = noise(&img);
    let img = wind(&img, &config.wind);
    let img = offset_blocks(&img, &config.blocks);
    img
}

fn scanlines<T: RgbaImage>(img: &T, config: &ScanlineOptions) -> RgbaBuf {
    println!("* Adding scanlines");

    ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        // Insert scanline gap
        if y % (config.vert_dist + config.gap_size) >= config.vert_dist {
            let base_y = y - (y % (config.vert_dist + config.gap_size));
            let prev_line = base_y + config.vert_dist - 1;
            let next_line = prev_line + config.gap_size;

            let mut blend_sources = vec![];
            for &(index, weight) in [(-1, 8), (0, 4), (1, 8)].iter() {
                let src_x = clamping_add(x, index, img.width() - 1);
                let (r, g, b, a) = img.get_pixel(src_x, prev_line).channels4();
                blend_sources.push(Rgba::from_channels(r, g, b, a / weight));

                if next_line < img.height() {
                    let (r, g, b, a) = img.get_pixel(src_x, next_line).channels4();
                    blend_sources.push(Rgba::from_channels(r, g, b, a / weight));
                }
            }

            return blend(&blend_sources[..]);
        }

        // Desync lines
        let line = y / (config.vert_dist + config.gap_size);
        let line = line as f64 + config.desync_phase_shift;

        let desync_x_shift = f64::sin(line / config.desync_frequency) * config.desync_amplitude;
        let desync_x_shift = desync_x_shift + 0.3 * config.desync_amplitude * (rand::thread_rng().next_f64() - 0.5);
        let x = clamping_add(x, desync_x_shift as i32, img.width() - 1);
        img.get_pixel(x, y)
    })
}

fn offset_red_cyan<T: RgbaImage>(img: &T, config: &LimitedShiftOptions) -> RgbaBuf {
    println!("* Offsetting color channels");

    ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let red_x = clamping_add(x, config.distance, img.width() - 1);
        let (r, _, _, a) = img.get_pixel(red_x, y).channels4();
        let red_px = Rgba::from_channels(r, 0, 0, a);

        let cyan_x = clamping_add(x, -config.distance, img.width() - 1);
        let (_, g, b, a) = img.get_pixel(cyan_x, y).channels4();
        let cyan_px = Rgba::from_channels(0, g, b, a);

        blend(&[red_px, cyan_px])
    })
}

fn offset_channels<T: RgbaImage>(img: &T, config: &ShiftOptions) -> RgbaBuf {
    println!("* Offsetting color channels");

    let r_channel = ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let x = clamping_add(x, config.r.current_shift_x as i32, img.width() - 1);
        let y = clamping_add(y, config.r.current_shift_y as i32, img.height() - 1);

        let (r, _, _, a) = img.get_pixel(x, y).channels4();
        Rgba::from_channels(r, 0, 0, a)
    });

    let g_channel = ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let x = clamping_add(x, config.g.current_shift_x as i32, img.width() - 1);
        let y = clamping_add(y, config.g.current_shift_y as i32, img.height() - 1);

        let (_, g, _, a) = img.get_pixel(x, y).channels4();
        Rgba::from_channels(0, g, 0, a)
    });

    let b_channel = ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let x = clamping_add(x, config.b.current_shift_x as i32, img.width() - 1);
        let y = clamping_add(y, config.b.current_shift_y as i32, img.height() - 1);

        let (_, _, b, a) = img.get_pixel(x, y).channels4();
        Rgba::from_channels(0, 0, b, a)
    });

    ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let p1 = r_channel.get_pixel(x, y);
        let p2 = g_channel.get_pixel(x, y);
        let p3 = b_channel.get_pixel(x, y);
        blend(&[p1, p2, p3])
    })
}

fn noise<T: RgbaImage>(img: &T) -> RgbaBuf {
    println!("* Adding grain");

    let noise = noise::Fbm::new()
        .set_frequency(96.0)
        .set_lacunarity(2.0)
        .set_octaves(6)
        .set_persistence(0.5)
        .set_seed(rand::thread_rng().next_u32() as usize);

    let max = 0.0;
    ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        // Normalize coordinates to [0,1]
        let pt = [x as f64 / img.width() as f64, y as f64 / img.height() as f64];
        let val = noise.get(pt);

        // Apply sigmoid shaping
        let val = val * 4.0; // Sharper edge
        let val = val - 1.25; // Stay in black area longer
        let exp = f64::exp(val);
        let val = exp / (1.0 + exp);

        let noise_alpha = 0.2;
        let val = (val * noise_alpha * (u8::max_value() as f64)) as u8;

        let (r, g, b, a) = img.get_pixel(x, y).channels4();

        let (r, g, b, a) = if a == 0 {
            let val = val as u8;
            (val, val, val, val)
        } else {
            let af = a as f64;
            /*
            let r = blend_alpha_one_minus_alpha(noise_alpha, val, r);
            let g = blend_alpha_one_minus_alpha(noise_alpha, val, g);
            let b = blend_alpha_one_minus_alpha(noise_alpha, val, b);
            */
            /*
            let r = blend_alpha_one_minus_alpha(af, r, val);
            let g = blend_alpha_one_minus_alpha(af, g, val);
            let b = blend_alpha_one_minus_alpha(af, b, val);
            */
            let r = clamping_add(r as u32, val as i32, u8::max_value() as u32) as u8;
            let g = clamping_add(g as u32, val as i32, u8::max_value() as u32) as u8;
            let b = clamping_add(b as u32, val as i32, u8::max_value() as u32) as u8;
            let a = (af + (1.0 - af) * noise_alpha) as u8;
            (r, g, b, a)
        };

        Rgba::from_channels(r, g, b, a)
    })
}

fn wind<T: RgbaImage>(img: &T, config: &WindOptions) -> RgbaBuf {
    println!("* Applying wind effect");

    let mut img = ImageBuffer::from_fn(img.width(), img.height(), |x, y| img.get_pixel(x, y));
    let mut rng = rand::thread_rng();
    for y in 0..img.height() {
        let mut x = img.width();
        while x > 0 {
            x -= 1;
            let (r, g, b, a) = img[(x, y)].channels4();
            if a > 0 && rng.next_f32() < config.wind_onset_chance {
                while x > 0 {
                    x -= 1;
                    img[(x, y)] = Rgba::from_channels(r, g, b, a);
                    if rng.next_f32() < config.wind_stop_chance {
                        break;
                    }
                }
            }
        }
    }

    img
}

fn offset_blocks<T: RgbaImage>(img: &T, config: &BlockShiftOptions) -> RgbaBuf {
    println!("* Shifting blocks");

    ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        for block in &config.blocks {
            if y < block.min_y { break; }
            if y >= block.min_y && y < block.min_y + block.height {
                return img.get_pixel(clamping_add(x, block.shift, img.width() - 1), y);
            }
        }

        img.get_pixel(x, y)
    })
}