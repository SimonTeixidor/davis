use color_quant::NeuQuant;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use std::io::{Error, Write};

pub fn to_sixel(width: u32, image: &DynamicImage, colors: usize) -> Result<Vec<u8>, Error> {
    let mut image_data = Vec::<u8>::new();
    to_sixel_writer(width, image, colors, &mut image_data)?;
    Ok(image_data)
}

// Copied from https://github.com/o2sh/onefetch/blob/master/src/onefetch/image_backends/sixel.rs,
// with some modifications.
pub fn to_sixel_writer<W: Write>(
    width: u32,
    image: &DynamicImage,
    colors: usize,
    mut output: W,
) -> Result<(), Error> {
    let image = image.resize(width, u32::MAX, FilterType::Lanczos3);

    let rgba_image = image.into_rgba8(); // convert the image to rgba samples
    let flat_samples = rgba_image.as_flat_samples();
    let color_map = NeuQuant::new(10, colors, flat_samples.as_slice());

    output.write_all(b"\x1BPq")?;
    output.write_all(format!("\"1;1;{};{}", rgba_image.width(), rgba_image.height()).as_bytes())?;

    for (i, pixel) in color_map.color_map_rgb().chunks(3).enumerate() {
        let color_multiplier = 100.0 / 255.0;
        write!(
            output,
            "#{};2;{};{};{}",
            i,
            (pixel[0] as f32 * color_multiplier) as u32,
            (pixel[1] as f32 * color_multiplier) as u32,
            (pixel[2] as f32 * color_multiplier) as u32
        )?;
    }

    // subtract 1 -> divide -> add 1 to round up the integer division
    for i in 0..((rgba_image.height() - 1) / 6 + 1) {
        let sixel_row = rgba_image.view(
            0,
            i * 6,
            rgba_image.width(),
            std::cmp::min(6, rgba_image.height() - i * 6),
        );

        let mut sixel_row = sixel_row
            .pixels()
            .map(|(x, y, p)| (color_map.index_of(&p.0), (x, y)))
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
