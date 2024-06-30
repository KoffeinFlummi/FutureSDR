#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use seify::{Device, DeviceTrait, RxStreamer, TxStreamer};

use futuresdr::anyhow::Result;
use futuresdr::blocks::gui::*;
use futuresdr::blocks::seify::SourceBuilder;
use futuresdr::blocks::{MessageCopy, Split, Combine, NullSink, FirBuilder};
use futuresdr::gui::{Gui, GuiFrontend};
use futuresdr::macros::connect;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;
use futuresdr::seify;
use futuresdr::seify::Direction;

mod phase_comparator;

use phase_comparator::*;

const SAMPLE_RATE: f64 = 2e6;
const FREQUENCY: f64 = 868.25e6;
const GAIN: f64 = 12.0;

fn find_usable_device() -> Result<Option<Device<Arc<dyn DeviceTrait<RxStreamer = Box<(dyn RxStreamer + 'static)>, TxStreamer = Box<(dyn TxStreamer + 'static)>> + Sync>>>> {
    for args in seify::enumerate()? {
        let device = seify::Device::from_args(args)?;
        let num_rx = device.num_channels(seify::Direction::Rx)?;
        if num_rx >= 2 {
            device.enable_agc(Direction::Rx, 1, false).unwrap();
            device.enable_agc(Direction::Rx, 0, false).unwrap();
            device.set_gain(Direction::Rx, 1, GAIN).unwrap();
            device.set_gain(Direction::Rx, 0, GAIN).unwrap();

            return Ok(Some(device));
        }
    }

    return Ok(None)
}

fn main() -> Result<()> {
    env_logger::init();

    let device = futuresdr::seify::Device::new().expect("Failed to find a Seify device");
    let freq_range = device.frequency_range(Direction::Rx, 0).unwrap();

    let mut fg = Flowgraph::new();

    let device = find_usable_device()?.unwrap();

    let src = SourceBuilder::new()
        .device(device.clone())
        .channels(vec![0, 1])
        .sample_rate(SAMPLE_RATE)
        .frequency(FREQUENCY)
        .build()?;

    let split11 = Split::new(|x: &Complex32| (*x, *x));
    let split12 = Split::new(|x: &Complex32| (*x, *x));

    let split21 = Split::new(|x: &Complex32| (*x, *x));
    let split22 = Split::new(|x: &Complex32| (*x, *x));

    let spectrum1 = SpectrumPlotBuilder::new(SAMPLE_RATE)
        .center_frequency(FREQUENCY)
        .fft_size(2048)
        .build();

    let spectrum2 = SpectrumPlotBuilder::new(SAMPLE_RATE)
        .center_frequency(FREQUENCY)
        .fft_size(2048)
        .build();

    let waterfall1 = WaterfallBuilder::new(SAMPLE_RATE)
        .center_frequency(FREQUENCY)
        .build();

    let waterfall2 = WaterfallBuilder::new(SAMPLE_RATE)
        .center_frequency(FREQUENCY)
        .build();

    let freq_min = freq_range.at_least(0.0).unwrap() / 1e6;
    let freq_max = freq_range.at_max(1e12).unwrap() / 1e6;
    let freq_slider = MessageSliderBuilder::<f64>::new(freq_min..=freq_max)
        .step_size(0.05)
        .initial_value(FREQUENCY / 1e6)
        .label("Frequency")
        .suffix("MHz")
        .multiplier(1e6)
        .build();

    let phase_comparator = PhaseComparator::new(2048);
    //let phase_filter = FirBuilder::new::<f64, f64, f64, [f64; 20]>([1.0; 20]);
    let output_plot = TimeseriesBuilder::new().build();

    // A whole bunch of blocks need access to the center frequency so we push all changes
    // in here for convenience, so we don't have to do point-to-point connections.
    let center_freq_hub = MessageCopy::new();

    connect!(fg,
             src.out1 > split11;
             src.out2 > split12;
             split11.out0 > phase_comparator.in0;
             split11.out1 > split21;
             split21.out0 > spectrum1;
             split21.out1 > waterfall1;
             split12.out0 > phase_comparator.in1;
             split12.out1 > split22;
             split22.out0 > spectrum2;
             split22.out1 > waterfall2;
             //phase_comparator > phase_filter > output_plot;
             phase_comparator > output_plot;
             freq_slider | center_freq_hub | freq_slider;
             waterfall1.drag_freq | center_freq_hub | waterfall1.center_freq;
             waterfall2.drag_freq | center_freq_hub | waterfall2.center_freq;
             spectrum1.drag_freq | center_freq_hub | spectrum1.center_freq;
             spectrum2.drag_freq | center_freq_hub | spectrum2.center_freq;
             center_freq_hub | src.freq);

    Gui::run(fg) // instead of rt.run(fg)
}
