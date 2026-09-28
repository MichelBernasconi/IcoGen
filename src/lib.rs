#![allow(non_snake_case)]

use image::imageops::FilterType;
use std::path::PathBuf;

pub struct IcoGenConfig {
    pub input_files: Vec<PathBuf>,
    pub output_dir: PathBuf,
    pub format: String,
    pub profile: String,
    pub custom_sizes: String,
    pub remove_bg: bool,
    pub bg_tolerance: u8,
}

fn load_image_or_svg(input_path: &std::path::Path) -> Result<image::DynamicImage, String> {
    let ext = input_path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if ext == "svg" {
        let svg_data = std::fs::read(input_path)
            .map_err(|e| format!("Failed to read SVG file: {}", e))?;
        
        let opt = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(&svg_data, &opt)
            .map_err(|e| format!("Failed to parse SVG: {}", e))?;
        
        let size = tree.size();
        let width = (size.width().ceil() as u32).max(1);
        let height = (size.height().ceil() as u32).max(1);

        let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
            .ok_or_else(|| "Failed to allocate pixmap for SVG rendering".to_string())?;

        resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

        let rgba_image = image::RgbaImage::from_raw(width, height, pixmap.take())
            .ok_or_else(|| "Failed to convert pixmap to image buffer".to_string())?;

        Ok(image::DynamicImage::ImageRgba8(rgba_image))
    } else {
        image::open(input_path).map_err(|e| e.to_string())
    }
}

pub fn generate_icons<F>(config: IcoGenConfig, mut log_callback: F) 
where 
    F: FnMut(String)
{
    let mut convert_only = false;
    let mut sizes = Vec::new();
    match config.profile.as_str() {
        "Android" => sizes.extend(&[36, 48, 72, 96, 144, 192]),
        "iOS" => sizes.extend(&[20, 29, 40, 58, 60, 76, 80, 87, 114, 120, 152, 167, 180, 1024]),
        "Favicon" => sizes.extend(&[16, 32, 48, 192, 512]),
        "Custom" => {
            for s in config.custom_sizes.split(',') {
                if let Ok(num) = s.trim().parse::<u32>() {
                    sizes.push(num);
                }
            }
        }
        "Convert Only" => convert_only = true,
        _ => {}
    }
    sizes.sort_unstable();
    sizes.dedup();

    if !config.output_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&config.output_dir) {
            log_callback(format!("Critical error creating folder: {}", e));
            return;
        }
    }

    for input_file in &config.input_files {
        let original_name = input_file.file_stem().unwrap_or_default().to_string_lossy();
        log_callback(format!("--- Processing {} ---", original_name));

        let mut img = match load_image_or_svg(input_file) {
            Ok(i) => i,
            Err(e) => {
                log_callback(format!("Error loading {}: {}", original_name, e));
                continue;
            }
        };

        if config.remove_bg {
            let mut rgba_img = img.to_rgba8();
            let (width, height) = rgba_img.dimensions();

            if width > 0 && height > 0 {
                // 1. Determine background color from border samples
                let mut sample_r: u64 = 0;
                let mut sample_g: u64 = 0;
                let mut sample_b: u64 = 0;
                let mut sample_count: u64 = 0;

                // Sample corners and borders
                for x in 0..width {
                    let p_top = rgba_img.get_pixel(x, 0);
                    let p_bottom = rgba_img.get_pixel(x, height - 1);
                    if p_top[3] > 10 {
                        sample_r += p_top[0] as u64;
                        sample_g += p_top[1] as u64;
                        sample_b += p_top[2] as u64;
                        sample_count += 1;
                    }
                    if p_bottom[3] > 10 {
                        sample_r += p_bottom[0] as u64;
                        sample_g += p_bottom[1] as u64;
                        sample_b += p_bottom[2] as u64;
                        sample_count += 1;
                    }
                }

                for y in 1..(height.saturating_sub(1)) {
                    let p_left = rgba_img.get_pixel(0, y);
                    let p_right = rgba_img.get_pixel(width - 1, y);
                    if p_left[3] > 10 {
                        sample_r += p_left[0] as u64;
                        sample_g += p_left[1] as u64;
                        sample_b += p_left[2] as u64;
                        sample_count += 1;
                    }
                    if p_right[3] > 10 {
                        sample_r += p_right[0] as u64;
                        sample_g += p_right[1] as u64;
                        sample_b += p_right[2] as u64;
                        sample_count += 1;
                    }
                }

                let (bg_r, bg_g, bg_b) = if sample_count > 0 {
                    (
                        (sample_r / sample_count) as f32,
                        (sample_g / sample_count) as f32,
                        (sample_b / sample_count) as f32,
                    )
                } else {
                    let p = rgba_img.get_pixel(0, 0);
                    (p[0] as f32, p[1] as f32, p[2] as f32)
                };

                // Distance metric helper: Euclidean distance in RGB space
                let color_dist = |p: &image::Rgba<u8>| -> f32 {
                    let dr = p[0] as f32 - bg_r;
                    let dg = p[1] as f32 - bg_g;
                    let db = p[2] as f32 - bg_b;
                    (dr * dr + dg * dg + db * db).sqrt()
                };

                let tolerance_threshold = (config.bg_tolerance as f32) * 1.732; // max possible Euclidean dist is 255 * sqrt(3) ~ 441.6
                let feather_range = 12.0f32; // smooth anti-aliased edge transition

                let mut visited = vec![false; (width * height) as usize];
                let mut queue = std::collections::VecDeque::with_capacity((width * 2 + height * 2) as usize);

                // Initialize BFS queue with all border pixels
                for x in 0..width {
                    queue.push_back((x, 0));
                    visited[x as usize] = true;

                    if height > 1 {
                        queue.push_back((x, height - 1));
                        visited[((height - 1) * width + x) as usize] = true;
                    }
                }

                for y in 1..(height.saturating_sub(1)) {
                    queue.push_back((0, y));
                    visited[(y * width) as usize] = true;

                    if width > 1 {
                        queue.push_back((width - 1, y));
                        visited[(y * width + width - 1) as usize] = true;
                    }
                }

                // Flood fill only connected background from outside
                while let Some((cx, cy)) = queue.pop_front() {
                    let p = rgba_img.get_pixel_mut(cx, cy);
                    let dist = color_dist(p);

                    if p[3] <= 5 || dist <= tolerance_threshold {
                        // Pixel is background
                        p[3] = 0;

                        // Expand to 4 neighbors
                        let neighbors = [
                            (cx.wrapping_sub(1), cy),
                            (cx + 1, cy),
                            (cx, cy.wrapping_sub(1)),
                            (cx, cy + 1),
                        ];

                        for (nx, ny) in neighbors {
                            if nx < width && ny < height {
                                let idx = (ny * width + nx) as usize;
                                if !visited[idx] {
                                    visited[idx] = true;
                                    queue.push_back((nx, ny));
                                }
                            }
                        }
                    } else if dist < tolerance_threshold + feather_range {
                        // Soft edge feathering on outer perimeter boundary
                        let factor = (dist - tolerance_threshold) / feather_range;
                        p[3] = ((p[3] as f32) * factor.clamp(0.0, 1.0)) as u8;
                    }
                }
            }

            img = image::DynamicImage::ImageRgba8(rgba_img);
        }

        if convert_only {
            let filename = format!("{}_converted.{}", original_name, config.format);
            let out_path = config.output_dir.join(&filename);

            match img.save(&out_path) {
                Ok(_) => log_callback(format!("Saved: {}", filename)),
                Err(e) => log_callback(format!("ERROR {}: {}", filename, e)),
            }
        } else {
            for (index, &size) in sizes.iter().enumerate() {
                let resized = img.resize(size, size, FilterType::Lanczos3);
                let filename = format!("{}_icon_{:02}_{}x{}.{}", original_name, index + 1, size, size, config.format);
                let out_path = config.output_dir.join(&filename);

                match resized.save(&out_path) {
                    Ok(_) => log_callback(format!("Saved: {}", filename)),
                    Err(e) => log_callback(format!("ERROR {}: {}", filename, e)),
                }
            }
        }
    }

    log_callback("Processing completed successfully!".to_string());
}
