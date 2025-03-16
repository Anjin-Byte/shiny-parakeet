use std::{sync::{mpsc::Sender, Arc, Mutex}, thread, time::Duration};

use eframe::{egui::{self, ColorImage, TextureHandle}, App, Frame};

use super::{command::RenderCommand, double_buffer::DoubleBufferReader};

pub struct MyApp {
    sender: Sender<RenderCommand>,
    db_reader: DoubleBufferReader<ColorImage>
}

impl MyApp {
    pub fn new(sender: Sender<RenderCommand>, db: DoubleBufferReader<ColorImage>) -> Self {
        Self { sender, db_reader: db }
    }
}

pub fn init(sender: Sender<RenderCommand>, db: DoubleBufferReader<ColorImage>, aspect_ratio: f64, view_width: u32)  -> eframe::Result {
    let view_height: u32 = {
        let height: u32 = (view_width as f64 / aspect_ratio) as u32;
        if height < 1 {
            1
        } else {
            height
        }
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([view_width as f32, view_height as f32]),
        ..Default::default()
    };

    eframe::run_native("My egui App", options, Box::new(|_| {
            Ok(Box::<MyApp>::new(MyApp::new(sender, db)))
        })
    )
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
        let _ = frame;
        println!("Update");
        let rendered_frame = self.db_reader.read().clone();
        let texture = ctx.load_texture("framebuffer",  rendered_frame, Default::default());

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Interrupt").clicked() {
                self.sender.send(RenderCommand::Interrupt).unwrap();
            }
            if ui.button("Pause").clicked() {
                self.sender.send(RenderCommand::Pause).unwrap();
            }
            if ui.button("Wake").clicked() {
                self.sender.send(RenderCommand::Wake).unwrap();
            }
            ui.image(&texture);
        });
    }
}
