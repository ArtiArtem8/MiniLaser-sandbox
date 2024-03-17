#![windows_subsystem = "windows"]


use log::info;
use macroquad::prelude::*;

use ray_cast::{Laser, NodeNetwork};

fn window_conf() -> Conf {
    let mut conf = Conf {
        window_title: "RayCast".to_owned(),
        window_width: 1920 / 2,
        window_height: 1080 / 2,

        ..Default::default()
    };
    conf.platform.swap_interval = Some(0);
    conf
}
#[macroquad::main(window_conf)]
async fn main() {
    env_logger::init();
    info!("Program started");
    println!("hello world");
    let mut network = NodeNetwork::new().await;
    let mut laser = Laser::new(vec2(screen_width()/2.0, screen_height()/2.0), vec2(1.0, 0.0));
    
    network.add_node(Vec2::new(0.0, 0.0));
    network.add_node(Vec2::new(screen_width(), 0.0));
    network.add_node(Vec2::new(0.0, screen_height()));
    network.add_node(Vec2::new(screen_width(), screen_height()));
    
    /*
    let count = 1_000;
    for _ in 0..count {
        network.add_node(vec2(rand::gen_range(0.0, screen_width()), rand::gen_range(0.0, screen_height())));
    }
    for _ in 0..count {
        network.add_connection(rand::gen_range(0, count), rand::gen_range(0, count));
    }

    network.add_node(vec2(screen_width() + 100.0, screen_height() + 100.0));
    for i in 0..count {
        network.add_connection(count, i);
    }
    */


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

