use std::collections::HashMap;

use log::{error, info};
use macroquad::color::{Color, SKYBLUE, WHITE};
use macroquad::input::{is_mouse_button_pressed,
                       is_mouse_button_released,
                       mouse_position,
                       MouseButton};
use macroquad::math::Vec2;
use macroquad::shapes::draw_line;
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

impl NodeNetwork {
    pub async fn new() -> Self {
        let texture = load_texture("assets/node2.png").await.unwrap();
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
    /*
    if self.contains(vec2tuple(mouse_position())) {
        is_hovered = true;
        if is_mouse_button_pressed(MouseButton::Left) {
            is_dragged = true;
        }
    } else { is_hovered = false; }

    if is_mouse_button_released(MouseButton::Left) && self.is_dragged {
        is_dragged = false;
    }
    */
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
    fn add_node(&mut self, position: Vec2) -> usize {
        info!("Added node at {:} keys: {}", position, self.key);
        self.nodes.insert(self.key, Node::new_default_radius(position));
        self.key += 1;
        return self.key - 1;
    }
    fn add_connection(&mut self, prev_conn: usize, cur_conn: usize) {
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
