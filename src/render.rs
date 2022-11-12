use crate::{Canvas, Circle, RenderConfig};
use csv::Reader;
use image::Rgba;
use std::io::Write;

pub struct Render {
    config: RenderConfig,
    circles: Vec<Circle>,
}

impl Render {
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

    fn find_height(&self) -> u32 {
        let mut max = 0;

        for c in &self.circles {
            if c.y > max {
                max = c.y;
            }
        }

        max
    }

    fn find_width(&self) -> u32 {
        let mut max = 0;

        for c in &self.circles {
            if c.x > max {
                max = c.x;
            }
        }

        max
    }

    fn hex_color(circle: &Circle) -> String {
        format!("#{:02x?}{:02x?}{:02x?}", circle.r, circle.g, circle.b)
    }

    pub fn run(&self) {
        if let Some(path) = &self.config.svg {
            self.svg(path);
        }

        if let Some(path) = &self.config.png {
            self.png(path);
        }
    }

    fn svg(&self, path: &str) {
        let mut output_file = std::fs::File::create(path).unwrap();

        let mut output = vec![];
        let width = self.find_width();
        let height = self.find_height();

        output.push(format!(
            "<svg id=\"sedimentSvg\" overflow=\"hidden\" viewBox=\"0 0 {} {}\" preserveAspectRatio=\"xMidYMid meet\" xmlns=\"http://www.w3.org/2000/svg\">",
            width, height
        ));

        for c in &self.circles {
            output.push(format!(
                "\t<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" />",
                c.x,
                c.y,
                c.radius,
                Self::hex_color(c)
            ));
        }

        output.push("</svg>".to_owned());

        output_file.write_all(output.join("\n").as_bytes()).unwrap();
    }

    fn png(&self, path: &str) {
        let width = self.find_width();
        let height = self.find_height();
        let mut output = Canvas::new(width, height);

        for circle in &self.circles {
            let color = Rgba::from([circle.r, circle.g, circle.b, 255]);

            imageproc::drawing::draw_filled_circle_mut(
                &mut output.img,
                (circle.x as i32, circle.y as i32),
                circle.radius as i32,
                color,
            );
        }

        output.save(path);
    }
}
