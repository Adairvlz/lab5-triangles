pub const WIDTH: i32 = 800;
pub const HEIGHT: i32 = 600;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriangleMode {
    Wireframe,
    FilledBarycentric,
    FilledScanline,
}

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub vertices: [Vertex; 3],
    pub color: [u8; 4],
}

#[derive(Clone, Copy, Debug)]
pub struct Fragment {
    pub x: i32,
    pub y: i32,
    pub color: [u8; 4],
}

// Se trabaja con coordenadas duplicadas para evaluar centros exactamente,
// sin errores de coma flotante: (2x+1, 2y+1).
fn edge(a: Vertex, b: Vertex, px2: i64, py2: i64) -> i64 {
    (px2 - 2 * a.x as i64) * (2 * (b.y - a.y) as i64)
        - (py2 - 2 * a.y as i64) * (2 * (b.x - a.x) as i64)
}

fn area2(a: Vertex, b: Vertex, c: Vertex) -> i64 {
    (b.x as i64 - a.x as i64) * (c.y as i64 - a.y as i64)
        - (b.y as i64 - a.y as i64) * (c.x as i64 - a.x as i64)
}

fn is_inside(a: Vertex, b: Vertex, c: Vertex, x: i32, y: i32) -> bool {
    let px2 = 2 * x as i64 + 1;
    let py2 = 2 * y as i64 + 1;
    let e0 = edge(a, b, px2, py2);
    let e1 = edge(b, c, px2, py2);
    let e2 = edge(c, a, px2, py2);
    (e0 >= 0 && e1 >= 0 && e2 >= 0) || (e0 <= 0 && e1 <= 0 && e2 <= 0)
}

fn bounds(t: &Triangle) -> Option<(i32, i32, i32, i32)> {
    let [a, b, c] = t.vertices;
    if area2(a, b, c) == 0 {
        return None;
    }
    let min_x = a.x.min(b.x).min(c.x).max(0);
    let max_x = a.x.max(b.x).max(c.x).min(WIDTH - 1);
    let min_y = a.y.min(b.y).min(c.y).max(0);
    let max_y = a.y.max(b.y).max(c.y).min(HEIGHT - 1);
    if min_x > max_x || min_y > max_y {
        None
    } else {
        Some((min_x, max_x, min_y, max_y))
    }
}

/// Calcula las coordenadas baricéntricas normalizadas (w0, w1, w2) para el centro del píxel (x + 0.5, y + 0.5).
/// Devuelve None si el triángulo es degenerado (área == 0).
#[allow(dead_code)]
pub fn barycentric_coords(
    a: Vertex,
    b: Vertex,
    c: Vertex,
    x: i32,
    y: i32,
) -> Option<(f64, f64, f64)> {
    let px2 = 2 * x as i64 + 1;
    let py2 = 2 * y as i64 + 1;
    let e0 = edge(b, c, px2, py2);
    let e1 = edge(c, a, px2, py2);
    let e2 = edge(a, b, px2, py2);
    let total = e0 + e1 + e2;
    if total == 0 {
        return None;
    }
    let total_f = total as f64;
    let w0 = e0 as f64 / total_f;
    let w1 = e1 as f64 / total_f;
    let w2 = e2 as f64 / total_f;
    Some((w0, w1, w2))
}

#[allow(dead_code)]
pub fn barycentric_triangle(a: Vertex, b: Vertex, c: Vertex, color: [u8; 4]) -> Vec<Fragment> {
    barycentric(&Triangle {
        vertices: [a, b, c],
        color,
    })
}

#[allow(dead_code)]
pub fn scanline_triangle(a: Vertex, b: Vertex, c: Vertex, color: [u8; 4]) -> Vec<Fragment> {
    scanline(&Triangle {
        vertices: [a, b, c],
        color,
    })
}

pub fn barycentric(t: &Triangle) -> Vec<Fragment> {
    let Some((min_x, max_x, min_y, max_y)) = bounds(t) else {
        return Vec::new();
    };
    let [a, b, c] = t.vertices;
    let mut result = Vec::new();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if is_inside(a, b, c, x, y) {
                result.push(Fragment {
                    x,
                    y,
                    color: t.color,
                });
            }
        }
    }
    result
}

pub fn scanline(t: &Triangle) -> Vec<Fragment> {
    let Some((min_x, max_x, min_y, max_y)) = bounds(t) else {
        return Vec::new();
    };
    let [a, b, c] = t.vertices;
    let edges = [(a, b), (b, c), (c, a)];
    let mut result = Vec::new();

    for y in min_y..=max_y {
        let sample_y = y as f64 + 0.5;
        let mut intersections = Vec::with_capacity(3);
        for (p, q) in edges {
            if p.y == q.y {
                // Una arista horizontal situada en y entero no cruza un centro y+0.5.
                continue;
            }
            let low = p.y.min(q.y) as f64;
            let high = p.y.max(q.y) as f64;
            // Regla semiabierta para intersecciones de vértices.
            if sample_y >= low && sample_y < high {
                let ratio = (sample_y - p.y as f64) / (q.y - p.y) as f64;
                intersections.push(p.x as f64 + ratio * (q.x - p.x) as f64);
            }
        }
        if intersections.len() < 2 {
            continue;
        }
        intersections.sort_by(f64::total_cmp);
        let left = intersections[0];
        let right = *intersections.last().unwrap();
        // Expandimos un píxel por seguridad numérica. is_inside decide
        // con enteros y la misma regla exacta que el otro algoritmo.
        let start = (((left - 0.5).ceil() as i32) - 1).max(min_x);
        let end = (((right - 0.5).floor() as i32) + 1).min(max_x);
        for x in start..=end {
            if is_inside(a, b, c, x, y) {
                result.push(Fragment {
                    x,
                    y,
                    color: t.color,
                });
            }
        }
    }
    result
}

fn line(a: Vertex, b: Vertex, color: [u8; 4]) -> Vec<Fragment> {
    let (mut x, mut y) = (a.x, a.y);
    let dx = (b.x - a.x).abs();
    let sx = if a.x < b.x { 1 } else { -1 };
    let dy = -(b.y - a.y).abs();
    let sy = if a.y < b.y { 1 } else { -1 };
    let mut error = dx + dy;
    let mut result = Vec::new();
    loop {
        if (0..WIDTH).contains(&x) && (0..HEIGHT).contains(&y) {
            result.push(Fragment { x, y, color });
        }
        if x == b.x && y == b.y {
            break;
        }
        let e2 = 2 * error;
        if e2 >= dy {
            error += dy;
            x += sx;
        }
        if e2 <= dx {
            error += dx;
            y += sy;
        }
    }
    result
}

pub fn wireframe(t: &Triangle) -> Vec<Fragment> {
    let [a, b, c] = t.vertices;
    let mut result = line(a, b, t.color);
    result.extend(line(b, c, t.color));
    result.extend(line(c, a, t.color));
    result
}

pub fn rasterize(triangles: &[Triangle], mode: TriangleMode) -> Vec<Fragment> {
    let mut result = Vec::new();
    for t in triangles {
        result.extend(match mode {
            TriangleMode::Wireframe => wireframe(t),
            TriangleMode::FilledBarycentric => barycentric(t),
            TriangleMode::FilledScanline => scanline(t),
        });
    }
    result
}

pub fn same_positions(first: &[Fragment], second: &[Fragment]) -> bool {
    let mut a: Vec<_> = first.iter().map(|f| (f.x, f.y)).collect();
    let mut b: Vec<_> = second.iter().map(|f| (f.x, f.y)).collect();
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_pixels_for_required_cases() {
        for triangles in crate::scenes::scenes() {
            let a = rasterize(&triangles, TriangleMode::FilledBarycentric);
            let b = rasterize(&triangles, TriangleMode::FilledScanline);
            assert!(same_positions(&a, &b), "Los algoritmos no coinciden");
        }
    }

    #[test]
    fn degenerate_is_empty() {
        let t = Triangle {
            vertices: [
                Vertex { x: 20, y: 20 },
                Vertex { x: 60, y: 60 },
                Vertex { x: 100, y: 100 },
            ],
            color: [255, 0, 0, 255],
        };
        assert!(barycentric(&t).is_empty());
        assert!(scanline(&t).is_empty());
        assert!(barycentric_coords(t.vertices[0], t.vertices[1], t.vertices[2], 50, 50).is_none());
    }

    #[test]
    fn test_barycentric_coords_weights() {
        let a = Vertex { x: 0, y: 0 };
        let b = Vertex { x: 10, y: 0 };
        let c = Vertex { x: 0, y: 10 };
        // Centro (2, 2)
        if let Some((w0, w1, w2)) = barycentric_coords(a, b, c, 2, 2) {
            assert!((w0 + w1 + w2 - 1.0).abs() < 1e-6);
            assert!(w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0);
        } else {
            panic!("Debe retornar coordenadas");
        }
        let t1 = barycentric_triangle(a, b, c, [255, 0, 0, 255]);
        let t2 = scanline_triangle(a, b, c, [255, 0, 0, 255]);
        assert_eq!(t1.len(), t2.len());
    }

    #[test]
    fn reversing_vertices_keeps_the_same_pixels() {
        let color = [255, 255, 255, 255];
        let original = Triangle {
            vertices: [
                Vertex { x: 135, y: 95 },
                Vertex { x: 645, y: 170 },
                Vertex { x: 385, y: 500 },
            ],
            color,
        };
        let reversed = Triangle {
            vertices: [
                original.vertices[2],
                original.vertices[1],
                original.vertices[0],
            ],
            color,
        };

        assert!(same_positions(
            &barycentric(&original),
            &barycentric(&reversed)
        ));
        assert!(same_positions(&scanline(&original), &scanline(&reversed)));
    }
}
