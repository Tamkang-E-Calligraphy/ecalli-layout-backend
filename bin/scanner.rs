use ecalli_layout_backend::feature::{AppError, BlobStorageConfig};
use std::fs::{self};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let _config = BlobStorageConfig::from_local_env()?;

    /*
    let mut missing_file = File::create("MissingRegularFont.txt")?;
    let fonttype = CalliFont::Regular;
    for entry in fs::read_dir("RegularFont")? {
        if let Ok(direntry) = entry && let Some(char_name) = direntry.path().file_stem().and_then(|osname| osname.to_str()).and_then(|st| st.chars().next()) {
            let client = config.get_frame_client(&fonttype, char_name);
            if !client.exists().await? {
                writeln!(&mut missing_file, "{char_name}")?;
            }
        }
    }
    */

    let read_string = fs::read_to_string("MissingRegularFont.txt")?;

    println!("Missing {} words", read_string.chars().count());

    for char_name in read_string.chars() {
        for entry in fs::read_dir("assets/Epen/")? {
            let fname = entry?.path();
            if fname.is_file()
                && fname
                    .extension()
                    .is_some_and(|ext| ext.to_str().unwrap() == "stk")
                && let Some(name_prefix) = fname.file_stem().and_then(|oss| oss.to_str())
                && name_prefix == char_name.to_string().as_str()
            {
                let orig_path = format!("assets/Epen/{char_name}.stk");
                let new_path = format!("assets/missing_Epen/{char_name}.stk");

                println!("Copying {} to {} ...", &orig_path, &new_path);
                fs::copy(orig_path, new_path)?;
            }
        }
    }

    Ok(())
}
