// #![windows_subsystem = "windows"]

use std::collections::HashMap;
use std::mem::size_of_val;
use std::time::Instant;

use log::{debug, info};
use macroquad::material::{gl_use_default_material, gl_use_material, MaterialParams};
use macroquad::miniquad::window::screen_size;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, ShaderSource};
use macroquad::prelude::scene::camera_pos;
use macroquad::prelude::*;
use macroquad::prelude::{draw_line, load_material, PipelineParams, BLUE, RED};
use macroquad::time::get_time;
use macroquad::ui::{root_ui, widgets};
use macroquad::{color::{Color, DARKGRAY}, hash, input::is_key_pressed, input::KeyCode, math::Vec2, prelude::vec2, text::draw_text, time::get_frame_time, window::{
    clear_background,
    next_frame,
    screen_height,
    screen_width,
    Conf,
}};
use ray_cast::{tuple2vec, vec2tuple, EdgeState, Laser, NodeNetwork, Segment};


mod labyrinth;

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
    let mut labyrinth = labyrinth::Labyrinth::new(5.0, (5, 5));
    labyrinth.generate_depth_first();


    // info!("{:?}", labyrinth.get_cells());
    let light_material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::One,
                )),
                alpha_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::One,
                    BlendFactor::One,
                )
                ),
                ..Default::default()
            },
            ..Default::default()
        },
    ).unwrap();


    // // default nodes at the corners of the screen
    // network.add_node(Vec2::new(0.0, 0.0));
    // network.add_node(Vec2::new(screen_width(), 0.0));
    // network.add_node(Vec2::new(0.0, screen_height()));
    // network.add_node(Vec2::new(screen_width(), screen_height()));
    // 
    // // default connections
    // network.add_connection(0, 1);
    // network.add_connection(0, 2);
    // network.add_connection(1, 3);
    // network.add_connection(2, 3);

    // node_circle( &mut network, Vec2::new(200.0, 200.0), 150.0);

    // lines_to_nodes(&mut network, &labyrinth.get_as_lines(), 20.0);

    let mut enable_collisions: bool = true;
    let mut time_delta: f32;
    let mut show_ui: bool = false;
    let mut frame_time: f32 = 0.0;
    let mut segments: Vec<Segment> = Vec::new();
    let mut collisions: Vec<(Vec2, Vec2, Color)> = Vec::new();
    
    let mut zoom: f32 = 1.0;
    let zoom_step: f32 = 0.001;
    let mut camera_target = vec2(screen_width() / 2.0, screen_height() / 2.0);
    let mut misc_ui = MiscUI::new();
    loop {
        clear_background(BACKGROUND);

        if is_key_pressed(KeyCode::Tab) { show_ui = !show_ui; }
        if is_key_pressed(KeyCode::CapsLock) { enable_collisions = !enable_collisions; }

        time_delta = get_frame_time();
        network.update(time_delta);
        unsafe { network.update_camera(camera_target, zoom); }
        if frame_time > 0.01667 && enable_collisions {
            segments = network.get_all_connections();
            collisions = laser.solve_collisions(&segments);
            frame_time = 0.0;
        } else { frame_time += time_delta; }
        handle_mouse_wheel(&mut zoom, &mut camera_target, mouse_position(), zoom_step);
        set_camera(&Camera2D {
            zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()) * zoom,
            target: camera_target,
            ..Default::default()
        });
        gl_use_material(&light_material);
        laser.draw_rays_explicit(&collisions);
        gl_use_default_material();
        network.draw();
        laser.draw_laser_texture();
        set_default_camera();
        // laser.draw(&network.get_all_connections());
        draw_text(format!("Frame time: {}", time_delta).as_str(), 20.0, 20.0, 30.0, DARKGRAY);
        draw_text("Tab for options, Capslock for disable collisions", 20.0, 40.0, 30.0, DARKGRAY);

        if show_ui {
            misc_ui.ui(&mut network);
            laser.ui();
        }
        next_frame().await
    }
}

struct MiscUI {
    lab_position: Vec2,
    lab_size: Vec2,
    lab_cell_size: f32,
    circle_position: Vec2,
    circle_radius: f32,
    circle_sides: f32,
    edge_state: EdgeState,
    edge_combobox: usize,
}

impl MiscUI {
    fn new() -> Self {
        Self {
            lab_position: Vec2::new(0.0, 0.0),
            lab_size: Vec2::new(10.0, 10.0),
            lab_cell_size: 50.0,
            circle_position: vec2tuple(screen_size()) / 2.0,
            circle_radius: 100.0,
            circle_sides: 20.0,
            edge_state: EdgeState::Reflective,
            edge_combobox: 0,
        }
    }
    fn ui(&mut self, node_network: &mut NodeNetwork) {
        widgets::Window::new(hash!(), Vec2::new(400., 0.), Vec2::new(300., 300.))
            .label("Misc")
            .ui(&mut *root_ui(), |ui| {
                ui.label(vec2(100.0, 0.0), "Labyrinth (pos in top left)");
                ui.slider(hash!(), "lab x",
                          0.0f32..screen_width(), &mut self.lab_position.x);
                ui.slider(hash!(), "lab y",
                          0.0f32..screen_height(), &mut self.lab_position.y);
                ui.slider(hash!(), "size in cells (square)",
                          0.0f32..screen_height(), &mut self.lab_size.x);
                ui.slider(hash!(), "cell size", 0.0f32..100.0, &mut self.lab_cell_size);
                self.lab_size = self.lab_size.round();
                if ui.button(vec2(100.0, 110.0), "Build Labyrinth") {
                    let size = (self.lab_size.x as usize, self.lab_size.x as usize);
                    let mut labyrinth = labyrinth::Labyrinth::new(self.lab_cell_size, size);
                    labyrinth.generate_depth_first();
                    lines_to_nodes(node_network, &labyrinth.get_as_lines(),
                                   tuple2vec(self.lab_position), self.edge_state);
                };
                ui.label(vec2(10.0, 130.0), "Circle (pos in center)");
                for _ in 0..12 { ui.separator(); }
                ui.slider(hash!(), "circle x", 0.0f32..screen_width(), &mut self.circle_position.x);
                ui.slider(hash!(), "circle y", 0.0f32..screen_height(), &mut self.circle_position.y);
                ui.slider(hash!(), "circle radius", 0.0f32..screen_height(), &mut self.circle_radius);
                ui.slider(hash!(), "circle sides", 1.0f32..1000.0f32, &mut self.circle_sides);
                self.circle_sides = self.circle_sides.round();
                if ui.button(vec2(100.0, 230.0), "Draw Circle") {
                    node_circle(node_network, self.circle_position,
                                self.circle_radius, self.edge_state, self.circle_sides as usize);
                };
                if ui.button(vec2(100.0, 250.0), "Delete all nodes") {
                    node_network.clean();
                };
                ui.combo_box(hash!(), "Edge type",
                             &["Solid", "Black", "Transparent"], &mut self.edge_combobox);
                match self.edge_combobox {
                    0 => self.edge_state = EdgeState::Reflective,
                    1 => self.edge_state = EdgeState::Absorptive,
                    2 => self.edge_state = EdgeState::Transparent,
                    _ => self.edge_state = EdgeState::Reflective
                }
            });
    }
}

fn handle_mouse_wheel(zoom: &mut f32, camera_target: &mut Vec2, mouse_position: (f32, f32), zoom_step: f32) {
    let mouse_position_screen = mouse_position;
    let mouse_position_world = screen_to_world(mouse_position_screen, &camera_target, *zoom);
    let wheel = mouse_wheel().1;
    if wheel == 0.0 { return; }
    if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
        let new_zoom = *zoom * (zoom_step * wheel).exp(); //.clamp(0.5, 4.0);
        // Adjust camera target to keep zoom centered at mouse position
        *camera_target = vec2(
            mouse_position_world.x - (mouse_position_screen.0 - screen_width() / 2.0) / new_zoom,
            mouse_position_world.y - (mouse_position_screen.1 - screen_height() / 2.0) / new_zoom,
        ).round();
        *zoom = new_zoom;
    } else if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
        // move camera horizontally
        camera_target.x -= wheel / *zoom * 0.25;
    } else {
        // move camera vertically
        camera_target.y -= wheel / *zoom * 0.25;
    }
}

const VERTEX_SHADER: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    varying lowp vec2 uv;
    varying lowp vec4 color;

    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
        uv = texcoord;
    }"#;
const FRAGMENT_SHADER: &str = r#"
#version 100
varying lowp vec4 color;
varying lowp vec2 uv;

uniform sampler2D Texture;

void main() {
    gl_FragColor = color * texture2D(Texture, uv) ;
}"#;

fn node_circle(node_network: &mut NodeNetwork, pos: Vec2, radius: f32, edge_state: EdgeState, sides: usize) {
    let mut key = 0;
    for i in 0..sides {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / (sides as f32);
        let pos = pos + Vec2::new(radius * angle.cos(), radius * angle.sin());
        key = node_network.add_node(pos);
        if i != 0 {
            node_network.add_connection(key, key - 1);
            if let Some(x) = node_network.connections.last_mut() {
                x.set_state(edge_state);
            }
        }
    }
    node_network.add_connection(key, key - 19);
    if let Some(x) = node_network.connections.last_mut() {
        x.set_state(edge_state);
    }
}

// Helper function to transform screen coordinates to world coordinates
fn screen_to_world(screen_pos: (f32, f32), camera_target: &Vec2, zoom: f32) -> Vec2 {
    let (sx, sy) = screen_pos;
    let screen_center = vec2(screen_width() / 2.0, screen_height() / 2.0);
    vec2(
        (sx - screen_center.x) / zoom + camera_target.x,
        (sy - screen_center.y) / zoom + camera_target.y,
    )
}

fn lines_to_nodes(node_network: &mut NodeNetwork, lines: &[((f32, f32), (f32, f32))], (offset_x, offset_y): (f32, f32), edge_state: EdgeState)
{
    let mut node_map: HashMap<u64, usize> = HashMap::new(); // Mapping from position to node id
    for line in lines {
        let pos1 = (offset_x + line.0.0, offset_y + line.0.1);
        let pos2 = (offset_x + line.1.0, offset_y + line.1.1);

        // Check if nodes already exist at these positions
        let k1 = *node_map.entry(into(pos1))
            .or_insert_with(|| node_network.add_node_with_radius(vec2tuple(pos1), 2.));
        let k2 = *node_map.entry(into(pos2))
            .or_insert_with(|| node_network.add_node_with_radius(vec2tuple(pos2), 2.));

        node_network.add_connection(k1, k2);
        if let Some(x) = node_network.connections.last_mut() {
            x.set_state(edge_state);
        }
        // draw_line(line.0.0, line.0.1, line.1.0, line.1.1, 5.0, Color::new(1.0, 1.0, 1.0, 1.0));
    }
}

fn into((x, y): (f32, f32)) -> u64 {
    let x_bits = x.to_bits() as u64;
    let y_bits = y.to_bits() as u64;
    (x_bits << 32) | y_bits
}
