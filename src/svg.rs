use crate::{builder::Circle, SvgConfig};
use csv::Reader;
use std::io::Write;

pub struct Svg {
    config: SvgConfig,
    circles: Vec<Circle>,
}

impl Svg {
    pub fn new(config: SvgConfig) -> Self {
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

    pub fn run(&mut self) {
        let mut output_file = std::fs::File::create(&self.config.output).unwrap();

        let mut output = vec![];
        let width = self.find_width();
        let height = self.find_height();

        output.push(format!(
            "<svg overflow=\"hidden\" width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">",
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
}