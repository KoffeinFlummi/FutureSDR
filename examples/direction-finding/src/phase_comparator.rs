use std::fmt::Debug;
use std::sync::Arc;

use rustfft::num_complex::Complex32;
use rustfft::num_traits::FromPrimitive;
use rustfft::{self, FftPlanner};
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use futuresdr::macros::async_trait;
use futuresdr::anyhow::Result;
use futuresdr::gui::{Color, GuiWidget};
use futuresdr::runtime::{
    Block, BlockMeta, BlockMetaBuilder, Kernel, MessageIo, MessageIoBuilder, Pmt, StreamIo,
    StreamIoBuilder, WorkIo,
};

/// Block that plots Fourier transforms of incoming samples.
///
/// Multiple input streams are supported. Center frequency is only used for
/// displaying the correct X-axis values in the UI, and can be updated from
/// somewhere else in the flowgraph and via user interaction like dragging.
pub struct PhaseComparator {
    plan: Arc<dyn rustfft::Fft<f32>>,
    scratch: Box<[Complex32]>,
    fft_size: usize,
    time_diff_iff: f64,
}

impl PhaseComparator {
    /// Construct a new spectrum block
    pub fn new(fft_size: usize) -> Block {

        let mut planner = FftPlanner::<f32>::new();
        let plan = planner.plan_fft_forward(fft_size);

        Block::new(
            BlockMetaBuilder::new("PhaseComparator").build(),
            StreamIoBuilder::new()
                .add_input::<Complex32>("in0")
                .add_input::<Complex32>("in1")
                .add_output::<f64>("out")
                .build(),
            MessageIoBuilder::new().build(),
            PhaseComparator {
                fft_size,
                plan,
                scratch: vec![Complex32::default(); fft_size * 10].into_boxed_slice(),
                time_diff_iff: 0.0,
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl Kernel for PhaseComparator {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let input1 = unsafe { sio.input(0).slice_mut::<Complex32>() };
        let input2 = unsafe { sio.input(1).slice_mut::<Complex32>() };
        let output = unsafe { sio.output(0).slice::<f64>() };

        let ilen = usize::min(input1.len(), input2.len());
        let num_chunks = usize::min(output.len(), ilen / self.fft_size);

        let m = num_chunks * self.fft_size;
        if m == 0 {
            return Ok(());
        }

        let mut output1 = vec![Complex32::default(); m];
        let mut output2 = vec![Complex32::default(); m];
        self.plan.process_outofplace_with_scratch(&mut input1[0..m], &mut output1[0..m], &mut self.scratch);
        self.plan.process_outofplace_with_scratch(&mut input2[0..m], &mut output2[0..m], &mut self.scratch);

        sio.input(0).consume(m);
        sio.input(1).consume(m);

        for (chunk1, chunk2) in output1.chunks(self.fft_size).zip(output2.chunks(self.fft_size)) {
            //let center_phase: Vec<_> = chunk
            //    .iter()
            //    .map(|c| c.arg())
            //    .collect();
            //let reordered = [&db[self.fft_size / 2..], &db[..self.fft_size / 2]].concat();
            //let _ = sender.try_send(reordered);
            let center_phase1 = chunk1[0].arg();
            let center_phase2 = chunk2[0].arg();

            let diff = (center_phase1 - center_phase2) as f64;
            let diff_ns = (1000.0 / 868.25) * (diff / (std::f64::consts::PI * 2.0));

            //const ALPHA: f64 = 0.999;
            //self.time_diff_iff = ALPHA * self.time_diff_iff + (1.0 - ALPHA) * diff_ns;
        }

        let phase_output = output1.chunks(self.fft_size)
            .zip(output2.chunks(self.fft_size))
            .map(|(chunk1, chunk2)| {
                let center_phase1 = chunk1[0].arg();
                let center_phase2 = chunk2[0].arg();
                let diff = (center_phase1 - center_phase2) as f64;
                let diff_ns = (1000.0 / 868.25) * (diff / (std::f64::consts::PI * 2.0));
                diff_ns
            })
            .collect::<Vec<_>>();

        output[..num_chunks].copy_from_slice(&phase_output);
        sio.output(0).produce(num_chunks);

        io.finished = sio.input(0).finished();

        Ok(())
    }
}
