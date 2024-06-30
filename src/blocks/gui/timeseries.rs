use std::collections::VecDeque;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use rustfft::num_complex::Complex32;
use rustfft::num_traits::FromPrimitive;
use rustfft::{self, FftPlanner};
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use crate::anyhow::Result;
use crate::gui::{Color, GuiWidget};
use crate::runtime::{
    Block, BlockMeta, BlockMetaBuilder, Kernel, MessageIo, MessageIoBuilder, Pmt, StreamIo,
    StreamIoBuilder, WorkIo,
};

/// Handle for a single line in a spectrum plot
pub struct TimeseriesLineHandle {
    receiver: Receiver<Vec<f64>>,
    /// Currently plotted values
    pub values: VecDeque<f64>,
    max_values: usize,
    /// Label for the line
    pub label: String,
    /// Color for the label
    pub color: Color,
}

/// GUI handle for a spectrum plot.
pub struct TimeseriesHandle {
    /// A handle for each line being plotted
    pub lines: Vec<TimeseriesLineHandle>,
    /// The plot's title, to be painted above the plot
    pub title: Option<String>,
}

/// Block that plots Fourier transforms of incoming samples.
///
/// Multiple input streams are supported. Center frequency is only used for
/// displaying the correct X-axis values in the UI, and can be updated from
/// somewhere else in the flowgraph and via user interaction like dragging.
pub struct Timeseries<T> {
    senders: Vec<Sender<Vec<f64>>>,
    phantom_data: PhantomData<T>,
}

impl TimeseriesHandle {
    /// Process changes received from the flowgraph block. Called by GUI
    /// implementations before drawing the UI.
    pub fn process_updates(&mut self) {
        for line in self.lines.iter_mut() {
            while let Ok(buffer) = line.receiver.try_recv() {
                line.values.extend(buffer);
            }

            while line.values.len() > line.max_values {
                line.values.pop_front();
            }
        }
    }
}

#[cfg(feature = "egui")]
impl egui::Widget for &mut TimeseriesHandle {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.process_updates();

        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));

        let width = ui.available_width();

        if let Some(title) = &self.title {
            ui.heading(title);
        }

        let mut plot = egui_plot::Plot::new(ui.next_auto_id())
            .set_margin_fraction(egui::Vec2::new(0.0, 0.1))
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .show_axes(egui::Vec2b::new(true, true))
            .y_axis_position(egui_plot::HPlacement::Right)
            .y_axis_width(3)
            .height(ui.available_height())
            .allow_double_click_reset(true)
            .reset();

        if self.lines.len() > 1 || self.lines[0].label != "in" {
            plot = plot.legend(egui_plot::Legend::default().position(egui_plot::Corner::LeftTop));
        }

        plot.show(ui, |plot_ui| {
            for line in self.lines.iter_mut() {
                if line.values.len() > 0 {
                    let points: Vec<_> = line.values
                        .iter()
                        .enumerate()
                        .map(|(i, x)| [i as f64, *x]) // TODO
                        .collect();
                    let line_color: egui::Color32 = line.color.into();
                    let egui_line = egui_plot::Line::new(points)
                        .name(&line.label)
                        .color(line_color);
                    plot_ui.line(egui_line);
                }
            }
        }).response
    }
}

impl GuiWidget for TimeseriesHandle {
    #[cfg(feature = "egui")]
    fn egui_ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(self)
    }

    //#[cfg(feature = "textplots")]
    //fn textplots_ui(&mut self, size: (u16, u16)) {
    //    use textplots::ColorPlot;

    //    self.process_updates();

    //    let min_freq = (self.center_frequency - self.sample_rate / 2.0) as f32;
    //    let max_freq = (self.center_frequency + self.sample_rate / 2.0) as f32;

    //    let line_points: Vec<Vec<_>> = self
    //        .lines
    //        .iter()
    //        .map(|line| {
    //            line.values
    //                .iter()
    //                .enumerate()
    //                .map(|(i, y)| {
    //                    (
    //                        (self.center_frequency
    //                            + ((i as f64 / line.values.len() as f64) - 0.5) * self.sample_rate)
    //                            as f32,
    //                        *y as f32,
    //                    )
    //                })
    //                .collect()
    //        })
    //        .collect();

    //    let line_shapes: Vec<_> = line_points
    //        .iter()
    //        .map(|points| textplots::Shape::Lines(&points))
    //        .collect();

    //    let mut chart = textplots::Chart::new_with_y_range(
    //        (size.0 as u32 - 10) * 2,
    //        (size.1 as u32 - 2) * 4,
    //        min_freq,
    //        max_freq,
    //        (self.min - (self.max - self.min) * 0.1) as f32,
    //        (self.max + (self.max - self.min) * 0.1) as f32,
    //    );
    //    let mut chart_ref = &mut chart;
    //    for (line, shape) in self.lines.iter_mut().zip(line_shapes.iter()) {
    //        chart_ref = chart_ref.linecolorplot(&shape, line.color.into());
    //    }

    //    chart_ref.display();
    //}
}

impl<T: FromPrimitive + Copy + Clone + Debug + Default + Send + Sync + 'static> Timeseries<T>
where
    Timeseries<T>: Kernel,
{
    /// Construct a new spectrum block
    pub fn new(
        senders: Vec<Sender<Vec<f64>>>,
        inputs: Vec<String>,
    ) -> Block {
        let mut stream_io = StreamIoBuilder::new();
        for l in &inputs {
            stream_io = stream_io.add_input::<T>(l);
        }

        Block::new(
            BlockMetaBuilder::new("Timeseries").build(),
            stream_io.build(),
            MessageIoBuilder::new().build(),
            Timeseries::<T> {
                senders,
                phantom_data: PhantomData,
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl Kernel for Timeseries<f64> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        for (i, sender) in self.senders.iter_mut().enumerate() {
            let input = unsafe { sio.input(i).slice::<f64>() };

            let _ = sender.try_send(input.into_iter().cloned().collect());
            sio.input(i).consume(input.len());
        }

        io.finished = self
            .senders
            .iter()
            .enumerate()
            .all(|(i, _)| sio.input(i).finished());

        Ok(())
    }
}

/// Builder for a [Timeseries]
///
/// If no lines are added manually, a single line is added with the default
/// input port name 'in'.
#[derive(Default)]
pub struct TimeseriesBuilder {
    lines: Vec<(String, Color)>,
    title: Option<String>,
}

impl TimeseriesBuilder {
    /// Start building a new spectrum plot
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Set the plot's title
    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Add a new line to the plot with the given color
    pub fn line(mut self, label: impl ToString, color: Color) -> Self {
        self.lines.push((label.to_string(), color));
        self
    }

    /// Build the block and return both the block and a handle
    /// for the corresponding GUI widget.
    ///
    /// Use if you want to handle drawing the UI yourself.
    pub fn build_detached(mut self) -> (Block, TimeseriesHandle) {
        if self.lines.len() == 0 {
            self.lines.push((
                "in".to_string(),
                Color {
                    r: 0xfa,
                    g: 0xbd,
                    b: 0x2f,
                },
            ));
        }

        let mut senders = Vec::new();
        let mut lines = Vec::new();
        let mut labels = Vec::new();
        for (label, color) in self.lines.into_iter() {
            let (sender, receiver) = channel(256);
            let line = TimeseriesLineHandle {
                receiver,
                values: VecDeque::new(),
                max_values: 10_000,
                label: label.clone(),
                color,
            };

            senders.push(sender);
            lines.push(line);
            labels.push(label);
        }

        let block = Timeseries::<f64>::new(senders, labels);

        let handle = TimeseriesHandle {
            lines,
            title: self.title,
        };

        (block, handle)
    }

    /// Build the block, leaving the GUI widget attached. In order to
    /// draw the UI, pass the flowgraph to [crate::gui::Gui::run].
    pub fn build(self) -> Block {
        let (mut block, handle) = self.build_detached();
        block.attach_gui_handle(Box::new(handle));
        block
    }
}
