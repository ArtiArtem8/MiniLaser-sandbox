use std::collections::HashMap;

use log::{error, info};
use macroquad::color::{BLACK, BLUE, Color, GREEN, RED, SKYBLUE, WHITE};
use macroquad::input::{is_mouse_button_pressed,
                       is_mouse_button_released,
                       mouse_position,
                       MouseButton};
use macroquad::math::Vec2;
use macroquad::prelude::{draw_texture, ImageFormat};
use macroquad::shapes::{draw_circle, draw_line, draw_poly_lines};
use macroquad::texture::{draw_texture_ex,
                         DrawTextureParams,
                         load_texture,
                         Texture2D};

#[derive(Clone, Default, Debug)]
pub struct Node {
    position: Vec2,
    radius: f32,
    color: Color,
    default_radius: f32,
    is_hovered: bool,
    is_dragged: bool,
}

#[derive(Clone, Debug)]
pub struct Edge {
    a: usize,
    b: usize,
    color: Color,
    thickness: f32,
    is_hovered: bool,
}

impl Edge {
    pub fn new(a: usize, b: usize) -> Self {
        Self { a, b, color: WHITE, thickness: 5.0, is_hovered: false }
    }

    fn draw(&self, start: Vec2, end: Vec2) {
        draw_line(start.x, start.y, end.x, end.y, self.thickness, self.color);
    }
    pub(crate) fn update(&mut self, delta: f32) {
        let target_color = if self.is_hovered { SKYBLUE } else { WHITE };
        lerp_color_in_place(&mut self.color, target_color, delta / 0.10);
    }
}

pub struct NodeNetwork {
    nodes: HashMap<usize, Node>,
    connections: Vec<Edge>,
    texture: Texture2D,
    dragged_node: Option<usize>,
    selected_node: Option<usize>,
    key: usize,
}

pub struct Ray {
    origin: Vec2,
    direction: Vec2,
}

pub struct Laser {
    position: Vec2,
    direction: Vec2,
    ray: Ray,
    color: Color,
    thickness: f32,
    texture: Texture2D,
}

impl Laser {
    pub fn new(position: Vec2, direction: Vec2) -> Self {
        Self {
            position,
            direction,
            ray: Ray { origin: position, direction },
            color: RED,
            thickness: 5.0,
            texture: Texture2D::from_file_with_format(
                include_bytes!("../assets/laser.png"),
                Some(ImageFormat::Png),
            ),
        }
    }


    fn draw_texture(&self) {
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
    pub fn draw(&self) {
        let end = self.ray.origin + self.ray.direction * 10_000.0;

        draw_line(self.position.x, self.position.y, end.x, end.y, self.thickness, self.color);
        self.draw_texture();
    }

    pub fn collide_many(&self, other: &Vec<(Vec2, Vec2)>) {
        for segment in other.iter() {
            self.collide(*segment);
        }
    }
    pub fn collide(&self, other: (Vec2, Vec2)) {
        let collision = self.ray.collides_with(other);
        if let Some(np) = collision {
            let pos = np.0;
            let normal = np.1;
            // info!("Collision at {:?}", pos);
            draw_circle(pos.x, pos.y, 10.0, BLUE);
        }
    }
    pub fn look_at(&mut self, position: Vec2) {
        self.direction = position - self.position;
        self.ray.direction = self.direction.normalize_or_zero();
    }
}

impl Ray {
    pub fn collides_with(&self, other: (Vec2, Vec2)) -> Option<(Vec2, Vec2)> {
        let (start, end) = other;
        // let ray_to_start = start - self.origin;
        // let ray_to_end = end - self.origin;
        let ray_dir = self.direction.normalize_or_zero();
        let ray_dir_perp = ray_dir.perp();

        let v1 = self.origin - start;
        let v2 = end - start;
        let v3 = ray_dir_perp;
        let t1 = v2.perp_dot(v1) / v2.dot(v3);
        let t2 = v1.dot(v3) / v2.dot(v3);
        return if t1 >= 0.0 && t2 >= 0.0 && t2 <= 1.0 {
            let collision = self.origin + ray_dir * t1;
            let mut normal_to_collision = (collision - start).normalize_or_zero().perp();
            if normal_to_collision.dot(ray_dir) > 0.0 {
                normal_to_collision = -normal_to_collision;
            }
            draw_line(collision.x, collision.y, (collision.x + normal_to_collision.x * 100.0), (collision.y + normal_to_collision.y * 100.0), 5.0, GREEN);
            Some((collision, normal_to_collision))
        } else {
            None
        };
        /*
        let seg1 = start - self.origin;
        let seg2 = end - self.origin;
        let ray_dir = self.direction.normalize_or_zero();
        let seg_cross = seg1.perp_dot(seg2);
        let dir_cross = ray_dir.perp_dot(seg2);

        // Check if the ray and segment are parallel
        if seg_cross.abs() < 1e-6 {
            return None;
        }

        let t = dir_cross / seg_cross;
        let u = seg1.perp_dot(ray_dir) / seg_cross;

        info!("t: {}, u: {}", t, u);
        // Check if the intersection point is within the segment and not behind the ray
        if t >= 0.0 && u >= 0.0 && u <= 1.0 {
            let intersection_point = start * t;
            return Some(Vec2::new(self.origin.x + intersection_point.x, self.origin.y + intersection_point.y));
        } else {
            return None;
        }


        let segment = end - start;
        let segment_perp = segment.perp();
        info!("determinant: {}", ray_dir.perp_dot(ray_to_end));
        let numerator = (start - self.origin).dot(segment_perp);
        let denominator = ray_dir.dot(segment_perp);

        if denominator.abs() <= f32::EPSILON {
            return None;
        }

        let t1 = numerator / denominator;
        if t1 < 0.0 || t1 > 1.0 {
            // info!("t1: {}", t1);
            return None;
        }
        let t2 = ray_to_start.dot(ray_dir_perp) / denominator;
        if t2 < 0.0/* || t1 + t2 > 1.0*/ {
            info!("t2: {}", t2);
            return None;
        }
        // let pos = self.origin - ray_dir + ray_dir * t1;
        let collision = self.origin + ray_dir * t1;
        return Some(collision);
        */
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
        for (i, node) in self.nodes.iter_mut() {
            node.update(_delta);
            node.is_hovered = node.contains(mouse_pos);
            if node.is_hovered {
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
                    error!("Edge b in ({edge:?}) does not exist");
                    error!("{:?}", self.nodes);
                    Vec2::new(0.0, 0.0)
                }
            };
            edge.is_hovered = Self::point_line_collision(mouse_pos, pos1, pos2, edge.thickness);
        }
    }
    pub fn get_all_connections(&self) -> Vec<(Vec2, Vec2)> {
        let mut connections = Vec::with_capacity(self.connections.len());
        for edge in &self.connections {
            connections.push((self.nodes[&edge.a].position, self.nodes[&edge.b].position));
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
        let mp = vec2tuple(mouse_position());
        let node = &self.nodes[&self.selected_node.unwrap()];
        draw_line(mp.x, mp.y, node.position.x, node.position.y, 5.0, WHITE);
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
                let node_index = self.add_node(vec2tuple(mouse_position()));

                if let Some(selected_index) = self.selected_node {
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
        info!("Added node at {:} keys: {}", position, self.key);
        self.nodes.insert(self.key, Node::new_default_radius(position));
        self.key += 1;
        return self.key - 1;
    }
    pub fn add_connection(&mut self, prev_conn: usize, cur_conn: usize) {
        if self.connections.iter().any(|edge|
            (edge.a == prev_conn && edge.b == cur_conn) ||
                (edge.a == cur_conn && edge.b == prev_conn)) {
            info!("Connection already exists");
            return;
        }
        self.connections.push(Edge::new(prev_conn, cur_conn));
        info!("Connection created between nodes {} and {}",
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
        // load_texture("assets/node.png").await.unwrap();
        // draw_circle(self.position.x, self.position.y, self.radius, self.default_color);
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
        let target_radius: f32;
        if self.is_hovered {
            target_radius = self.default_radius * 2.0;
        } else {
            target_radius = self.default_radius;
        };
        self.radius = lerpf(self.radius, target_radius, delta / 0.10);
    }
    fn handle_drag(&mut self, delta: f32) {
        if self.is_dragged {
            lerp_color_in_place(&mut self.color, WHITE, delta / 0.10);
            self.position = vec2tuple(mouse_position());
        } else {
            lerp_color_in_place(&mut self.color, WHITE, delta / 0.10);
        };
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }

    fn ne(&self, other: &Self) -> bool {
        self != other
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
