use glicol::Engine;

use crate::SAMPLE_RATE;

pub struct Glicol<const N: usize> {
    pub glicol: Engine<N>,
    pub playing: bool,
    code: String,
}

impl<const N: usize> Glicol<N> {
    pub fn new() -> Self {
        let mut glicol = Engine::<N>::new();

        glicol.set_sr(SAMPLE_RATE.try_into().unwrap());

        Glicol {
            glicol,
            playing: false,
            code: String::new(),
        }
    }

    pub fn engine(&mut self) -> &mut Engine<N> {
        &mut self.glicol
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn code(&self) -> &String {
        &self.code
    }

    pub fn set_code(&mut self, code: String) -> Result<(), glicol::EngineError> {
        self.glicol.update_with_code(&code);

        if let Err(e) = self.glicol.parse() {
            self.glicol.update_with_code(&self.code);
            return Err(e);
        }

        if let Err(e) = self.glicol.make_graph() {
            self.glicol.update_with_code(&self.code);
            return Err(e);
        }

        self.code = code;

        self.play();
        Ok(())
    }
}
