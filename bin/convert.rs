use ecalli_layout_backend::feature::AppError;
use image::{self, ExtendedColorType, ImageBuffer, ImageEncoder, LumaA, codecs::png::PngEncoder};
use jwalk::WalkDir;
use std::fs::{self, File};
use std::path::PathBuf;
use std::time::Instant;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

const SOURCE_PATH: &str = "assets/于右任標準草書字典";
const TARGET_DIR: &str = "NewCursiveDir";
const IMAGE_DIR: &str = "標草書";

fn create_transparent_png_zip(
    charname: char,
    new_file: File,
    input_paths: &[(u16, PathBuf)],
    threshold: u8,
) -> Result<(), AppError> {
    let mut zip_writer = ZipWriter::new(new_file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755);
    for (n, input_path) in input_paths {
        // Fetch 001 number.
        let frame_name = input_path.file_stem().unwrap().to_str().unwrap();
        let new_fname = format!("{frame_name}.png");
        let old_img = image::open(input_path)?.to_luma8();

        zip_writer.start_file(&new_fname, options)?;

        let (width, height) = old_img.dimensions();
        let mut lumaa_img = ImageBuffer::<LumaA<u8>, Vec<u8>>::new(width, height);

        for (x, y, p_luma8) in old_img.enumerate_pixels() {
            let luma_value = p_luma8.0[0];

            let alpha_value = if luma_value >= threshold {
                0 // Transparent (Background)
            } else {
                255 // Opaque (Drawing)
            };

            lumaa_img.put_pixel(x, y, LumaA([luma_value, alpha_value]));
        }

        let encoder = PngEncoder::new(&mut zip_writer);

        encoder.write_image(lumaa_img.as_raw(), width, height, ExtendedColorType::La8)?;

        if input_paths.len() == *n as usize {
            lumaa_img.save(format!("{IMAGE_DIR}/{charname}.png"))?;
        }
    }
    zip_writer.finish()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    for entry in WalkDir::new(SOURCE_PATH).min_depth(1) {
        let fname = entry.expect("failed to get entry path").path();
        let letter = fname
            .file_prefix()
            .and_then(|prefix| prefix.to_str())
            .and_then(|fname| fname.chars().last())
            .expect("Failed to extract last char of the file prefix: {fname}");

        if fname.is_dir() {
            let stage_start = Instant::now();
            let unfiltered_name = fname.file_prefix().unwrap().to_str().unwrap();
            println!("Checking if {unfiltered_name} contains {letter}...");
            let checked_dir = match fs::read_dir(format!("{SOURCE_PATH}/{unfiltered_name}")) {
                Ok(dir) => dir,
                Err(e) => {
                    println!("{e}");
                    continue;
                }
            };
            let mut sortable_paths: Vec<(u16, PathBuf)> = checked_dir
                .filter_map(|e| e.ok())
                .filter_map(|entry| {
                    let frame_num = entry.path().file_stem()?.to_str()?.parse().ok()?;

                    Some((frame_num, entry.path()))
                })
                .collect();
            sortable_paths.sort_by_key(|&(numeric_key, _)| numeric_key);
            let res_path = format!("{TARGET_DIR}/{letter}.zip");
            let new_file = File::create(res_path)?;
            create_transparent_png_zip(letter, new_file, &sortable_paths, 255)?;
            println!(
                "Complete the zip archive in {:?}, removing the missing char from the checklist.",
                stage_start.elapsed()
            );
            //fs::remove_file(format!("{checklist_dir_path}/{missing_char}.png"))?;
        }
    }

    Ok(())
}
