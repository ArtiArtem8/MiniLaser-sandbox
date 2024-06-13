// #![windows_subsystem = "windows"]


use log::{debug, info};
use macroquad::{
    color::{Color, DARKGRAY},
    color_u8,
    input::is_key_pressed,
    input::KeyCode,
    math::Vec2,
    prelude::vec2,
    text::draw_text,
    time::get_frame_time, window::{
        clear_background,
        Conf,
        next_frame,
        screen_height,
        screen_width,
    }};
use macroquad::time::get_time;

use ray_cast::{Laser, NodeNetwork, Segment};

fn window_conf() -> Conf {
    let mut conf = Conf {
        window_title: "RayCast".to_owned(),
        window_width: 1920 / 2,
        window_height: 1080 / 2,
        ..Default::default()
    };
    conf.platform.swap_interval = Some(0);
    conf.high_dpi = true;
    conf
}

const BACKGROUND: Color = Color::new(0.1568627450980392, 0.16470588235294117,
                                     0.21176470588235294, 1.0);

#[macroquad::main(window_conf)]
async fn main() {
    #[cfg(target_family = "wasm")]
    sapp_console_log::init_with_level(log::Level::Info).unwrap();

    #[cfg(not(target_family = "wasm"))]
    env_logger::init();

    info!("Program started");
    debug!("Debug mode enabled");

    let mut network = NodeNetwork::new().await;
    let mut laser = Laser::new(vec2(screen_width() / 2.0, screen_height() / 2.0), vec2(1.0, 0.0));

    // default nodes at the corners of the screen
    network.add_node(Vec2::new(0.0, 0.0));
    network.add_node(Vec2::new(screen_width(), 0.0));
    network.add_node(Vec2::new(0.0, screen_height()));
    network.add_node(Vec2::new(screen_width(), screen_height()));

    // default connections
    network.add_connection(0, 1);
    network.add_connection(0, 2);
    network.add_connection(1, 3);
    network.add_connection(2, 3);


    let mut time_delta: f32;
    let mut show_ui: bool = false;
    let mut frame_time: f32 = 0.0;
    let mut lasers: Vec<Segment> = network.get_all_connections();
    loop {
        clear_background(BACKGROUND);

        if is_key_pressed(KeyCode::Tab) { show_ui = !show_ui; }

        time_delta = get_frame_time();
        network.update(time_delta);
        if frame_time > 0.01667 {
            lasers = network.get_all_connections();
            frame_time = 0.0;
        } else { frame_time += time_delta; }
        network.draw();
        laser.draw_rays_new(&lasers);
        laser.draw_laser_texture();
        // laser.draw(&network.get_all_connections());
        draw_text(format!("Frame time: {time_delta}").as_str(), 20.0, 20.0, 30.0, DARKGRAY);
        draw_text("Tab for options", 20.0, 40.0, 30.0, DARKGRAY);

        if show_ui {
            laser.ui();
        }
        next_frame().await
    }
}
