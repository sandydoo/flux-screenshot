use flux::{
    settings::{ClearPressure, ColorScheme, Mode, Noise, Settings},
    Flux,
};
use glow::HasContext;
use glutin::{dpi, event_loop::EventLoop};
use std::{fs, path::Path, rc::Rc, time};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let settings = Settings {
        mode: Mode::Normal,
        fluid_size: 128,
        fluid_frame_rate: 60.0,
        fluid_timestep: 1.0 / 60.0,
        viscosity: 5.0,
        velocity_dissipation: 0.0,
        clear_pressure: ClearPressure::KeepPressure,
        diffusion_iterations: 3,
        pressure_iterations: 19,
        color_scheme: ColorScheme::Peacock,
        line_length: 550.0,
        line_width: 10.0,
        line_begin_offset: 0.4,
        line_variance: 0.45,
        grid_spacing: 15,
        view_scale: 1.6,
        noise_channels: vec![
            Noise {
                scale: 2.5,
                multiplier: 1.0,
                offset_increment: 0.0015,
            },
            Noise {
                scale: 15.0,
                multiplier: 0.7,
                offset_increment: 0.0015 * 6.0,
            },
            Noise {
                scale: 30.0,
                multiplier: 0.5,
                offset_increment: 0.0015 * 12.0,
            },
        ],
    };

    // Macbook Pro 13
    let logical_size = dpi::LogicalSize::new(1280, 800);
    let physical_size = logical_size.to_physical(2.625);

    // Triple 1440p
    // let physical_size = dpi::PhysicalSize::new(7680, 1440);
    // let logical_size = physical_size.to_logical(1.0);

    println!("{:?}", logical_size);

    let (gl, _context, _event_loop) = get_headless_context(physical_size);

    let gl = Rc::new(gl);
    let mut flux = Flux::new(
        &gl,
        logical_size.width,
        logical_size.height,
        physical_size.width,
        physical_size.height,
        &Rc::new(settings),
    )
    .unwrap();

    let (renderbuffer, framebuffer) = unsafe {
        let renderbuffer = gl.create_renderbuffer().unwrap();
        gl.bind_renderbuffer(glow::RENDERBUFFER, Some(renderbuffer));
        gl.renderbuffer_storage(
            glow::RENDERBUFFER,
            glow::SRGB8_ALPHA8,
            physical_size.width as _,
            physical_size.height as _,
        );
        let framebuffer = gl.create_framebuffer().unwrap();
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
        gl.framebuffer_renderbuffer(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::RENDERBUFFER,
            Some(renderbuffer),
        );
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        gl.bind_renderbuffer(glow::RENDERBUFFER, None);

        (renderbuffer, framebuffer)
    };

    let mut now = time::Duration::from_secs(0);
    let capture_time = time::Duration::from_millis(4_500);

    while now < capture_time {
        flux.compute(now.as_secs_f64() * 1000.0);
        unsafe { gl.finish() }
        now += time::Duration::from_nanos(16_666_667);
    }

    unsafe {
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
        gl.bind_renderbuffer(glow::RENDERBUFFER, Some(renderbuffer));
    }
    flux.render();

    let mut pixels: Vec<u8> =
        vec![0; 3 * physical_size.width as usize * physical_size.height as usize];

    println!("Reading back pixels...");
    unsafe {
        gl.read_pixels(
            0,
            0,
            physical_size.width as _,
            physical_size.height as _,
            glow::RGB,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(&mut pixels),
        );
    }

    let mut frame =
        image::RgbImage::from_vec(physical_size.width, physical_size.height, pixels).unwrap();

    println!("Flipping pixels...");
    image::imageops::flip_vertical_in_place(&mut frame);

    let output_file = Path::new("output/headless.png");
    if let Some(folder_path) = output_file.parent() {
        fs::create_dir_all(folder_path).expect("Can’t create a folder for the image");
        println!("Saving image to {}...", output_file.to_str().unwrap());
        frame.save(&output_file).expect("Can’t save the image");
    }

    println!("Cleaning up buffers...");
    unsafe {
        gl.delete_framebuffer(framebuffer);
        gl.delete_renderbuffer(renderbuffer);
    }
}

// TODO: take advantage of surfaceless on supported platforms
pub fn get_headless_context(
    physical_size: dpi::PhysicalSize<u32>,
) -> (
    glow::Context,
    glutin::Context<glutin::PossiblyCurrent>,
    EventLoop<()>,
) {
    let event_loop = glutin::event_loop::EventLoop::new();
    let context = glutin::ContextBuilder::new()
        .with_gl_profile(glutin::GlProfile::Core)
        // .with_srgb(false)
        .with_double_buffer(Some(false))
        .build_headless(&event_loop, physical_size)
        .unwrap();
    let context = unsafe { context.make_current().unwrap() };

    let gl =
        unsafe { glow::Context::from_loader_function(|s| context.get_proc_address(s) as *const _) };

    (gl, context, event_loop)
}
