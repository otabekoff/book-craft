//! Turns the book's icon (any PNG/JPEG/WebP/ICO) into every size Windows needs.

use std::io::Cursor;
use std::path::Path;

use anyhow::{Context, Result, bail};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};

use crate::project::Project;
use crate::samples;

pub struct IconSet {
    /// Square, at least 256×256.
    pub image: DynamicImage,
}

impl IconSet {
    pub fn load(project: &Project) -> Result<Self> {
        let path = project.config.meta.icon.trim();
        let image = if path.is_empty() {
            image::load_from_memory(&samples::default_icon_png(&project.config.meta.accent)?)?
        } else {
            let file = project.resolve(path)?;
            load_any(&file).with_context(|| format!("Reading the icon {path}"))?
        };
        Ok(Self { image: square(image) })
    }

    pub fn from_image(image: DynamicImage) -> Self {
        Self { image: square(image) }
    }

    pub fn png(&self, size: u32) -> Result<Vec<u8>> {
        encode_png(&self.image.resize_exact(size, size, FilterType::Lanczos3))
    }

    /// A multi-resolution `.ico` (16–256 px).
    pub fn ico(&self) -> Result<Vec<u8>> {
        let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
        for size in [16u32, 20, 24, 32, 40, 48, 64, 96, 128, 256] {
            let rgba = self.image.resize_exact(size, size, FilterType::Lanczos3).to_rgba8();
            let img = ico::IconImage::from_rgba_data(size, size, rgba.into_raw());
            dir.add_entry(ico::IconDirEntry::encode(&img)?);
        }
        let mut out = Vec::new();
        dir.write(&mut out)?;
        Ok(out)
    }

    /// The icon centred on a transparent canvas of `w`×`h`, filling `fill` of the shorter side.
    pub fn padded_png(&self, w: u32, h: u32, fill: f32) -> Result<Vec<u8>> {
        let side = ((w.min(h) as f32) * fill).round().max(1.0) as u32;
        let icon = self.image.resize_exact(side, side, FilterType::Lanczos3).to_rgba8();
        let mut canvas = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));
        image::imageops::overlay(&mut canvas, &icon, ((w - side) / 2) as i64, ((h - side) / 2) as i64);
        encode_png(&DynamicImage::ImageRgba8(canvas))
    }
}

fn load_any(path: &Path) -> Result<DynamicImage> {
    let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    if ext == "svg" {
        bail!("SVG icons are not supported yet — export the icon as a 1024×1024 PNG.");
    }
    let bytes = std::fs::read(path)?;
    if ext == "ico" {
        let dir = ico::IconDir::read(Cursor::new(&bytes))?;
        let best = dir
            .entries()
            .iter()
            .max_by_key(|e| e.width())
            .ok_or_else(|| anyhow::anyhow!("The .ico file is empty"))?;
        let img = best.decode()?;
        let rgba = RgbaImage::from_raw(img.width(), img.height(), img.rgba_data().to_vec())
            .ok_or_else(|| anyhow::anyhow!("Bad icon data"))?;
        return Ok(DynamicImage::ImageRgba8(rgba));
    }
    Ok(image::load_from_memory(&bytes)?)
}

/// Pads non-square images and scales small ones up to 256 px.
fn square(img: DynamicImage) -> DynamicImage {
    let (w, h) = img.dimensions();
    let side = w.max(h).max(256);
    if w == h && w >= 256 {
        return img;
    }
    let scale = side as f32 / w.max(h) as f32;
    let (nw, nh) = ((w as f32 * scale).round() as u32, (h as f32 * scale).round() as u32);
    let resized = img.resize_exact(nw.max(1), nh.max(1), FilterType::Lanczos3).to_rgba8();
    let mut canvas = RgbaImage::from_pixel(side, side, Rgba([0, 0, 0, 0]));
    image::imageops::overlay(&mut canvas, &resized, ((side - nw) / 2) as i64, ((side - nh) / 2) as i64);
    DynamicImage::ImageRgba8(canvas)
}

fn encode_png(img: &DynamicImage) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        let img = image::load_from_memory(&samples::default_icon_png("#336699").unwrap()).unwrap();
        let set = IconSet::from_image(img);
        let ico = set.ico().unwrap();
        let dir = ico::IconDir::read(Cursor::new(ico)).unwrap();
        assert_eq!(dir.entries().len(), 10);
        let wide = image::load_from_memory(&set.padded_png(310, 150, 0.8).unwrap()).unwrap();
        assert_eq!(wide.dimensions(), (310, 150));
        let rect = IconSet::from_image(DynamicImage::new_rgba8(100, 40));
        assert_eq!(rect.image.dimensions(), (256, 256));
    }
}
