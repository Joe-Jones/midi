#![feature(try_from)]

extern crate jack;
extern crate time_calc;

use std::convert::TryFrom;
use std::f64;
use std::u32;
use std::io;
use jack::prelude::{AsyncClient, Client, JackControl, MidiOutPort,
                    MidiOutSpec, ClosureProcessHandler, ProcessScope, RawMidi, client_options, JackFrames};
use time_calc::{
    SampleHz, Bpm, Beats,
};

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
    println!("Samples per beat: {}", samples_per_beat);
    
    // make a midi port
    let mut maker = client.register_port("midi_out", MidiOutSpec::default()).unwrap();

    let mut next_beat : JackFrames = 0;
    
    let cback = move |_: &Client, ps: &ProcessScope| -> JackControl {
        
        let mut put_p = MidiOutPort::new(&mut maker, ps);

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
