use image::{imageops::FilterType, DynamicImage};
use imagequant;
use std::io::Write;

pub fn to_sixel(width: u32, image: &DynamicImage) -> Result<Vec<u8>, Error> {
    let mut image_data = Vec::<u8>::new();
    to_sixel_writer(width, image, &mut image_data)?;
    Ok(image_data)
}

pub fn to_sixel_writer<W: Write>(
    width: u32,
    image: &DynamicImage,
    mut output: W,
) -> Result<(), Error> {
    let image = image.resize(width, u32::MAX, FilterType::Lanczos3);

    let rgba_image = image.into_rgba8();

    let pixels = rgba_image
        .pixels()
        .map(|p| rgb::RGBA8 {
            r: p[0],
            g: p[1],
            b: p[2],
            a: p[3],
        })
        .collect::<Vec<_>>();

    let mut liq = imagequant::new();
    let mut img = liq
        .new_image(
            &*pixels,
            rgba_image.width() as usize,
            rgba_image.height() as usize,
            0.0,
        )
        .unwrap();
    let mut res = liq.quantize(&img).unwrap();
    res.set_dithering_level(1.0);
    let (palette, pixels) = res.remapped(&mut img).unwrap();

    output.write_all(b"\x1BPq")?;
    output.write_all(
        format!(
            "\"1;1;{};{}",
            rgba_image.width() as usize,
            rgba_image.height() as usize
        )
        .as_bytes(),
    )?;

    for (i, pixel) in palette.iter().enumerate() {
        let color_multiplier = 100.0 / 255.0;
        write!(
            output,
            "#{};2;{};{};{}",
            i,
            (pixel.r as f32 * color_multiplier) as u32,
            (pixel.g as f32 * color_multiplier) as u32,
            (pixel.b as f32 * color_multiplier) as u32
        )?;
    }

    // subtract 1 -> divide -> add 1 to round up the integer division
    for i in 0..((rgba_image.height() - 1) / 6 + 1) {
        let from = (i * rgba_image.width() * 6) as usize;
        let to = (((i + 1) * rgba_image.width() * 6) as usize).min(pixels.len());
        let to_coords = |j| {
            (
                j % rgba_image.width() as usize,
                j / rgba_image.width() as usize,
            )
        };
        let mut sixel_row = pixels[from..to]
            .iter()
            .enumerate()
            .map(|(j, p)| (p, to_coords(j)))
            .collect::<Vec<_>>();
        sixel_row.sort();

        for samples in Grouped(&*sixel_row, |r| r.0) {
            write!(output, "#{}", samples[0].0)?;

            // Group by x-pixel and OR together the y-bits.
            let bytes = Grouped(&*samples, |(_, (x, _))| x).map(|v| {
                (
                    v[0].1 .0 as i32,
                    v.iter()
                        .map(|(_, (_, y))| (1 << y))
                        .fold(0, |acc, x| acc | x),
                )
            });

            let mut last = -1;
            for (x, byte) in bytes {
                if last + 1 != x {
                    write!(output, "!{}", x - last - 1)?;
                    output.write_all(&[0x3f])?;
                }
                output.write_all(&[byte + 0x3f])?;
                last = x;
            }

            output.write_all(&[b'$'])?;
        }
        output.write_all(&[b'-'])?;
    }
    output.write_all(b"\x1B\\")?;
    Ok(())
}

struct Grouped<'a, K: Eq, T, F: Fn(T) -> K>(&'a [T], F);
impl<'a, K: Eq, T: Copy, F: Fn(T) -> K> Iterator for Grouped<'a, K, T, F> {
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }
        let mut i = 1;
        let mut iter = self.0.windows(2);
        while let Some([a, b]) = iter.next() {
            if (self.1)(*a) == (self.1)(*b) {
                i += 1
            } else {
                break;
            }
        }
        let (head, tail) = self.0.split_at(i);
        self.0 = tail;
        Some(head)
    }
}

pub enum Error {
    IoError(std::io::Error),
    LiqError(imagequant::liq_error),
}

impl From<imagequant::liq_error> for Error {
    fn from(e: imagequant::liq_error) -> Self {
        Error::LiqError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}
