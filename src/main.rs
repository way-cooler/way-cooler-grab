extern crate clap;
extern crate dbus;
extern crate image;

use std::fs::File;

use clap::{Arg, App};

use dbus::{Connection, BusType, Message, MessageItem};
use dbus::arg::Array;

use image::png::PNGEncoder;
use image::{ColorType, ImageFormat, load_from_memory};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
// Bit depth of image.
const BIT_DEPTH: u8 = 8;
// Time to wait for D-Bus to respond
const WAIT_TIME: i32 = 2000;

fn main() {
    let matches = App::new("wc-grab")
        .version(VERSION)
        .author("Timidger <APragmaticPlace@gmail.com>")
        .arg(Arg::with_name("Output File")
             .short("o")
             .long("output")
             .value_name("Output File")
             .help("Sets an custom output file to write to. Defauts to \"screenshot.png\"")
             .takes_value(true))
        .arg(Arg::with_name("Version")
            .short("v")
            .long("version")
            .help("Displays the version number of the program"))
        .get_matches();
    if matches.is_present("Version") {
        println!("{}", VERSION);
        return
    }
    let output_name = matches.value_of("Output File").unwrap_or("screenshot.png");
    let mut out = File::create(output_name)
        .expect("Could not create output file");

    let con = Connection::get_private(BusType::Session)
        .expect("Could not connect to D-Bus");
    let res = resolution(&con);
    let msg = Message::new_method_call("org.way-cooler",
                                       "/org/way_cooler/Screen",
                                       "org.way_cooler.Screen",
                                       "Scrape")
        .expect("Could not construct message -- is Way Cooler running?");
    let reply = con.send_with_reply_and_block(msg, WAIT_TIME)
        .expect("Could not talk to Way Cooler -- is Way Cooler running?");
    let arr: Array<u8, _> = reply.get1()
        .expect("Way Cooler returned an unexpected value");
    let mut arr = arr.collect::<Vec<u8>>();
    convert_to_png(&mut arr);
    let mut png_buf = Vec::with_capacity(4 * (res.0 * res.1) as usize);
    {
        let encoder = PNGEncoder::new(&mut png_buf);
        encoder.encode(arr.as_slice(), res.0, res.1, ColorType::RGBA(BIT_DEPTH))
            .expect("Could not encode image to PNG");
    }
    let mut image = load_from_memory(png_buf.as_slice())
        .expect("Could not read encoded image");
    image = image.flipv();
    image.save(&mut out, ImageFormat::PNG)
        .expect("Could not save image to file");
}

fn resolution(con: &Connection) -> (u32, u32) {
    let screens_msg = Message::new_method_call("org.way-cooler",
                                               "/org/way_cooler/Screen",
                                               "org.way_cooler.Screen",
                                               "ActiveScreen")
    .expect("Could not construct message -- is Way Cooler running?");
    let screen_r = con.send_with_reply_and_block(screens_msg, WAIT_TIME)
        .expect("Could not talk to Way Cooler -- is Way Cooler running?");
    let screen_r = &screen_r.get_items()[0];
    let output_id = match screen_r {
        &MessageItem::Str(ref string) => {
            string.clone()
        }
        _ => panic!("Wrong type from Screen")
    };
    let res_msg = Message::new_method_call("org.way-cooler",
                                           "/org/way_cooler/Screen",
                                           "org.way_cooler.Screen",
                                           "Resolution")
        .expect("Could not construct message -- is Way Cooler running?")
        .append(MessageItem::Str(output_id));
    let reply: MessageItem = con.send_with_reply_and_block(res_msg, WAIT_TIME)
        .expect("Could not talk to Way Cooler -- is Way Cooler running?")
        .get1()
        .expect("Way Cooler returned an unexpected value");
    match reply {
        MessageItem::Struct(items) => {
            let (width, height) = (&items[0], &items[1]);
            println!("{:?}, {:?}", width, height);
            (width.inner::<u32>()
             .expect("Way Cooler returned an unexpected value"),
             height.inner::<u32>().expect("Way Cooler returned an unexpected value"))
        },
        _ => panic!("Colud not get resolution of screen")
    }
}

fn convert_to_png(buffer: &mut Vec<u8>) {
    let mut length = buffer.len();
    length -= length % 4;
    let mut i = 0;
    while i < length {
        // a b c d -> d a b c
        buffer[i + 2] ^= buffer[i + 3];
        buffer[i + 3] = buffer[i + 2] ^ buffer[i + 3];
        buffer[i + 2] ^= buffer[i + 3];
        buffer[i] ^= buffer[i + 2];
        buffer[i + 2] = buffer[i] ^ buffer[i + 2];
        buffer[i] ^= buffer[i + 2];
        buffer[i + 2] ^= buffer[i + 1];
        buffer[i + 1] = buffer[i + 1] ^ buffer[i + 2];
        buffer[i + 2] ^= buffer[i + 1];
        i += 4;
    }
}
