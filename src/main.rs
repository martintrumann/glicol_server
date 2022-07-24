use std::{collections::VecDeque, sync::Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, Stream,
};

const SAMPLE_RATE: u32 = 48_000;
const GLICOL_BUFFER_SIZE: usize = 128;

mod glicol;
use crate::glicol::Glicol;

mod inp;
use inp::Inputs;

lazy_static::lazy_static! {
    static ref GLICOL: Mutex<Glicol<GLICOL_BUFFER_SIZE>> = Mutex::new(Glicol::<GLICOL_BUFFER_SIZE>::new());
}

struct App {
    stream: Stream,
    running: bool,
}

impl App {
    fn init() -> Self {
        let host = cpal::default_host();

        let device = host.default_output_device().expect("no output available");

        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");

        let config = supported_configs_range
            .find(|c| c.channels() == 2 && c.sample_format() == SampleFormat::F32)
            .expect("no supported config?!")
            .with_sample_rate(cpal::SampleRate(SAMPLE_RATE));

        let mut out = VecDeque::with_capacity(4100);
        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut glicol = GLICOL.lock().unwrap();

                    if !glicol.playing {
                        return;
                    }

                    let glicol = glicol.engine();

                    while out.len() < data.len() {
                        let (blk, _) = glicol.next_block(Vec::new());

                        for i in 0..blk[0].len() {
                            out.push_back(blk[0][i]);
                            out.push_back(blk[1][i]);
                        }
                    }

                    for i in 0..data.len() {
                        data[i] = out.pop_front().unwrap()
                    }
                },
                |err| eprintln!("an error occurred on the output audio stream: {}", err),
            )
            .unwrap();

        let _ = stream.pause();

        Self {
            stream,
            running: true,
        }
    }

    fn parse_inp(&mut self, s: String) {
        let mut glicol = GLICOL.lock().unwrap();

        let (command, arg) = if let Some((cmd, arg)) = s.trim().split_once(' ') {
            (cmd, Some(arg))
        } else {
            (s.trim(), None)
        };

        match command {
            "quit" | "q" => self.running = false,
            "play" | "p" => {
                let _ = self.stream.play();
                glicol.play();
            }
            "stop" | "s" | "pause" => {
                let _ = self.stream.pause();
                glicol.pause();
            }
            "set_bpm" if arg.is_some() => {
                if let Some(arg) = arg.unwrap().parse::<u16>().ok() {
                    let engine = glicol.engine();
                    engine.set_bpm(arg.into())
                } else {
                    eprintln!("Not a number")
                }
            }
            "set_bpm" if arg.is_none() => eprintln!("no arg"),
            "ping" => eprintln!("pong"),
            "" => (),
            _ => match glicol.set_code(s) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("GLICOL ERROR: {:?}", e)
                }
            },
        }
    }
}

fn main() {
    let mut app = App::init();
    let mut inp = Inputs::init();
    loop {
        let commands = inp.strings();

        for inp_str in commands {
            app.parse_inp(inp_str);
        }

        if !app.running {
            break;
        };
    }
}
