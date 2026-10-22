use crate::rasterizer::{Triangle, Vertex};

fn tri(coords: [(i32, i32); 3], color: [u8; 4]) -> Triangle {
    Triangle {
        vertices: coords.map(|(x, y)| Vertex { x, y }),
        color,
    }
}

pub fn scenes() -> Vec<Vec<Triangle>> {
    vec![
        vec![
            tri([(80, 80), (230, 105), (140, 245)], [255, 180, 70, 255]),
            tri([(300, 55), (740, 190), (390, 545)], [60, 190, 240, 255]),
        ],
        vec![
            tri([(80, 150), (360, 150), (200, 410)], [255, 160, 75, 255]),
            tri([(450, 120), (450, 470), (720, 300)], [90, 205, 190, 255]),
        ],
        vec![tri(
            [(135, 95), (645, 170), (385, 500)],
            [165, 135, 255, 255],
        )],
        vec![tri(
            [(385, 500), (645, 170), (135, 95)],
            [165, 135, 255, 255],
        )],
        vec![tri(
            [(-150, 100), (550, -100), (600, 530)],
            [255, 125, 170, 255],
        )],
        vec![tri(
            [(120, 120), (380, 270), (640, 420)],
            [235, 210, 75, 255],
        )],
    ]
}

pub fn names() -> &'static [&'static str] {
    &[
        "Pequeno y grande",
        "Arista horizontal y vertical",
        "Orden original",
        "Orden invertido",
        "Parcialmente fuera",
        "Degenerado (area cero)",
    ]
}
