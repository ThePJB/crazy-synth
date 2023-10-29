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
        delay_line: DelayLine::new(1000),
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
    delay_line: DelayLine,
    seed: u32,
}

impl AudioThreadContext {
    fn write_chunk(&mut self, output: &mut [f32], info: &cpal::OutputCallbackInfo) {
        while let Some(new_params) = self.cons.pop() {
            self.p = new_params;
            self.delay_line.resize((self.p.e + 1.0) / 2.0 * 44100.0);
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
        
        let w = w * d;
        let out = self.delay_line.tick(w);
        
        let period = a * 2.0 + 0.1;
        let duty_cycle = b;
        let et = 5.0 + e * 9.0;
        let freq = et.exp2();
        let ct = c * 8.0;
        let fm_freq1 = ct.exp2();
        let dt = 5.0 + d * 9.0;
        // let fm_freq2 = dt.exp2();
        let fm_freq2 = d*2.0;
        // c and d can be fm freq multiplier and amplitude
        // what about f cuz. harmonics? yea dont set it to begin with
        // f be amplitude and make it maybe exp shit too

        // sort out this shit

        let period_samples = (period * 44100.0) as u64;
        let n = self.n % period_samples;
        let t = n as f32 / 44100.0;

        let wn = 2.0 * PI / 44100.0;
        let mut f_curr = freq;
        self.fm_phase += wn * fm_freq1;
        f_curr += self.fm_phase.sin() * fm_freq2 * freq;


        self.phase += wn * f_curr;
        if self.phase > 2.0*PI {
            self.phase -= 2.0*PI;
        }
        if self.fm_phase > 2.0*PI {
            self.fm_phase -= 2.0*PI;
        }
        self.n += 1;

        // todo obviously window
        if t/period < duty_cycle {
            (w + out * c) * 0.1 * f

        } else {
            0.0
        }


    }
}

// more eg panning sin function etc
// fft view always good