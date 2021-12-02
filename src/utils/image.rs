use image::{error::ImageError, DynamicImage, ImageFormat};
use std::io::Write;
use webp::Encoder;

pub fn remove_files(paths: &[String]) {
    for path in paths {
        let _ = std::fs::remove_file(path);
    }
}

fn thumbnail(
    image: &DynamicImage,
    size: (u32, u32),
    path: &str,
    format: ImageFormat,
) -> Result<(), ImageError> {
    image
        .thumbnail(size.0, size.1)
        .save_with_format(path, format)?;

    Ok(())
}

fn webp_thumbnail(image: &DynamicImage, size: (u32, u32), path: &str) -> Result<(), ImageError> {
    let image_webp =
        Encoder::from_image(&image.resize(size.0, size.1, image::imageops::CatmullRom))
            .encode(100.0);
    let v = image_webp.iter().map(|a| *a).collect::<Vec<u8>>();

    match std::fs::File::create(path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(&v) {
                return Err(ImageError::IoError(e));
            }
        }
        Err(e) => return Err(ImageError::IoError(e)),
    }

    Ok(())
}

pub fn create_images(
    image: &DynamicImage,
    name: &str,
    max_mobile: Option<(u32, u32)>,
    max_desktop: Option<(u32, u32)>,
) -> Result<Vec<String>, ImageError> {
    let max_mobile = max_mobile.unwrap_or((500, 500));
    let max_desktop = max_desktop.unwrap_or((700, 700));
    let mut paths: Vec<String> = vec![];
    let mut new_name: String;
    let has_alpha = image.color().has_alpha();

    new_name = format!(
        "./uploads/mobile/{}.{}",
        name,
        if has_alpha { "png" } else { "jpg" }
    );
    if let Err(e) = thumbnail(
        &image,
        max_mobile,
        &new_name,
        if has_alpha {
            ImageFormat::Png
        } else {
            ImageFormat::Jpeg
        },
    ) {
        remove_files(&paths);

        return Err(e);
    }
    paths.push(new_name);

    new_name = format!(
        "./uploads/{}.{}",
        name,
        if has_alpha { "png" } else { "jpg" }
    );
    if let Err(e) = thumbnail(
        &image,
        max_desktop,
        &new_name,
        if has_alpha {
            ImageFormat::Png
        } else {
            ImageFormat::Jpeg
        },
    ) {
        remove_files(&paths);

        return Err(e);
    }
    paths.push(new_name);

    // Create webp format
    // for mobile
    new_name = format!("./uploads/mobile/{}.webp", name);
    if let Err(e) = webp_thumbnail(&image, max_mobile, &new_name) {
        remove_files(&paths);

        return Err(e);
    }
    paths.push(new_name);

    // for desktop
    new_name = format!("./uploads/{}.webp", name);
    if let Err(e) = webp_thumbnail(&image, max_desktop, &new_name) {
        remove_files(&paths);

        return Err(e);
    }
    paths.push(new_name);

    Ok(paths)
}

pub struct Uploader {
    files: Vec<String>,
}

impl Uploader {
    pub fn new() -> Uploader {
        Uploader { files: Vec::new() }
    }

    pub fn handle(
        &mut self,
        image: &DynamicImage,
        name: &str,
        max_mobile: Option<(u32, u32)>,
        max_desktop: Option<(u32, u32)>,
    ) -> Result<(), ImageError> {
        let max_mobile = max_mobile.unwrap_or((500, 500));
        let max_desktop = max_desktop.unwrap_or((700, 700));
        let mut paths: Vec<String> = vec![];
        let mut new_name: String;
        let has_alpha = image.color().has_alpha();

        new_name = format!(
            "./uploads/mobile/{}.{}",
            name,
            if has_alpha { "png" } else { "jpg" }
        );
        if let Err(e) = thumbnail(
            &image,
            max_mobile,
            &new_name,
            if has_alpha {
                ImageFormat::Png
            } else {
                ImageFormat::Jpeg
            },
        ) {
            remove_files(&paths);

            return Err(e);
        }
        paths.push(new_name);

        new_name = format!(
            "./uploads/{}.{}",
            name,
            if has_alpha { "png" } else { "jpg" }
        );
        if let Err(e) = thumbnail(
            &image,
            max_desktop,
            &new_name,
            if has_alpha {
                ImageFormat::Png
            } else {
                ImageFormat::Jpeg
            },
        ) {
            remove_files(&paths);

            return Err(e);
        }
        paths.push(new_name);

        // Create webp format
        // for mobile
        new_name = format!("./uploads/mobile/{}.webp", name);
        if let Err(e) = webp_thumbnail(&image, max_mobile, &new_name) {
            remove_files(&paths);

            return Err(e);
        }
        paths.push(new_name);

        // for desktop
        new_name = format!("./uploads/{}.webp", name);
        if let Err(e) = webp_thumbnail(&image, max_desktop, &new_name) {
            remove_files(&paths);

            return Err(e);
        }
        paths.push(new_name);

        self.files.append(&mut paths);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.files.clear();
    }
}

impl Drop for Uploader {
    fn drop(&mut self) {
        for path in &self.files {
            let _ = std::fs::remove_file(path);
        }
    }
}
