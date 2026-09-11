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
            if let Some(bg_pixel) = rgba_img.get_pixel_checked(0, 0) {
                let bg_color = *bg_pixel;
                for pixel in rgba_img.pixels_mut() {
                    let diff_r = (pixel[0] as i32 - bg_color[0] as i32).abs() as u8;
                    let diff_g = (pixel[1] as i32 - bg_color[1] as i32).abs() as u8;
                    let diff_b = (pixel[2] as i32 - bg_color[2] as i32).abs() as u8;
                    if diff_r <= config.bg_tolerance && diff_g <= config.bg_tolerance && diff_b <= config.bg_tolerance {
                        pixel[3] = 0;
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
