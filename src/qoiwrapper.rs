/* The tinyQOI implementation produces RGB888 images and the gc9a01
driver only supports RGB565 for now, so this is a wrapper which converts
the pixels on the fly.

Not super efficient but it gets things going.
*/

use embedded_graphics::{
    draw_target::{DrawTarget, DrawTargetExt},
    geometry::{Dimensions, OriginDimensions, Size},
    image::ImageDrawable,
    pixelcolor::Rgb565,
};
use tinyqoi::Qoi;

pub struct Wrapper<'a> {
    pub image: &'a Qoi<'a>,
}

impl OriginDimensions for Wrapper<'_> {
    fn size(&self) -> Size {
        self.image.size()
    }
}

impl ImageDrawable for Wrapper<'_> {
    type Color = Rgb565;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        target.fill_contiguous(
            &self.image.bounding_box(),
            self.image.pixels().map(|p| Rgb565::from(p)),
        )
    }

    fn draw_sub_image<D>(
        &self,
        target: &mut D,
        area: &embedded_graphics::primitives::Rectangle,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.draw(&mut target.translated(-area.top_left).clipped(area))
    }
}
