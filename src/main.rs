#![windows_subsystem = "windows"]


use macroquad::{
    time::get_frame_time,
    text::draw_text,
    prelude::vec2,
    math::Vec2,
    input::KeyCode,
    input::is_key_pressed,
    color::{Color, DARKGRAY},
    window::{
        clear_background,
        next_frame,
        screen_height,
        screen_width,
        Conf
    }
};

use ray_cast::{Laser, NodeNetwork};

use log::{info, debug};

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


    let mut time_delta: f32;
    let mut show_ui: bool = false;
    loop {
        clear_background(Color::from_hex(0x282A36));

        if is_key_pressed(KeyCode::Tab) { show_ui = !show_ui; }

        time_delta = get_frame_time();
        network.update(time_delta);
        network.draw();
        laser.draw(&network.get_all_connections());
        draw_text(format!("Frame time: {time_delta}").as_str(), 20.0, 20.0, 30.0, DARKGRAY);
        draw_text("Tab for options", 20.0, 40.0, 30.0, DARKGRAY);

        if show_ui {
            laser.ui();
        }
        next_frame().await
    }
}

