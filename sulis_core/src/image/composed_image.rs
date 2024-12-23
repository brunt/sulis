//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::io::{Error, ErrorKind};
use std::rc::Rc;

use serde::Deserialize;

use crate::image::simple_image::SimpleImageBuilder;
use crate::image::{Image, SimpleImage};
use crate::io::{DrawList, GraphicsRenderer};
use crate::resource::ResourceSet;
use crate::ui::AnimationState;
use crate::util::{invalid_data_error, Rect, Size};

const GRID_DIM: i32 = 3;
const GRID_LEN: i32 = GRID_DIM * GRID_DIM;

#[derive(Debug)]
pub struct ComposedImage {
    images: Vec<Rc<dyn Image>>,
    id: String,
    size: Size,
    middle_size: Size,
}

fn get_images_from_grid(
    grid: Vec<String>,
    resources: &ResourceSet,
) -> Result<Vec<Rc<dyn Image>>, Error> {
    grid.into_iter()
        .map(|id| {
            resources
                .images
                .get(&id)
                .map(Rc::clone)
                .ok_or_else(|| invalid_data_error(&format!("Unable to locate sub image {}", id)))
        })
        .collect::<Result<Vec<_>, _>>()
}

fn get_images_from_inline(
    grid: Vec<String>,
    sub_image_data: SubImageData,
    resources: &mut ResourceSet,
) -> Result<Vec<Rc<dyn Image>>, Error> {
    let size = sub_image_data.size;
    let spritesheet = sub_image_data.spritesheet;

    grid.into_iter()
        .map(|id| {
            let image_display = format!("{spritesheet}/{id}");
            let builder = SimpleImageBuilder {
                id: id.clone(),
                image_display,
                size,
            };
            let image = SimpleImage::generate(builder, resources)?;
            resources.images.insert(id, Rc::clone(&image));
            Ok(image)
        })
        .collect()
}

impl ComposedImage {
    pub fn generate(
        builder: ComposedImageBuilder,
        resources: &mut ResourceSet,
    ) -> Result<Rc<dyn Image>, Error> {
        if builder.grid.len() as i32 != GRID_LEN {
            return Err(invalid_data_error(&format!(
                "Composed image grid must be length {GRID_LEN}"
            )));
        }

        let images_vec = match builder.generate_sub_images {
            Some(sub_image_data) => {
                get_images_from_inline(builder.grid, sub_image_data, resources)?
            }
            None => get_images_from_grid(builder.grid, resources)?,
        };

        // Helper function to validate uniformity of dimensions in rows or columns
        fn validate_dimension<F>(
            images: &[Rc<dyn Image>],
            dim_size: i32,
            index_fn: F,
            dimension_name: &str,
        ) -> Result<i32, Error>
        where
            F: Fn(i32, i32) -> usize,
        {
            let mut total_dim = 0;
            for primary in 0..dim_size {
                let reference_size = images[index_fn(primary, 0)].get_size();
                let ref_dim = if dimension_name == "height" {
                    reference_size.height
                } else {
                    reference_size.width
                };

                for secondary in 0..dim_size {
                    let size = images[index_fn(primary, secondary)].get_size();
                    let dim = if dimension_name == "height" {
                        size.height
                    } else {
                        size.width
                    };

                    if dim != ref_dim {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "All images in {} {} {} must have the same {}",
                                dimension_name,
                                if dimension_name == "height" {
                                    "row"
                                } else {
                                    "column"
                                },
                                primary,
                                dimension_name
                            ),
                        ));
                    }
                }
                total_dim += ref_dim;
            }
            Ok(total_dim)
        }

        // Validate row heights
        let total_height = validate_dimension(
            &images_vec,
            GRID_DIM,
            |y, x| (y * GRID_DIM + x) as usize,
            "height",
        )?;

        // Validate column widths
        let total_width = validate_dimension(
            &images_vec,
            GRID_DIM,
            |x, y| (y * GRID_DIM + x) as usize,
            "width",
        )?;

        let middle_size = *images_vec.get((GRID_LEN / 2) as usize).unwrap().get_size();

        Ok(Rc::new(ComposedImage {
            images: images_vec,
            size: Size::new(total_width, total_height),
            middle_size,
            id: builder.id,
        }))
    }
}

impl Image for ComposedImage {
    fn append_to_draw_list(
        &self,
        draw_list: &mut DrawList,
        state: &AnimationState,
        rect: Rect,
        millis: u32,
    ) {
        let fill_width = rect.w - (self.size.width - self.middle_size.width) as f32;
        let fill_height = rect.h - (self.size.height - self.middle_size.height) as f32;

        let image = &self.images[0];
        let mut draw = Rect {
            x: rect.x,
            y: rect.y,
            w: image.get_width_f32(),
            h: image.get_height_f32(),
        };
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x += image.get_width_f32();
        let image = &self.images[1];
        draw.w = fill_width;
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x += fill_width;
        let image = &self.images[2];
        draw.w = image.get_width_f32();
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x = rect.x;
        draw.y += image.get_height_f32();
        let image = &self.images[3];
        draw.w = image.get_width_f32();
        draw.h = fill_height;
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x += image.get_width_f32();
        let image = &self.images[4];
        draw.w = fill_width;
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x += fill_width;
        let image = &self.images[5];
        draw.w = image.get_width_f32();
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x = rect.x;
        draw.y += fill_height;
        let image = &self.images[6];
        draw.w = image.get_width_f32();
        draw.h = image.get_height_f32();
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x += image.get_width_f32();
        let image = &self.images[7];
        draw.w = fill_width;
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x += fill_width;
        let image = &self.images[8];
        draw.w = image.get_width_f32();
        image.append_to_draw_list(draw_list, state, draw, millis);
    }

    fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        state: &AnimationState,
        rect: Rect,
        millis: u32,
    ) {
        let mut draw_list = DrawList::empty_sprite();
        self.append_to_draw_list(&mut draw_list, state, rect, millis);
        renderer.draw(draw_list);
    }

    fn get_width_f32(&self) -> f32 {
        self.size.width as f32
    }

    fn get_height_f32(&self) -> f32 {
        self.size.height as f32
    }

    fn get_size(&self) -> &Size {
        &self.size
    }

    fn id(&self) -> String {
        self.id.clone()
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct SubImageData {
    size: Size,
    spritesheet: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ComposedImageBuilder {
    id: String,
    grid: Vec<String>,
    generate_sub_images: Option<SubImageData>,
}
