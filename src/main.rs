mod rasterizer;
mod scenes;

use image::ColorType;
use rasterizer::{rasterize, same_positions, Fragment, TriangleMode, HEIGHT, WIDTH};
use raylib::prelude::*;
use std::hint::black_box;
use std::time::{Duration, Instant};

fn make_pixels(fragments: &[Fragment]) -> Vec<u8> {
    let mut pixels = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel.copy_from_slice(&[17, 22, 36, 255]);
    }
    for f in fragments {
        let index = ((f.y * WIDTH + f.x) * 4) as usize;
        pixels[index..index + 4].copy_from_slice(&f.color);
    }
    pixels
}

fn save_png(filename: &str, pixels: &[u8]) {
    std::fs::create_dir_all("capturas").expect("No se pudo crear capturas/");
    image::save_buffer(
        filename,
        pixels,
        WIDTH as u32,
        HEIGHT as u32,
        ColorType::Rgba8,
    )
    .expect("No se pudo guardar PNG");
    println!("Guardado: {filename}");
}

fn median(durations: &mut [Duration]) -> Duration {
    durations.sort();
    durations[durations.len() / 2]
}

fn benchmark(triangles: &[Vec<rasterizer::Triangle>], profile: bool) {
    let cases: Vec<_> = triangles.iter().flatten().copied().collect();
    let reps = if profile { 1200 } else { 100 };
    let trials = if profile { 1 } else { 9 };
    println!(
        "Triangulos: {}, repeticiones por muestra: {}",
        cases.len(),
        reps
    );

    let modes = [
        TriangleMode::FilledBarycentric,
        TriangleMode::FilledScanline,
    ];
    for mode in modes {
        let mut samples = Vec::new();
        let mut count = 0;
        for _ in 0..trials {
            let start = Instant::now();
            for _ in 0..reps {
                let fragments = rasterize(black_box(&cases), mode);
                count = fragments.len();
                black_box(fragments);
            }
            samples.push(start.elapsed());
        }
        let elapsed = median(&mut samples);
        println!("{mode:?}: {count} fragmentos, mediana para {reps} ejecuciones: {elapsed:?}, por ejecucion: {:.3} ms",
            elapsed.as_secs_f64() * 1000.0 / reps as f64);
    }
    let a = rasterize(&cases, TriangleMode::FilledBarycentric);
    let b = rasterize(&cases, TriangleMode::FilledScanline);
    println!(
        "Comparacion exacta: {}",
        if same_positions(&a, &b) {
            "IGUALES"
        } else {
            "DIFERENTES"
        }
    );
}

fn main() {
    let all_scenes = scenes::scenes();
    if std::env::args().any(|a| a == "--benchmark-triangles") {
        benchmark(&all_scenes, true);
        return;
    }
    if std::env::args().any(|a| a == "--measure") {
        benchmark(&all_scenes, false);
        return;
    }
    if std::env::args().any(|a| a == "--save-captures") {
        for (scene_idx, scene_tris) in all_scenes.iter().enumerate().take(1) {
            for &(m, name) in &[
                (TriangleMode::Wireframe, "wireframe"),
                (TriangleMode::FilledBarycentric, "barycentric"),
                (TriangleMode::FilledScanline, "scanline"),
            ] {
                let frags = rasterize(scene_tris, m);
                let px = make_pixels(&frags);
                save_png(
                    &format!("capturas/escena_{}_{}.png", scene_idx + 1, name),
                    &px,
                );
            }
        }
        return;
    }

    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT + 75)
        .title("Laboratorio - Rellenar triangulos")
        .build();
    rl.set_target_fps(60);
    let image = unsafe { Image::from_raw(raylib::ffi::GenImageColor(WIDTH, HEIGHT, Color::BLACK)) };
    let mut texture = rl
        .load_texture_from_image(&thread, &image)
        .expect("Error creando textura");
    let mut mode = TriangleMode::FilledBarycentric;
    let mut scene = 0usize;
    let mut previous = None;

    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_Q) {
            mode = TriangleMode::Wireframe;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_W) {
            mode = TriangleMode::FilledBarycentric;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_E) {
            mode = TriangleMode::FilledScanline;
        }
        for i in 0..6 {
            let key = match i {
                0 => KeyboardKey::KEY_ONE,
                1 => KeyboardKey::KEY_TWO,
                2 => KeyboardKey::KEY_THREE,
                3 => KeyboardKey::KEY_FOUR,
                4 => KeyboardKey::KEY_FIVE,
                _ => KeyboardKey::KEY_SIX,
            };
            if rl.is_key_pressed(key) {
                scene = i;
            }
        }
        if previous != Some((scene, mode)) {
            let fragments = rasterize(&all_scenes[scene], mode);
            let pixels = make_pixels(&fragments);
            texture
                .update_texture(&pixels)
                .expect("Error actualizando textura");
            let b = rasterize(&all_scenes[scene], TriangleMode::FilledBarycentric);
            let s = rasterize(&all_scenes[scene], TriangleMode::FilledScanline);
            println!(
                "Escena {} ({}) | modo {:?} | fragmentos {} | coinciden {}",
                scene + 1,
                scenes::names()[scene],
                mode,
                fragments.len(),
                same_positions(&b, &s)
            );
            previous = Some((scene, mode));
        }
        if rl.is_key_pressed(KeyboardKey::KEY_S) {
            let fragments = rasterize(&all_scenes[scene], mode);
            let pixels = make_pixels(&fragments);
            let name = match mode {
                TriangleMode::Wireframe => "wireframe",
                TriangleMode::FilledBarycentric => "barycentric",
                TriangleMode::FilledScanline => "scanline",
            };
            save_png(
                &format!("capturas/escena_{}_{}.png", scene + 1, name),
                &pixels,
            );
        }
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(25, 30, 42, 255));
        d.draw_texture(&texture, 0, 0, Color::WHITE);
        d.draw_text(
            "Q: contorno | W: baricentrico | E: scanline | 1-6: escenas | S: guardar PNG",
            14,
            HEIGHT + 10,
            16,
            Color::RAYWHITE,
        );
        d.draw_text(
            &format!(
                "Escena {}: {} | Modo: {:?}",
                scene + 1,
                scenes::names()[scene],
                mode
            ),
            14,
            HEIGHT + 37,
            17,
            Color::GOLD,
        );
    }
}
