#![feature(array_zip)]
use std::collections::{HashMap, HashSet};

use image::{ImageBuffer, RgbImage};
use rand::prelude::*;

type Color = [u8; 3];
type ColorBase = [u8; 3];

fn color_base_to_color(cb: ColorBase, color_size: u64) -> Color {
    cb.map(|cbc| (cbc as u64 * 255 / (color_size - 1)) as u8)
}
type ColorOffset = [i16; 3];
type Location = [usize; 2];
type LocationOffset = [isize; 2];
fn lo_length(lo: LocationOffset) -> isize {
    // TUNEABLE
    lo[0].pow(2) * 5 + lo[1].pow(2)
}

const FLAG: [Color; 4] = [[252, 244, 52], [255, 255, 255], [156, 89, 209], [0, 0, 0]];
fn make_image(scale: u64, num_seeds: usize, seed: u64) -> RgbImage {
    let mut rng = StdRng::seed_from_u64(seed);
    let size = scale.pow(3) as usize;
    let color_size = scale.pow(2);
    let total_pixels = scale.pow(6);
    let color_bases: Vec<ColorBase> = (0..total_pixels)
        .map(|n| {
            let r_base = n % color_size;
            let g_base = (n / color_size) % color_size;
            let b_base = n / color_size.pow(2);
            [r_base as u8, g_base as u8, b_base as u8]
        })
        .collect();
    let mut color_offsets: Vec<ColorOffset> = color_bases
        .iter()
        .map(|color| color.map(|c| c as i16))
        .flat_map(|color| {
            vec![
                [color[0], color[1], color[2]],
                [color[0], color[1], -color[2]],
                [color[0], -color[1], color[2]],
                [color[0], -color[1], -color[2]],
                [-color[0], color[1], color[2]],
                [-color[0], color[1], -color[2]],
                [-color[0], -color[1], color[2]],
                [-color[0], -color[1], -color[2]],
            ]
            .into_iter()
        })
        .collect();
    color_offsets
        .sort_by_key(|color_offset| color_offset.map(|c| (c as i64).pow(2)).iter().sum::<i64>());
    let mut location_offsets: Vec<LocationOffset> = (0..total_pixels)
        .flat_map(|n| {
            let i = (n as usize % size) as isize;
            let j = (n as usize / size) as isize;
            vec![[i, j], [i, -j], [-i, j], [-i, -j]].into_iter()
        })
        .collect();
    location_offsets.sort_by_key(|location_offset| lo_length(*location_offset));
    let mut grid: Vec<Vec<Option<Color>>> = vec![vec![None; size]; size];
    let mut color_base_to_location: HashMap<ColorBase, Location> = HashMap::new();
    let mut open_locs: HashSet<Location> = (0..total_pixels)
        .map(|n| [n as usize % size, n as usize / size])
        .collect();
    let mut seed_locs: Vec<Location> = vec![];
    // TUNEABLE
    let switchover = (total_pixels as f64 * 0.9) as usize;
    for i in 0..total_pixels as usize {
        let debug: Option<usize> = None;
        if let Some(freq) = debug {
            if i % freq == 0 {
                println!("{}/{}", i, total_pixels);
            }
            if i == switchover {
                println!("SWITCHOVER");
            }
        }
        let color_base = loop {
            let tmp_color_base = [
                rng.gen_range(0..color_size) as u8,
                rng.gen_range(0..color_size) as u8,
                rng.gen_range(0..color_size) as u8,
            ];
            let tmp_color = color_base_to_color(tmp_color_base, color_size);
            let mut min_distance = 3 * 255_usize.pow(2);
            for &flag_color in &FLAG {
                let mut single_distance = 0;
                for (ct, cf) in tmp_color.zip(flag_color) {
                    let flag_color_edge_distance = cf.min(255 - cf) as usize;
                    let raw_distance = if ct < cf { cf - ct } else { ct - cf } as usize;
                    let axis_distance = if raw_distance <= flag_color_edge_distance {
                        // TUNEABLE
                        2 * raw_distance
                    } else {
                        raw_distance + flag_color_edge_distance
                    };
                    single_distance += axis_distance.pow(2);
                }
                min_distance = min_distance.min(single_distance);
            }
            // TUNEABLE
            let target_distance = 3 * 30_usize.pow(2);
            if min_distance < target_distance {
                break tmp_color_base;
            }
        };
        // Place seeds near desired colors in desired places
        if i < num_seeds {
            let (flag_index, _) = FLAG
                .iter()
                .enumerate()
                .min_by_key(|(_i, c)| {
                    c.zip(color_base_to_color(color_base, color_size))
                        .into_iter()
                        .map(|(flag_color, target_color)| {
                            (flag_color as isize - target_color as isize).pow(2)
                        })
                        .sum::<isize>()
                })
                .expect("A color");
            let mut row;
            let mut col;
            loop {
                let row_low = flag_index * size / 4;
                let row_high = (flag_index + 1) * size / 4;
                row = rng.gen_range(row_low..row_high);
                col = rng.gen_range(0..size);
                let mut too_close = false;
                for loc in &seed_locs {
                    let dist_sq: isize = loc
                        .zip([row, col])
                        .map(|(l1, l2)| {
                            let il1 = l1 as isize;
                            let il2 = l2 as isize;
                            (il1 - il2)
                                .abs()
                                .min(il1 - il2 + size as isize)
                                .min(il1 - il2 + size as isize)
                        })
                        .map(|d| d.pow(2))
                        .iter()
                        .sum::<isize>();
                    let dist: f64 = (dist_sq as f64).sqrt();
                    let min_spacing = size as f64 / (2.0 * (num_seeds as f64).sqrt());
                    if dist < min_spacing {
                        too_close = true;
                    }
                }
                if !too_close {
                    break;
                }
            }
            grid[row][col] = Some(color_base_to_color(color_base, color_size));
            color_base_to_location.insert(color_base, [row, col]);
            seed_locs.push([row, col]);
            open_locs.remove(&[row, col]);
            continue;
        }
        let most_similar_location: Location = color_offsets
            .iter()
            .filter_map(|color_offset| {
                let prov_new_color_base =
                    color_base.zip(*color_offset).map(|(c, co)| c as i16 + co);
                if prov_new_color_base.iter().any(|&c| c < 0 || c > 255) {
                    None
                } else {
                    let new_color_base = prov_new_color_base.map(|c| c as u8);
                    color_base_to_location.get(&new_color_base).copied()
                }
            })
            .next()
            .unwrap();
        // Find most similar opening
        let mut closest_location = None;
        if i < switchover {
            'offsets: for &location_offset in &location_offsets {
                let new_location = most_similar_location
                    .zip(location_offset)
                    .map(|(x, dx)| x as isize + dx);
                for x in new_location {
                    if x < 0 || x >= size as isize {
                        continue 'offsets;
                    }
                }
                let new_location = new_location.map(|x| x as usize);
                if grid[new_location[0]][new_location[1]].is_none() {
                    closest_location = Some(new_location);
                    break;
                }
            }
        } else {
            closest_location = open_locs
                .iter()
                .min_by_key(|&&[r, c]| {
                    let dr = r as isize - most_similar_location[0] as isize;
                    let dc = c as isize - most_similar_location[1] as isize;
                    lo_length([dr, dc])
                })
                .copied()
        }
        if closest_location.is_none() {
            println!(
                "{} loc {:?} color_base {:?}",
                i, most_similar_location, color_base
            );
            continue;
        }
        let closest_location = closest_location.expect("Found a slot");
        grid[closest_location[0]][closest_location[1]] =
            Some(color_base_to_color(color_base, color_size));
        color_base_to_location.insert(color_base, closest_location);
        open_locs.remove(&closest_location);
    }
    let mut img: RgbImage = ImageBuffer::new(size as u32, size as u32);
    for (i, row) in grid.into_iter().enumerate() {
        for (j, color) in row.into_iter().enumerate() {
            if let Some(color) = color {
                img.put_pixel(j as u32, i as u32, image::Rgb(color))
            } else {
                println!("Missing pixel: {:?}", [i, j]);
            }
        }
    }
    img
}

fn main() {
    for seed in 0..10 {
        let scale = 9;
        let num_seeds = 30;
        let filename = format!("img-{}-{}-{}.png", scale, num_seeds, seed);
        println!("Start {}", filename);
        let img = make_image(scale, num_seeds, seed);
        img.save(&filename).unwrap();
    }
}
