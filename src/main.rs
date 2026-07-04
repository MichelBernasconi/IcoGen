#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

use image::imageops::FilterType;
use rfd::FileDialog;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use std::thread;

struct AppState {
    input_files: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    
    let state = Rc::new(RefCell::new(AppState {
        input_files: Vec::new(),
        output_dir: None,
    }));
    
    // Browse Images
    let ui_weak = ui.as_weak();
    let state_clone = state.clone();
    ui.on_browse_images(move || {
        if let Some(paths) = FileDialog::new()
            .add_filter("Immagini", &["png", "jpg", "jpeg", "bmp"])
            .pick_files()
        {
            state_clone.borrow_mut().input_files = paths.clone();
            if let Some(ui) = ui_weak.upgrade() {
                if paths.len() == 1 {
                    ui.set_selected_files_text(paths[0].file_name().unwrap_or_default().to_string_lossy().to_string().into());
                } else {
                    ui.set_selected_files_text(format!("{} file selezionati", paths.len()).into());
                }
            }
        }
    });

    // Browse Output
    let ui_weak = ui.as_weak();
    let state_clone = state.clone();
    ui.on_browse_output(move || {
        if let Some(path) = FileDialog::new().pick_folder() {
            state_clone.borrow_mut().output_dir = Some(path.clone());
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_output_dir_text(path.display().to_string().into());
            }
        }
    });

    // Generate Icons
    let ui_weak = ui.as_weak();
    let state_clone = state.clone();
    ui.on_generate_icons(move || {
        let ui = match ui_weak.upgrade() {
            Some(ui) => ui,
            None => return,
        };
        
        let state = state_clone.borrow();
        let input_files = state.input_files.clone();
        let output_dir = match &state.output_dir {
            Some(d) => d.clone(),
            None => {
                ui.set_log_text("Errore: Nessuna cartella di output selezionata.".into());
                return;
            }
        };
        
        if input_files.is_empty() {
            ui.set_log_text("Errore: Nessuna immagine selezionata.".into());
            return;
        }

        ui.set_is_processing(true);
        ui.set_log_text(format!("Avvio elaborazione di {} file...\n", input_files.len()).into());
        
        let profile = ui.get_profile().to_string();
        let custom_sizes = ui.get_custom_sizes().to_string();
        let format = ui.get_output_format().to_string();
        let remove_bg = ui.get_remove_bg();
        let bg_tolerance = ui.get_bg_tolerance() as u8;

        let mut sizes = Vec::new();
        match profile.as_str() {
            "Android" => sizes.extend(&[36, 48, 72, 96, 144, 192]),
            "iOS" => sizes.extend(&[20, 29, 40, 58, 60, 76, 80, 87, 114, 120, 152, 167, 180, 1024]),
            "Favicon" => sizes.extend(&[16, 32, 48, 192, 512]),
            "Personalizzato" => {
                for s in custom_sizes.split(',') {
                    if let Ok(num) = s.trim().parse::<u32>() {
                        sizes.push(num);
                    }
                }
            }
            _ => {}
        }
        sizes.sort_unstable();
        sizes.dedup();

        let ui_handle = ui_weak.clone();
        
        thread::spawn(move || {
            let send_log = |msg: String| {
                let ui_handle = ui_handle.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        let mut current_log = ui.get_log_text().to_string();
                        current_log.push_str(&msg);
                        current_log.push('\n');
                        ui.set_log_text(current_log.into());
                    }
                });
            };

            if !output_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&output_dir) {
                    send_log(format!("Errore critico creazione cartella: {}", e));
                    let ui_handle = ui_handle.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() { ui.set_is_processing(false); }
                    });
                    return;
                }
            }

            for input_file in input_files {
                let original_name = input_file.file_stem().unwrap_or_default().to_string_lossy();
                send_log(format!("--- Processando {} ---", original_name));

                let mut img = match image::open(&input_file) {
                    Ok(i) => i,
                    Err(e) => {
                        send_log(format!("Errore caricamento {}: {}", original_name, e));
                        continue;
                    }
                };

                if remove_bg {
                    let mut rgba_img = img.to_rgba8();
                    if let Some(bg_pixel) = rgba_img.get_pixel_checked(0, 0) {
                        let bg_color = *bg_pixel;
                        for pixel in rgba_img.pixels_mut() {
                            let diff_r = (pixel[0] as i32 - bg_color[0] as i32).abs() as u8;
                            let diff_g = (pixel[1] as i32 - bg_color[1] as i32).abs() as u8;
                            let diff_b = (pixel[2] as i32 - bg_color[2] as i32).abs() as u8;
                            if diff_r <= bg_tolerance && diff_g <= bg_tolerance && diff_b <= bg_tolerance {
                                pixel[3] = 0;
                            }
                        }
                    }
                    img = image::DynamicImage::ImageRgba8(rgba_img);
                }

                for (index, &size) in sizes.iter().enumerate() {
                    let resized = img.resize(size, size, FilterType::Lanczos3);
                    let filename = format!("{}_icon_{:02}_{}x{}.{}", original_name, index + 1, size, size, format);
                    let out_path = output_dir.join(&filename);

                    match resized.save(&out_path) {
                        Ok(_) => send_log(format!("Salvato: {}", filename)),
                        Err(e) => send_log(format!("ERRORE {}: {}", filename, e)),
                    }
                }
            }

            send_log("Elaborazione completata con successo!".to_string());
            let ui_handle = ui_handle.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle.upgrade() { ui.set_is_processing(false); }
            });
        });
    });

    ui.run()
}
