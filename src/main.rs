use log::info;
use macroquad::prelude::*;

use ray_cast::{Laser, NodeNetwork};

#[macroquad::main("RayCast")]
async fn main() {
    env_logger::init();
    info!("Program started");

    let mut network = NodeNetwork::new().await;
    let mut laser = Laser::new(vec2(screen_width()/2.0, screen_height()/2.0), vec2(0.0, -1.0));
    // draw giant star using network
    
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
    
    

    let mut time_delta: f32;
    loop {
        clear_background(Color::from_hex(0x282A36));
        time_delta = get_frame_time();
        network.update(time_delta);
        network.draw();
        laser.draw();
        laser.look_at(Vec2::from(mouse_position()));
        laser.collide_many(&network.get_all_connections());
        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);


        draw_text(format!("Frame time: {time_delta}").as_str(), 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}

