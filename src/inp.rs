use std::{
    io::Read,
    sync::mpsc::{self, TryRecvError},
};

#[cfg(feature = "fifo")]
use std::{env::temp_dir, fs::File};

pub struct Inputs(Vec<AsyncRead>);

impl Inputs {
    pub fn init() -> Self {
        let stdin = std::io::stdin();
        let stdin = AsyncRead::new(stdin);

        let mut out = Vec::new();
        out.push(stdin);

        #[cfg(feature = "fifo")]
        {
            let pipe_file = {
                let mut path = temp_dir();
                path.push("glicol.fifo");
                path
            };

            if !pipe_file.exists() {
                nix::unistd::mkfifo(&pipe_file, nix::sys::stat::Mode::S_IRWXU).unwrap();
            };

            out.push(AsyncRead::from_path(pipe_file));
        }

        Self(out)
    }

    pub fn strings(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        for inp in self.0.iter_mut() {
            let mut inp_str = String::new();
            if inp.read_to_string(&mut inp_str).unwrap() {
                out.push(inp_str)
            };
        }
        out
    }
}

pub struct AsyncRead {
    pub active: bool,
    inner: mpsc::Receiver<String>,
}

impl AsyncRead {
    #[cfg(feature = "fifo")]
    pub fn from_path(path: std::path::PathBuf) -> Self {
        let (send, recv) = mpsc::channel();

        std::thread::spawn(move || {
            let mut inner = File::open(path).unwrap();
            loop {
                let mut string = String::new();

                if inner.read_to_string(&mut string).is_err() {
                    eprintln!("Read Error");
                };

                if send.send(string).is_err() {
                    eprintln!("send error");
                    return;
                }
            }
        });

        Self {
            active: true,
            inner: recv,
        }
    }

    pub fn new(mut inner: impl Read + std::marker::Send + 'static) -> Self {
        let (send, recv) = mpsc::channel();

        std::thread::spawn(move || loop {
            let mut string = String::new();

            if inner.read_to_string(&mut string).is_err() {
                eprintln!("Read Error");
            };

            if send.send(string).is_err() {
                eprintln!("send error");
                return;
            }
        });

        Self {
            active: true,
            inner: recv,
        }
    }

    pub fn read_to_string(&mut self, string: &mut String) -> Result<bool, TryRecvError> {
        if !self.active {
            return Ok(false);
        }

        let out = match self.inner.try_recv() {
            Ok(s) if s.is_empty() => false,
            Ok(s) => {
                *string = s;
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => {
                eprintln!("Disconnected reader");
                self.active = false;
                false
            }
        };

        Ok(out)
    }
}
