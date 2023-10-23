use crate::{optimizer::Optimizer, Canvas, Circle, RenderConfig};
use csv::Reader;
use image::Rgba;
use std::io::Write;

pub struct Render {
    config: RenderConfig,
    circles: Vec<Circle>,
}

impl Render {
    pub fn image_width(circles: &[Circle]) -> u32 {
        let mut max = 0;

        for c in circles {
            let new_x = c.x + c.radius;
            if new_x > max {
                max = new_x;
            }
        }

        max
    }

    fn image_height(circles: &[Circle]) -> u32 {
        let mut max = 0;

        for c in circles {
            let new_y = c.y + c.radius;
            if new_y > max {
                max = new_y;
            }
        }

        max
    }

    pub fn new(config: RenderConfig) -> Self {
        let mut csv = Reader::from_path(&config.input).unwrap();
        let mut circles = vec![];

        for line in csv.deserialize::<Circle>() {
            match line {
                Err(e) => {
                    panic!("Error reading {}: {}", config.input, e);
                }
                Ok(circle) => {
                    circles.push(circle);
                }
            }
        }

        Self { config, circles }
    }

    fn hex_color(circle: &Circle) -> String {
        format!("#{:02x?}{:02x?}{:02x?}", circle.r, circle.g, circle.b)
    }

    pub fn run(&self) {
        let optimizer = Optimizer::new(self.circles.clone());
        let pruned_circles = optimizer.parallel_prune();

        if let Some(path) = &self.config.svg {
            Self::svg_to_file(&pruned_circles, path);
        }

        if let Some(path) = &self.config.png {
            Self::png_to_file(&pruned_circles, path);
        }
    }

    pub fn render_svg(circles: &[Circle]) -> String {
        let mut output = vec![];
        let width = Self::image_width(circles);
        let height = Self::image_height(circles);

        output.push(format!(
            "<svg id=\"sedimentSvg\" overflow=\"hidden\" viewBox=\"0 0 {} {}\" preserveAspectRatio=\"xMidYMid meet\" xmlns=\"http://www.w3.org/2000/svg\">",
            width, height
        ));

        for c in circles {
            output.push(format!(
                "\t<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" />",
                c.x,
                c.y,
                c.radius,
                Self::hex_color(c)
            ));
        }

        output.push("</svg>".to_owned());
        output.join("\n")
    }

    fn svg_to_file(circles: &[Circle], path: &str) {
        let mut output_file = std::fs::File::create(path).unwrap();
        let raw_svg = Self::render_svg(circles);
        output_file.write_all(raw_svg.as_bytes()).unwrap();
    }

    pub fn render_raster(circles: &[Circle]) -> Canvas {
        let mut output = Self::create_empty_canvas(circles);

        for circle in circles {
            Self::add_raster_circle(&mut output, circle);
        }

        output
    }

    pub fn create_empty_canvas(circles: &[Circle]) -> Canvas {
        let width = Self::image_width(circles);
        let height = Self::image_height(circles);
        Canvas::new(width, height)
    }

    pub fn add_raster_circle(canvas: &mut Canvas, circle: &Circle) {
        let color = Rgba::from([circle.r, circle.g, circle.b, 255]);

        imageproc::drawing::draw_filled_circle_mut(
            &mut canvas.img,
            (circle.x as i32, circle.y as i32),
            circle.radius as i32,
            color,
        );
    }

    fn png_to_file(circles: &[Circle], path: &str) {
        Self::render_raster(circles).save(path);
    }
}
