use log::info;
use macroquad::prelude::*;
use macroquad::prelude::scene::NodeWith;
use macroquad::telemetry::textures_count;
use ray_cast::NodeNetwork;


#[macroquad::main("RayCast")]
async fn main() {
    env_logger::init();
    info!("Program started");

    let mut network = NodeNetwork::new().await;

    // let texture: Texture2D = load_texture("assets/node.png").await.unwrap();


    let mut time_delta: f32;
    loop {
        clear_background(Color::from_hex(0x282A36));
        time_delta = get_frame_time();
        network.update(time_delta);
        
        network.draw();
        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        


        draw_text(format!("Frame time: {time_delta}").as_str(), 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}

