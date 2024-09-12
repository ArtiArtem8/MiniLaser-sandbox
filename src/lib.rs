use std::collections::{HashMap, VecDeque};
use std::ptr::addr_of_mut;
use std::time::Instant;

// #[cfg(not(target_family = "wasm"))]
use log::{debug, error};
use macroquad::color::{BLACK, BLUE, Color, DARKGRAY, hsl_to_rgb, RED, SKYBLUE, WHITE};
use macroquad::experimental::scene::camera_pos;
use macroquad::hash;
use macroquad::input::{is_key_down, is_mouse_button_pressed, is_mouse_button_released,
                       KeyCode, mouse_position as other_mouse_position, MouseButton};
use macroquad::math::{DVec2, Vec2, vec2};
use macroquad::prelude::{draw_text, glam, ImageFormat};
use macroquad::shapes::{draw_circle, draw_line};
use macroquad::texture::{draw_texture_ex,
                         DrawTextureParams,
                         Texture2D};
use macroquad::ui::{root_ui, widgets};
use macroquad::window::{screen_height, screen_width};

mod labyrinth;

// #[cfg(target_family = "wasm")]
// use macroquad::logging::info;
//
// #[cfg(target_family = "wasm")]
// #[cfg(debug_assertions)]
// use macroquad::logging::debug;
//
// #[cfg(target_family = "wasm")]
// use macroquad::logging::error;

static mut ESTIMATE_IN_SECONDS: bool = false;
static mut OBJECT_REFLECTIVITY: f32 = 1.0;
static mut ESTIMATE_MILLIS: f32 = 1.0;
static mut MAX_RAYS: f32 = 1000.0;
static mut CAMERA_TARGET: Vec2 = vec2(0.0, 0.0);
static mut ZOOM: f32 = 1.0;

fn mouse_position() -> (f32, f32) {
    unsafe {
        screen_to_world(other_mouse_position())
    }
}

unsafe fn screen_to_world(mouse_pos: (f32, f32)) -> (f32, f32) {
    let (sx, sy) = mouse_pos;
    let screen_center = vec2(screen_width() / 2.0, screen_height() / 2.0);
    (
        (sx - screen_center.x) / ZOOM + CAMERA_TARGET.x,
        (sy - screen_center.y) / ZOOM + CAMERA_TARGET.y,
    )
}

unsafe fn world_to_screen((x, y): (f32, f32)) -> (f32, f32) {
    
    let screen_center = vec2(screen_width() / 2.0, screen_height() / 2.0);
    let (x, y) = (x - CAMERA_TARGET.x, y - CAMERA_TARGET.y);
    (x * ZOOM + screen_center.x, y * ZOOM + screen_center.y)
    
}

#[derive(Clone, Default, Debug)]
pub struct Node {
    position: Vec2,
    radius: f32,
    color: Color,
    default_radius: f32,
    is_hovered: bool,
    is_dragged: bool,
    dragged_start_pos: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EdgeState {
    #[default]
    Reflective,
    Absorptive,
    Transparent,
}


#[derive(Clone, Debug)]
pub struct Edge {
    a: usize,
    b: usize,
    color: Color,
    thickness: f32,
    is_hovered: bool,
    state: EdgeState,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Segment(Vec2, Vec2, EdgeState);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct CollisionInfo {
    pub position: Vec2,
    pub normal: Vec2,
}

impl Edge {
    pub const fn new(a: usize, b: usize) -> Self {
        Self { a, b, color: WHITE, thickness: 5.0, is_hovered: false, state: EdgeState::Reflective }
    }

    pub const fn new_with_state(a: usize, b: usize, state: EdgeState) -> Self {
        Self { a, b, color: WHITE, thickness: 5.0, is_hovered: false, state }
    }

    pub fn set_state(&mut self, state: EdgeState) {
        debug!( "Setting state to {:?} from {:?}", state, self.state);
        self.state = state
    }

    pub fn cycle_state(&mut self) {
        self.set_state(match self.state {
            EdgeState::Reflective => EdgeState::Absorptive,
            EdgeState::Absorptive => EdgeState::Transparent,
            EdgeState::Transparent => EdgeState::Reflective,
        })
    }

    fn draw(&self, start: Vec2, end: Vec2) {
        draw_line(start.x, start.y, end.x, end.y, self.thickness, self.color);
    }
    pub(crate) fn update(&mut self, delta: f32) {
        let target_color = if self.is_hovered { SKYBLUE } else {
            match self.state {
                EdgeState::Reflective => WHITE,
                EdgeState::Absorptive => BLACK,
                EdgeState::Transparent => Color::new(1.0, 1.0, 1.0, 0.5),
            }
        };
        lerp_color_in_place(&mut self.color, target_color, delta / 0.10);
    }
}

pub struct NodeNetwork {
    pub nodes: HashMap<usize, Node>,
    pub connections: Vec<Edge>,
    texture: Texture2D,
    dragged_node: Option<usize>,
    selected_node: Option<usize>,
    key: usize,
}


   


#[derive(Clone, Copy, Debug)]
pub struct Ray {
    origin: Vec2,
    direction: Vec2,
    color: Color,
}

pub struct Laser {
    position: Vec2,
    direction: Vec2,
    ray: Ray,
    thickness: f32,
    texture: Texture2D,
}

impl Laser {
    pub const MAX_DISTANCE: f32 = 20_000.0;

    pub fn new(position: Vec2, direction: Vec2) -> Self {
        Self {
            position,
            direction,
            ray: Ray { origin: position + direction * 35.0, direction, color: Color::new(1.0, 0., 0., 1.) },
            thickness: 5.0,
            texture: Texture2D::from_file_with_format(
                include_bytes!("../assets/laser.png"),
                Some(ImageFormat::Png),
            ),
        }
    }

    pub fn ui(&mut self) {
        let mut rotation = self.ray.direction.y.atan2(self.ray.direction.x).to_degrees();
        if rotation < 0.0 { rotation += 360.0; }
        widgets::Window::new(hash!(), Vec2::new(0., 0.), Vec2::new(400., 100.))
            .label("Laser")
            .ui(&mut *root_ui(), |ui| {
                ui.slider(hash!(), "pos x", 0.0f32..screen_width(), &mut self.position.x);
                ui.slider(hash!(), "pos y", 0.0f32..screen_height(), &mut self.position.y);
                ui.slider(hash!(), "rotation", 0.0f32..360.0f32, &mut rotation);
                ui.slider(hash!(), "thickness", 0.01f32..10.0f32, &mut self.thickness);
                unsafe { ui.slider(hash!(), "OBJECT_REFLECTIVITY ", 0.00f32..1.0f32, &mut *addr_of_mut!(OBJECT_REFLECTIVITY)); }

                // not allow in web
                #[cfg(not(target_family = "wasm"))]
                {
                    unsafe {
                        ui.checkbox(hash!(), "estimate in milliseconds",
                                    &mut *addr_of_mut!(ESTIMATE_IN_SECONDS));
                    }
                    unsafe {
                        ui.slider(hash!(), "milliseconds", 0.00f32..100.0f32,
                                  &mut *addr_of_mut!(ESTIMATE_MILLIS));
                    }
                }
                unsafe {
                    ui.slider(hash!(), "max rays", 1.0f32..100_000.0f32,
                              &mut *addr_of_mut!(MAX_RAYS));
                }
                unsafe { MAX_RAYS = MAX_RAYS.round(); }
            });
        self.direction = Vec2::from_angle(rotation.to_radians());
        self.ray.origin = self.position;
        self.ray.direction = self.direction;
    }


    pub fn draw_laser_texture(&self) {
        let center = Vec2::new(self.position.x, self.position.y);
        let size = 80.0; // in pixels
        let top_left = center - Vec2::new(size, size) / 2.0;
        draw_texture_ex(
            &self.texture,
            top_left.x,
            top_left.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(size, size)),
                rotation: self.ray.direction.y.atan2(self.ray.direction.x),
                ..Default::default()
            },
        );
    }
    // 
    // pub fn draw_rays(&mut self, other: &[Segment]) {
    //     self.ray.origin = self.position + self.direction * 40.0;
    //     let (points, _) = unsafe { self.collide_many(other) };
    //     let mut prev_pos = self.position;
    //     let mut t = 0.0;
    //     let max_rays = unsafe { MAX_RAYS };
    //     // draw_text(format!("{} {:?}", points.len(), points.iter().take(5).collect::<Vec<_>>()).as_str(), 20.0, 60.0, 30.0, DARKGRAY);
    //     // debug!("{} {:?}", points.len(), points.iter().take(5).collect::<Vec<_>>());
    //     for pos in points.iter() {
    //         let color = hsl_to_rgb(t / max_rays, 1.0, 0.5);
    //         // color.a = 1.0 - t/ 1000.0;
    //         draw_line(prev_pos.x, prev_pos.y, pos.x, pos.y, self.thickness,
    //                   color);
    //         prev_pos = *pos;
    //         t += 1.0;
    //     }
    // }
    // 
    pub fn draw_rays_new(&mut self, other: &[Segment]) {
        let lines = unsafe { self.solve_collisions(other) };
        draw_text(format!("Rays: {}", lines.len()).as_str(), 20.0, 20.0, 30.0, DARKGRAY);

        // let ray = self.ray;
        // self.ray.origin += Vec2::Y * 2.0;
        // lines.extend( unsafe { self.solve_collisions(other) } );
        // self.ray.origin += Vec2::Y * -4.0;
        // lines.extend( unsafe { self.solve_collisions(other) } );
        // for i in -45..45 {
        //      self.ray.direction = rotate(ray.direction, 2.0 * PI / 360.0 * i as f32);
        //     lines.extend( unsafe { self.solve_collisions(other) } );
        // }
        // self.ray = ray;
        for line in lines.iter() {
            draw_line(line.0.x, line.0.y, line.1.x, line.1.y, self.thickness,
                      line.2);
        }
    }
    pub fn draw_rays_explicit(&mut self, collisions: &[(Vec2, Vec2, Color)]) {
        let lines = collisions;
        draw_text(format!("Rays: {}", lines.len()).as_str(), 20.0, 20.0, 30.0, DARKGRAY);
        for line in lines.iter() {
            draw_line(line.0.x, line.0.y, line.1.x, line.1.y, self.thickness,
                      line.2);
        }
    }
    // 
    // 
    // pub fn draw2(&mut self, other: &Vec<Segment>, time: f64) {
    //     self.ray.origin = self.position + self.direction * 40.0;
    //     let (points, _) = unsafe { self.collide_many(other) };
    //     let mut prev_pos = self.position;
    //     let mut t = 0.0;
    //     let max_rays = unsafe { MAX_RAYS };
    //     for pos in points.iter().take((time * 500. % 1000.) as usize) {
    //         let mut color = hsl_to_rgb(t / max_rays, 1.0, 0.5);
    //         color.a = 1.0 - t / 1000.0;
    //         draw_line(prev_pos.x, prev_pos.y, pos.x, pos.y, self.thickness,
    //                   color);
    //         prev_pos = *pos;
    //         t += 1.0;
    //     }
    //     // if points.len() < 1_000 {
    //     //     // let end = prev_pos + direction * 10_000.0;
    //     //     // draw_line(prev_pos.x, prev_pos.y, end.x, end.y, self.thickness, self.color);
    //     // }
    //     self.draw_laser_texture();
    // }
    // 
    // pub unsafe fn collide_many(&self, other: &[Segment]) -> (Vec<Vec2>, Vec2) {
    //     let mut collision_points: Vec<Vec2> = Vec::new();
    //     let mut ray = self.ray;
    //     let mut ray_origin_segment: Option<&Segment> = None;
    // 
    // 
    //     // if ESTIMATE_IN_SECONDS {
    //     //     let start = Instant::now();
    //     //     let max_time = ESTIMATE_MILLIS as f64 / 1_000f64;
    //     //     while start.elapsed().as_secs_f64() < max_time {
    //     //         let (closest_point, ray_origin_new) = Self::find_closest_segment(ray, other, ray_origin_segment);
    //     //         ray_origin_segment = ray_origin_new;
    //     //         collision_points.push(closest_point.0);
    //     //         if matches!(closest_point.2, EdgeState::Absorptive) { break; }
    //     //         if closest_point.0.distance(ray.origin) > MAX_DISTANCE { break; }
    //     //         ray = Ray {
    //     //             origin: closest_point.0,
    //     //             direction: closest_point.1,
    //     //         };
    //     //     }
    //     // } else {
    //     for _ in 0..MAX_RAYS as u32 {
    //         let (closest_point, ray_origin_new) =
    //             Self::find_closest_segment(ray, other, ray_origin_segment);
    //         ray_origin_segment = ray_origin_new;
    //         collision_points.push(closest_point.0);
    //         if matches!(closest_point.2, EdgeState::Absorptive) { break; }
    //         if closest_point.0.distance(ray.origin) > Self::MAX_DISTANCE { break; }
    //         ray = Ray {
    //             origin: closest_point.0,
    //             direction: closest_point.1,
    //             color: RED,
    //         };
    //     }
    //     // }
    // 
    //     (collision_points, self.ray.direction)
    // }
    pub fn solve_collisions(&self, segments: &[Segment]) -> Vec<(Vec2, Vec2, Color)> {
        let ray = self.ray;
        let mut ray_stack: VecDeque<(Ray, Option<Segment>)> = [(ray, None)].into();
        let mut lines_stack: Vec<(Vec2, Vec2, Color)> = Vec::new();
        while let Some((ray, segment)) = ray_stack.pop_front() {
            // if ray.color.a <= f32::EPSILON { continue; }
            if ray.color.a <= 0.1f32 { continue; }
            debug_assert!(ray.direction.is_normalized(),
                          "ray not normal: {}, normal is {:?}, len is {:}",
                          ray.direction, ray.direction.normalize(), ray.direction.length());
            if let Some((collision, segment)) = Self::find_closest_segment_new(ray, segments, segment.as_ref()) {
                debug_assert!(collision.normal.is_normalized(),
                              "not normal: {}, normal is {:?} {:?}",
                              collision.normal, collision.normal.normalize(), collision);
                match segment.2 {
                    EdgeState::Reflective => {
                        ray_stack.push_back((Ray {
                            origin: collision.position,
                            direction: reflect(ray.direction, collision.normal),
                            color: ray.color, // TODO: use segment color
                        }, Some(*segment)));
                    }
                    EdgeState::Transparent => {
                        let is_critical = collision.normal.dot(ray.direction).abs().acos() == 0.8509;
                        let fresnel = ray.direction.dot(collision.normal).powi(6) * 0.97;
                        // debug!("{}", FresnelReflectAmount(1.0, 1.33, collision.normal, ray.direction));
                        ray_stack.push_back((Ray {
                            origin: collision.position,
                            direction: reflect(ray.direction, collision.normal),
                            color: {
                                if is_critical {
                                    ray.color
                                } else { (ray.color.to_vec() * (1.0 - fresnel)).to_array().into() }
                            }, // TODO: use segment color
                        }, Some(*segment)));
                        if !is_critical {
                            // debug!("refract {:}, arcsin {}", refract, (1.0f32 / 1.33f32).asin());
                            ray_stack.push_back((Ray {
                                origin: collision.position,
                                direction: ray.direction,
                                color: (ray.color.to_vec() * fresnel).to_array().into(), // TODO: use segment color
                            }, Some(*segment)));
                        }
                    }
                    EdgeState::Absorptive => {}
                }
                lines_stack.push((ray.origin, collision.position, ray.color));
            } else {
                lines_stack.push((ray.origin, ray.origin + ray.direction * Self::MAX_DISTANCE, ray.color));
            }
            if lines_stack.len() >= unsafe { MAX_RAYS as usize } { break; }
        }
        lines_stack
    }

    fn find_closest_segment_new<'a>(
        ray: Ray,
        segments: &'a [Segment],
        ray_origin_segment: Option<&'a Segment>,
    ) -> Option<(CollisionInfo, &'a Segment)> {
        let mut collision: CollisionInfo = CollisionInfo {
            position: ray.origin + ray.direction * Self::MAX_DISTANCE,
            normal: ray.direction,
        };
        let mut new_collision_segment: Option<&Segment> = None;

        for segment in segments.iter() {
            if let Some(origin_segment) = ray_origin_segment {
                if segment == origin_segment { continue; }
            }
            if let Some((col_position, col_normal)) = ray.collides_with((segment.0, segment.1)) {
                if ray.origin.distance_squared(col_position) < ray.origin.distance_squared(collision.position) {
                    new_collision_segment = Some(segment);
                    collision = CollisionInfo { position: col_position, normal: col_normal };
                }
            }
        }

        if let Some(segment) = new_collision_segment {
            Some((collision, segment))
        } else { None }
    }

    // fn find_closest_segment<'a>(
    //     ray: Ray,
    //     other: &'a [Segment],
    //     ray_origin_segment: Option<&'a Segment>,
    // ) -> (Segment, Option<&'a Segment>) {
    //     let mut closest_point: Segment = Segment(
    //         ray.origin + ray.direction * Self::MAX_DISTANCE,
    //         ray.direction,
    //         EdgeState::Reflective,
    //     );
    //     let mut ray_origin_new: Option<&Segment> = None;
    // 
    //     for segment in other.iter() {
    //         if let Some(origin_segment) = ray_origin_segment {
    //             if segment == origin_segment { continue; }
    //         }
    //         match Self::collide(ray, (segment.0, segment.1)) {
    //             Some((col_position, col_reflection)) => {
    //                 // if closest_point.0.distance(ray.origin) > col_position.distance(ray.origin) {
    //                 if closest_point.0.distance_squared(ray.origin) > col_position.distance_squared(ray.origin) {
    //                     ray_origin_new = Some(segment);
    //                     closest_point = Segment(col_position, col_reflection, segment.2);
    //                 }
    //             }
    //             None => {}
    //         }
    //     }
    // 
    //     (closest_point, ray_origin_new)
    // }
    // pub fn collide(ray: Ray, other: (Vec2, Vec2)) -> Option<(Vec2, Vec2)> {
    //     if let Some((pos, normal)) = ray.collides_with(other) {
    //         let reflection = ray.direction - (2.0 * normal.dot(ray.direction)) * normal;
    //         // draw_line(pos.x, pos.y,
    //         //           pos.x + reflection.x * 100.0,
    //         //           pos.y + reflection.y * 100.0,
    //         //           5.0, RED);
    //         // draw_circle(pos.x, pos.y, 10.0, BLUE);
    //         return Some((pos, reflection));
    //     }
    //     None
    // }
    pub fn look_at(&mut self, position: Vec2) {
        self.direction = position - self.position;
        self.ray.direction = self.direction.normalize_or_zero();
    }
}

impl Ray {
    pub fn collides_with(&self, other: (Vec2, Vec2)) -> Option<(Vec2, Vec2)> {
        let (start, end) = other;
        let ray_dir = self.direction.normalize_or_zero();
        let ray_dir_perp = ray_dir.perp();

        let start_to_origin = self.origin - start;
        let line_segment = end - start;
        let denominator = line_segment.dot(ray_dir_perp);

        if denominator.abs() < f32::EPSILON {
            return None; // Lines are parallel, no collision
        }

        let t1 = line_segment.perp_dot(start_to_origin) / denominator;
        let t2 = start_to_origin.dot(ray_dir_perp) / denominator;

        if t1 >= 0.0 && t2 >= 0.0 && t2 <= 1.0 {
            let collision = self.origin + ray_dir * t1;
            let normal_to_collision: Vec2 = if (collision - start).length_squared() <= f32::EPSILON {
                (collision - end).normalize().perp()
            } else {
                (collision - start).normalize().perp()
            };
            // draw_line(collision.x, collision.y, (collision.x + normal_to_collision.x * 100.0),
            // (collision.y + normal_to_collision.y * 100.0), 5.0, WHITE);
            Some((collision, normal_to_collision))
        } else {
            None
        }
    }
}

impl NodeNetwork {
    pub async fn new() -> Self {
        let texture = Texture2D::from_file_with_format(
            include_bytes!("../assets/node2.png"), Some(ImageFormat::Png));
        // let texture = load_texture("E:\\CLion\\ray_cast\\assets\\node2.png").await.unwrap();
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            texture,
            dragged_node: None,
            selected_node: None,
            key: 0,
        }
    }
    pub fn clean(&mut self) {
        self.nodes.clear();
        self.connections.clear();
        self.dragged_node = None;
        self.selected_node = None;
        self.key = 0;
    }
    pub unsafe fn update_camera(&mut self, camera_target: Vec2, zoom: f32) {
        CAMERA_TARGET = camera_target;
        ZOOM = zoom;
    }
    pub fn update(&mut self, _delta: f32) {
        self.handle_mouse();
        self.handle_selection();
        let mouse_pos = vec2tuple(mouse_position());
        if self.dragged_node.is_some() && is_mouse_button_released(MouseButton::Left) {
            if let Some(node_index) = self.dragged_node {
                if let Some(node) = self.nodes.get_mut(&node_index) {
                    node.is_dragged = false;
                }
            }
            self.dragged_node = None;
        }

        let mut is_some_hovered_node = false;
        for (i, node) in self.nodes.iter_mut() {
            node.update(_delta);
            node.is_hovered = node.contains(mouse_pos);
            if node.is_hovered {
                is_some_hovered_node = true;
                if is_mouse_button_pressed(MouseButton::Left)
                    && !node.is_dragged
                    && self.dragged_node.is_none() {
                    self.dragged_node = Some(*i);
                    node.is_dragged = true;
                }
            }
        }

        for edge in &mut self.connections {
            edge.update(_delta);
            let pos1 = self.nodes[&edge.a].position;
            let pos2 = match self.nodes.get(&edge.b) {
                Some(x) => x.position,
                None => {
                    error!("Edge b in ({:?}) does not exist", edge);
                    error!("{:?}", self.nodes);
                    Vec2::new(0.0, 0.0)
                }
            };
            edge.is_hovered = Self::point_line_collision(mouse_pos, pos1, pos2, edge.thickness);

            if edge.is_hovered && !is_some_hovered_node &&
                is_mouse_button_pressed(MouseButton::Left) {
                edge.cycle_state();
            }
        }
    }
    pub fn get_all_connections(&self) -> Vec<Segment> {
        let mut connections = Vec::with_capacity(self.connections.len());
        for edge in &self.connections {
            connections.push(Segment(self.nodes[&edge.a].position,
                                     self.nodes[&edge.b].position,
                                     edge.state));
        }
        connections
    }
    pub fn draw(&self) {
        for edge in &self.connections {
            edge.draw(self.nodes[&edge.a].position, self.nodes[&edge.b].position);
        }
        for (_, node) in &self.nodes {
            node.draw(&self.texture);
        }
    }
    fn handle_selection(&mut self) {
        if self.selected_node.is_none() { return; }
        let mp = vec2tuple(other_mouse_position());
        let node = &self.nodes[&self.selected_node.unwrap()];
        let mut new_mp = node.position;
        new_mp = Self::ctrl_shift(mp, node, &mut new_mp);
        let (node_x, node_y) = unsafe { world_to_screen((node.position.x, node.position.y)) };
        draw_line(new_mp.x, new_mp.y, node_x, node_y, 5.0, WHITE);
    }
    fn handle_mouse(&mut self) {
        if is_mouse_button_pressed(MouseButton::Right) && self.dragged_node.is_none() {
            let mouse_pos = vec2tuple(mouse_position());
            let mut selected_index = None;

            // Check if any node is clicked
            for (i, node) in self.nodes.iter() {
                if node.contains(mouse_pos) {
                    selected_index = Some(*i);
                    break;
                }
            }

            if let Some(selected_index) = selected_index {
                if self.selected_node == Some(selected_index) {
                    self.selected_node = None;
                } else if let Some(prev_selected_index) = self.selected_node {
                    self.add_connection(prev_selected_index, selected_index);
                    self.selected_node = None;
                } else {
                    self.selected_node = Some(selected_index);
                }
            } else {
                let mp = vec2tuple(mouse_position());
                let mut new_mp = mp;
                if self.selected_node.is_some() {
                    let node = &self.nodes[&self.selected_node.unwrap()];
                    new_mp = node.position;
                    new_mp = Self::ctrl_shift(mp, node, &mut new_mp);
                }
                let node_index = self.add_node(new_mp);
                if let Some(selected_index) = self.selected_node {
                    debug!("Adding connection from {} to {}", selected_index, node_index);
                    self.add_connection(selected_index, node_index);
                    self.selected_node = None;
                }
            }
        }


        if is_mouse_button_pressed(MouseButton::Middle)
            && self.dragged_node.is_none()
            && self.selected_node.is_none() {
            // Remove node or connection
            let mouse_pos = vec2tuple(mouse_position());
            for (i, node) in self.nodes.iter() {
                if node.contains(mouse_pos) {
                    self.remove_node(*i);
                    return;
                }
            }
            for (i, edge) in &mut self.connections.iter().enumerate() {
                let pos1 = self.nodes[&edge.a].position;
                let pos2 = self.nodes[&edge.b].position;
                if Self::point_line_collision(mouse_pos, pos1, pos2, edge.thickness) {
                    self.connections.remove(i);
                    // self.connections.retain(|edge| edge.a != edge.b && edge.a != edge.b);
                    return;
                }
            }
        }
    }

    fn ctrl_shift(mp: Vec2, node: &Node, new_mp: &Vec2) -> Vec2 {
        let mut new_mp = *new_mp;
        if is_key_down(KeyCode::LeftControl) {
            let diff = mp - node.position;
            if diff.x.abs() > diff.y.abs() { new_mp.x = mp.x } else { new_mp.y = mp.y }
        } else { new_mp = mp; }
        new_mp
    }
    fn point_line_collision(point: Vec2, line_start: Vec2, line_end: Vec2, thickness: f32) -> bool {
        let distance = point_to_line_distance(point, line_start, line_end);
        distance <= thickness / 2.0
    }

    fn remove_node(&mut self, index: usize) {
        if let Some(_) = self.nodes.get(&index) {
            // Remove the node from the connections vector
            self.connections.retain(|edge| edge.a != index && edge.b != index);

            // Remove the node itself
            self.nodes.remove(&index);
        }
        self.nodes.remove(&index);
    }
    pub fn add_node(&mut self, position: Vec2) -> usize {
        debug!("Added node at {:} keys: {}", position, self.key);
        self.nodes.insert(self.key, Node::new_default_radius(position));
        self.key += 1;
        self.key - 1
    }

    pub fn add_node_with_radius(&mut self, position: Vec2, radius: f32) -> usize {
        debug!("Added node at {:} keys: {} with radius {}", position, self.key, radius);
        self.nodes.insert(self.key, Node::new(position, radius));
        self.key += 1;
        self.key - 1
    }


    pub fn add_connection(&mut self, prev_conn: usize, cur_conn: usize) {
        if self.connections.iter().any(|edge|
        (edge.a == prev_conn && edge.b == cur_conn) ||
            (edge.a == cur_conn && edge.b == prev_conn)) {
            debug!("Connection already exists");
            return;
        }
        self.connections.push(Edge::new(prev_conn, cur_conn));
        debug!("Connection created between nodes {} and {}",
                        prev_conn, cur_conn);
    }
}

impl Node {
    pub fn new(position: Vec2, radius: f32) -> Self {
        Self {
            position,
            radius,
            color: WHITE,
            default_radius: radius,
            ..Default::default()
        }
    }
    pub fn new_default_radius(position: Vec2) -> Self {
        Self::new(position, 8.0)
    }
    pub fn contains(&self, position: Vec2) -> bool {
        (position - self.position).length_squared() <= self.radius.powi(2)
    }
    fn draw(&self, texture2d: &Texture2D) {
        draw_texture_ex(texture2d,
                        self.position.x - self.radius, self.position.y - self.radius,
                        self.color,
                        DrawTextureParams {
                            dest_size: Some(Vec2::new(self.radius * 2.0, self.radius * 2.0)),
                            ..core::default::Default::default()
                        });
    }
    fn update(&mut self, delta: f32) {
        self.handle_drag(delta);
        self.handle_hover(delta);
    }
    fn handle_hover(&mut self, delta: f32) {
        let target_radius: f32 = if
        self.is_hovered { self.default_radius * 2.0 } else { self.default_radius };

        self.radius = lerpf(self.radius, target_radius, delta / 0.10);
    }
    fn handle_drag(&mut self, delta: f32) {
        if self.is_dragged {
            lerp_color_in_place(&mut self.color, WHITE, delta / 0.10);
            let mouse_pos = vec2tuple(mouse_position());
            if is_key_down(KeyCode::LeftControl) {
                let diff = mouse_pos - self.dragged_start_pos;
                if diff.x.abs() > diff.y.abs() {
                    self.position.x = mouse_pos.x;
                } else { self.position.y = mouse_pos.y; }
            } else {
                self.position = mouse_pos;
            }
        } else {
            lerp_color_in_place(&mut self.color, WHITE, delta / 0.10);
            self.dragged_start_pos = self.position;
        };
    }
}


#[inline(always)]
#[must_use]
fn lerpf(from: f32, to: f32, t: f32) -> f32 {
    from + t * (to - from)
}

#[inline(always)]
#[must_use]
#[allow(dead_code)]
fn lerp_color(from: Color, to: Color, t: f32) -> Color {
    Color {
        r: lerpf(from.r, to.r, t),
        g: lerpf(from.g, to.g, t),
        b: lerpf(from.b, to.b, t),
        a: lerpf(from.a, to.a, t),
    }
}


fn lerp_color_in_place(from: &mut Color, to: Color, t: f32) {
    from.r = lerpf(from.r, to.r, t);
    from.g = lerpf(from.g, to.g, t);
    from.b = lerpf(from.b, to.b, t);
    from.a = lerpf(from.a, to.a, t);
}

#[inline(always)]
pub const fn vec2tuple((x, y): (f32, f32)) -> Vec2 {
    Vec2::new(x, y)
}
#[inline(always)]
pub const fn tuple2vec(vec: Vec2) -> (f32, f32) {
    (vec.x, vec.y)
}

pub fn rotate(direction: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(direction.x * cos - direction.y * sin, direction.x * sin + direction.y * cos)
}

pub fn reflect(direction: Vec2, normal: Vec2) -> Vec2 {
    let res = direction - (2.0 * normal * direction.dot(normal));
    debug_assert!(res.is_normalized(), "res direction not normal: {}, normal is {:?}", res, res.normalize());
    res.normalize()
}

pub fn refract(direction: Vec2, normal: Vec2, eta: f32) -> Option<Vec2> {
    let dot = direction.dot(normal);
    let k = 1.0 - eta.powi(2) * (1.0 - dot.powi(2));
    if k < 0.0 { return None; }
    Some(eta * direction - (eta * dot + k.sqrt()) * normal)
}

pub fn FresnelReflectAmount(n1: f32, n2: f32, normal: Vec2, incident: Vec2) -> f32
{
    // Schlick aproximation
    let mut r0 = (n1 - n2) / (n1 + n2);
    r0 *= r0;
    let mut cosX = normal.dot(incident);
    if n1 > n2
    {
        let n = n1 / n2;
        let sin_t2 = n * n * (1.0 - cosX * cosX);
        // Total internal reflection
        if sin_t2 > 1.0 {
            return 1.0;
        }
        cosX = (1.0 - sin_t2).sqrt();
    }
    let x = 1.0 - cosX;
    let mut ret = r0 + (1.0 - r0) * x * x * x * x * x;

    // adjust reflect multiplier for object reflectivity
    unsafe { ret = OBJECT_REFLECTIVITY + (1.0 - OBJECT_REFLECTIVITY) * ret; }
    return ret;
}

fn point_to_line_distance(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let segment_length_squared = (line_end - line_start).length_squared();
    if segment_length_squared == 0.0 { return (point - line_start).length(); }

    let t = ((point.x - line_start.x) * (line_end.x - line_start.x)
        + (point.y - line_start.y) * (line_end.y - line_start.y))
        / segment_length_squared;

    if t < 0.0 {
        return (point - line_start).length();
    }
    if t > 1.0 {
        return (point - line_end).length();
    }

    let projection = Vec2::new(
        line_start.x + t * (line_end.x - line_start.x),
        line_start.y + t * (line_end.y - line_start.y),
    );

    (point - projection).length()
}
