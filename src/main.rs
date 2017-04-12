#[macro_use] extern crate log;
extern crate loggerv;

extern crate clap;
extern crate portmidi;
extern crate rosc;

use std::process;
use std::error::Error;
use std::io::Write;
use std::str::FromStr;
use std::collections::HashSet;
use std::time::Duration;
use std::thread;
use std::net::UdpSocket;
use std::sync::mpsc;

use clap::{Arg, App, SubCommand, ArgMatches};
use portmidi::{PortMidi, InputPort, DeviceInfo, MidiEvent};
use rosc::{OscPacket, OscMessage, encoder};

use message::Message;

mod message;

const BUF_LEN: usize = 1024;

fn main() {
    if let Err(e) = run() {
        let mut stderr = std::io::stderr();

        writeln!(
            &mut stderr,
            "Application error: {}",
            e
            ).expect("Could not write to stderr");

        process::exit(1);
    }
}

fn run() -> Result<(), Box<Error>> {
    let matches = App::new("midioscar")
        .version("0.1.0")
        .author("Jan-Erik Rediger <jer@rrbone.net>")
        .about("Send MIDI input as OSC output")
        .help_short("h")
        .subcommand(SubCommand::with_name("serve")
                    .help_short("H")
                    .arg(Arg::with_name("v")
                         .short("v")
                         .multiple(true)
                         .help("Sets the level of verbosity"))
                    .arg(Arg::with_name("inputs")
                         .short("i")
                         .long("input")
                         .value_name("INPUT")
                         .takes_value(true)
                         .multiple(true)
                         .required(true))
                    .arg(Arg::with_name("hosts")
                         .short("h")
                         .long("host")
                         .value_name("HOST")
                         .takes_value(true)
                         .multiple(true)
                         .required(true)))
        .subcommand(SubCommand::with_name("list")
                    .help_short("h"))
        .get_matches();

    if matches.subcommand_matches("list").is_some() {
        let pm = PortMidi::new()?;

        let devices = pm.devices()?;
        let in_devices = devices.iter().filter(|dev| dev.is_input()).collect::<Vec<_>>();
        println!("{} devices found (use listed ID as interface ID):", in_devices.len());

        for device in in_devices {
            println!("{}", device);
        }

        return Ok(())
    }

    if let Some(matches) = matches.subcommand_matches("serve") {
        loggerv::init_with_verbosity(matches.occurrences_of("v")).unwrap();

        return serve(matches);
    }

    Err("Unknown subcommand".into())
}

fn serve(matches: &ArgMatches) -> Result<(), Box<Error>> {
    let hosts = matches.values_of("hosts").unwrap().map(String::from).collect::<Vec<String>>();
    let inputs = matches.values_of("inputs").unwrap();

    let pm = PortMidi::new()?;
    let avail_devices = pm.devices()?
        .into_iter()
        .filter(|dev| dev.is_input())
        .collect::<Vec<_>>();

    let inputs = inputs
        .map(|inp| i32::from_str(inp))
        .collect::<Result<Vec<_>, _>>()?;

    let input_devices = inputs.iter().collect::<HashSet<_>>();
    let in_count = input_devices.len();
    let in_devices = avail_devices.into_iter()
        .filter(|dev| input_devices.contains(&dev.id()) )
        .collect::<Vec<_>>();

    let mut input_ports = in_devices.into_iter()
        .map(|dev| pm.input_port(dev, BUF_LEN))
        .collect::<Result<Vec<_>, _>>()?;

    println!("Listening on:");
    println!("  {:?}", inputs);
    println!("Forward OSC messages to:");
    println!("  {:?}", hosts);

    if in_count == 1 {
        simple_poll(input_ports.swap_remove(0), hosts);
    } else {
        threaded_poll(input_ports, hosts);
    }

    Ok(())
}

fn handle_event(hosts: &[String], device: &DeviceInfo, event: MidiEvent) {
    let msg = Message::from(event.message);

    info!("On {:?} (#{}): {:?}", device.name(), device.id(), msg);
    publish(hosts, msg.to_osc(&device.name()));
}

fn simple_poll(in_port: InputPort, hosts: Vec<String>) {
    let timeout = Duration::from_millis(10);

    while let Ok(_) = in_port.poll() {
        if let Ok(Some(events)) = in_port.read_n(BUF_LEN) {
            for ev in events {
                handle_event(&hosts, &in_port.device(), ev)
            }
        }
        // there is no blocking receive method in PortMidi, therefore
        // we have to sleep some time to prevent a busy-wait loop
        thread::sleep(timeout);
    }
}

fn threaded_poll(in_ports: Vec<InputPort>, hosts: Vec<String>) {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let timeout = Duration::from_millis(10);

        loop {
            for port in &in_ports {
                if let Ok(Some(events)) = port.read_n(BUF_LEN) {
                    tx.send((port.device(), events)).unwrap();
                }
            }
            thread::sleep(timeout);
        }
    });

    loop {
        let (device, events) = rx.recv().unwrap();
        for event in events {
            handle_event(&hosts, &device, event);
        }
    }
}

fn publish(hosts: &[String], msg: OscMessage) {
    let msg_buf = encoder::encode(&OscPacket::Message(msg)).expect("Can't encode message");
    let sock = UdpSocket::bind("0.0.0.0:0").expect("Can't create UDP socket");

    info!("Publishing message to {} hosts.", hosts.len());

    for host in hosts {
        if let Err(e) = sock.send_to(&msg_buf, host) {
            error!("Failed to write message to {}", host);
            error!("  Error: {}", e);
        }
    }
}
