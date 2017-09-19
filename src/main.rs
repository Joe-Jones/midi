#![feature(try_from)]

extern crate jack;
extern crate time_calc;

use std::convert::TryFrom;
use std::f64;
use std::u32;
use std::usize;
use std::io;
use jack::prelude::{AsyncClient, Client, JackControl, MidiOutPort,
                    MidiOutSpec, ClosureProcessHandler, ProcessScope, RawMidi, client_options, JackFrames};
use time_calc::{
    SampleHz, Bpm, Beats, Bars, TimeSig
};

struct PatternPlayer {
    samples_per_step: JackFrames,
    position: JackFrames,
    pattern: Vec<u8>,
}

impl PatternPlayer {
    fn new(bpm: &Bpm, sample_frequency : SampleHz, time_signature : &TimeSig, steps_per_bar: usize, pattern: Vec<u8>) -> PatternPlayer {

        return PatternPlayer{
            samples_per_step: 0,
            position: 0,
            pattern: pattern
        };
    }
}

struct Pattern {
    steps_per_bar: u32,
    pattern: Vec<u8>,
    note: u8,
}

impl Pattern {

    fn next_event(&self, position: f64) -> Result<Option<(f64, u8)>, String> {
        let relative_to_pattern = position * f64::from(self.steps_per_bar);
        let index = relative_to_pattern.ceil() as usize;
        let mut n = index;
        while n < self.pattern.len() {
            if self.pattern[n] > 0 {
                return Ok(Some((n as f64 / self.steps_per_bar as f64, self.pattern[n])));
            }
            n += 1;
        }
        
        Ok(None)
    }

}

fn main() {
    // open client
    let (client, _status) = Client::new("kick-drum", client_options::NO_START_SERVER)
        .unwrap();

    // the try_from used here needs the nightly build.
    let sample_frequency : SampleHz = f64::from( u32::try_from(client.sample_rate())
                                            .map_err( |err| format!("Sample rate could not be converted to a u32! The error was {}", err.to_string())).unwrap());
    
    let bpm: Bpm = 120.0;

    let samples_per_beat : JackFrames = u32::try_from(Beats(1).samples(bpm, sample_frequency))
        .map_err( |err| format!("Samples per beat could not be fitted into a u32! the error was {}", err.to_string())).unwrap();
    // println!("Samples per beat: {}", samples_per_beat);
    
    // make a midi port
    let mut maker = client.register_port("midi_out", MidiOutSpec::default()).unwrap();

    let mut next_beat : JackFrames = 0;

    let time_signature = TimeSig{ top: 4, bottom: 4 };
    let kick_drum_pattern : Vec<u8> = vec![127, 127, 127, 127];
    let snare_drum_pattern     : Vec<u8> = vec![0  , 127, 0  , 127];

    let kick_drum = Pattern{ steps_per_bar: 4, pattern: kick_drum_pattern, 36 };
    let snare_drum = Pattern{ steps_per_bar: 4, pattern: snare_drum_pattern, 37 };
        
    let frames_per_bar : JackFrames = u32::try_from(Bars(1).samples(bpm, time_signature, sample_frequency))
        .map_err( |err| format!("Samples per bar could not be fitted into a u32! the error was {}", err.to_string())).unwrap();

    //let mut next_bar : JackFrames = 0;
    let mut frames_into_bar = 0;
    let mut cursor : Jackframes = 0;
    
    let cback = move |_: &Client, ps: &ProcessScope| -> JackControl {
        
        let mut put_p = MidiOutPort::new(&mut maker, ps);

        while cursor < ps.nframes() {
            let (time, velocity) = kick_drum.next_event(frames_into_bar as f64 / frames_per_bar as f64);
            let 
        }

        cursor -= ps.n_frames();
        
                
        while next_beat < ps.n_frames() {
            put_p.write(&RawMidi {
                time: next_beat,
                bytes: &[0b10010000 /* Note On, channel 1 */,
                         36 /* Key number */, 0b01111111 /* Velocity */],
            })
                .unwrap();
            next_beat += samples_per_beat;
        }
        next_beat -= ps.n_frames();
                
        JackControl::Continue
    };

    // activate
    let process = ClosureProcessHandler::new(cback);
    let active_client = AsyncClient::new(client, (), process).unwrap();

    // wait
    println!("Press any key to quit");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    // optional deactivation
    active_client.deactivate().unwrap();
}
