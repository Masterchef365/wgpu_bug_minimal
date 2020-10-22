use crate::Vertex;

#[allow(unused)]
pub fn rainbow_cube() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, 1.0, 1.0], [1.0, 0.0, 1.0]),
    ];

    let indices = vec![
        0, 1, 3, 3, 1, 2, 1, 5, 2, 2, 5, 6, 5, 4, 6, 6, 4, 7, 4, 0, 7, 7, 0, 3, 3, 2, 7, 7, 2, 6,
        4, 5, 0, 0, 5, 1,
    ];

    (vertices, indices)
}

pub fn grid(size: i32, scale: f32) -> Vec<Vertex> {
    const LIGHT_GRAY: [f32; 3] = [0.3; 3];
    const DARK_GRAY: [f32; 3] = [0.15; 3];
    let mut vertices = Vec::new();
    let length = size as f32 * scale;
    for i in -size..=size {
        let color = if i.abs() % 10 == 0 {
            LIGHT_GRAY
        } else {
            DARK_GRAY
        };
        let i = i as f32 * scale;
        vertices.push(Vertex::new([-length, 0.0, i], color));
        vertices.push(Vertex::new([length, 0.0, i], color));
        vertices.push(Vertex::new([i, 0.0, -length], color));
        vertices.push(Vertex::new([i, 0.0, length], color));
    }
    vertices
}
