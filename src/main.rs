extern crate dbus;
extern crate image;

use std::fs::File;

use dbus::{Connection, BusType, Message, MessageItem};
use dbus::arg::Array;

use image::png::PNGEncoder;
use image::{ColorType};

// Bit depth of image.
const BIT_DEPTH: u8 = 8;
// Time to wait for D-Bus to respond
const WAIT_TIME: i32 = 2000;

fn main() {
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
    let out = File::create("out.png")
        .expect("Could not write out to file");
    let encoder = PNGEncoder::new(out);
    encoder.encode(arr.as_slice(), res.0, res.1, ColorType::RGBA(BIT_DEPTH))
        .expect("Could not encode image to PNG")
}

fn resolution(con: &Connection) -> (u32, u32) {
    let screens_msg = Message::new_method_call("org.way-cooler",
                                               "/org/way_cooler/Screen",
                                               "org.way_cooler.Screen",
                                               "List")
    .expect("Could not construct message -- is Way Cooler running?");
    let screen_r = con.send_with_reply_and_block(screens_msg, WAIT_TIME)
        .expect("Could not talk to Way Cooler -- is Way Cooler running?");
    let screen_r = &screen_r.get_items()[0];
    let output_id = match screen_r {
        &MessageItem::Array(ref items, _) => {
            match items[0] {
                MessageItem::Str(ref string) => string.clone(),
                _ => panic!("Array didn't contain output id")
            }
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
