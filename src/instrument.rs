use std::f32::consts::PI;

use cpal::traits::*;
use cpal::Device;
use std::collections::VecDeque;
use ringbuf::*;
use minirng::hash::*;

pub struct DelayLine {
    pub buf: VecDeque<f32>,
}
impl DelayLine {
    pub fn new(n: usize) -> Self {
        DelayLine { buf: vec![0.0; n].into() }
    }
    pub fn tick(&mut self, x: f32) -> f32 {
        let val = self.buf.pop_front();
        self.buf.push_back(x);
        val.unwrap()
    }
    pub fn resize(&mut self, n: f32) {
        let new_len = n.max(1.0) as usize;
        self.buf.resize(new_len, 0.0);
    }
}

#[derive(Clone)]
pub struct InstrumentParams {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

pub fn initialize_audio(initial: InstrumentParams) -> UIThreadContext {
    // init code goes here
    let (prod, cons) = RingBuffer::<InstrumentParams>::new(200).split();
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to retrieve default output device");
    println!("Output device : {}", device.name().expect("couldnt get device name (??? idk)"));
    let config = device.default_output_config().expect("failed to get default output config");
    println!("Default output config : {:?}", config);
    let sample_rate = config.sample_rate().0;
    let sample_format = config.sample_format();
    let channels = config.channels();

    let mut ac = AudioThreadContext {
        p: initial,
        cons,
        n: 0,
        phase: 0.0,
        fm_phase: 0.0,
        env_phase: 0.0,
        delay_line: vec![0.0; 10000].into(),
        seed: random_seed(),
    };

    let output_callback = move |output: &mut [f32], info: &cpal::OutputCallbackInfo| {
        ac.write_chunk(output, info);
    };

    let config = cpal::StreamConfig {
        channels: channels,
        sample_rate: config.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_output_stream(&config, output_callback, |_| panic!("error"), None),
        _ => panic!("unsupported"),
    }.expect("failed to make stream");
    stream.play().expect("failed to play stream");
    UIThreadContext {
        stream,
        prod,
    }
}

pub struct UIThreadContext {
    stream: cpal::Stream,
    prod: Producer<InstrumentParams>,   
}

impl UIThreadContext {
    pub fn send_struct(&mut self, p: InstrumentParams) {
        self.prod.push(p);
    }
}

pub struct AudioThreadContext {
    p: InstrumentParams,
    cons: Consumer<InstrumentParams>,
    n: u64,
    env_phase: f32,
    phase: f32,
    fm_phase: f32,
    delay_line: VecDeque<f32>,
    seed: u32,
}

impl AudioThreadContext {
    fn write_chunk(&mut self, output: &mut [f32], info: &cpal::OutputCallbackInfo) {
        while let Some(new_params) = self.cons.pop() {
            self.p = new_params;
            let et = ((self.p.e + 1.0) / 2.0) * 4.0;
            let new_len = (et.exp2()).max(1.0) as usize;
            // self.delay_line.resize(new_len, 0.0);
            self.delay_line = vec![0.0; new_len].into();
        }
        for sample in output.iter_mut() {
            *sample = self.next_sample();
        }
    }

    fn next_sample(&mut self) -> f32 {
        let w = next_f32(&mut self.seed);
        
        let a = self.p.a / 2.0 + 0.5;
        let b = self.p.b / 2.0 + 0.5;
        let c = self.p.c / 2.0 + 0.5;
        let d = self.p.d / 2.0 + 0.5;
        let e = self.p.e / 2.0 + 0.5;
        let f = self.p.f / 2.0 + 0.5;
        
        let y = w*d*2.0 + 2.0*c*self.delay_line.pop_front().unwrap();
        self.delay_line.push_back(y);
        
        let period = a * 2.0 + 0.1;

        let wn = 2.0 * PI / 44100.0;

        self.env_phase += wn * 1.0 / period;
        if self.env_phase > 2.0 * PI {
            self.env_phase -= 2.0*PI;
        }
        let env = if self.env_phase > 2.0*PI*b {
            0.0
        } else {
            1.0
        };

        y * 0.1 * f * env
    }
}

// more eg panning sin function etc
// fft view always good