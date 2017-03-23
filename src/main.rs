extern crate dbus;
extern crate image;

use std::fs::File;

use dbus::{Connection, BusType, Message, MessageItem};
use dbus::arg::Array;

use image::png::PNGEncoder;
use image::{ColorType, load_from_memory};

// Bit depth of image.
const BIT_DEPTH: u8 = 8;
// Time to wait for D-Bus to respond
const WAIT_TIME: i32 = 2000;

fn main() {
    let con = Connection::get_private(BusType::Session).unwrap();
    let res = resolution(&con);
    let msg = Message::new_method_call("org.way-cooler",
                                     "/org/way_cooler/Screen",
                                     "org.way_cooler.Screen",
                                     "Scrape").unwrap();
    let reply = con.send_with_reply_and_block(msg, WAIT_TIME).unwrap();
    let arr: Array<u8, _> = reply.get1().unwrap();
    let arr = arr.collect::<Vec<u8>>();
    let out = File::create("out.png").unwrap();
    let encoder = PNGEncoder::new(out);
    encoder.encode(arr.as_slice(), res.0, res.1, ColorType::RGBA(BIT_DEPTH))
        .unwrap()
}

fn resolution(con: &Connection) -> (u32, u32) {
    let screens_msg = Message::new_method_call("org.way-cooler",
                                               "/org/way_cooler/Screen",
                                               "org.way_cooler.Screen",
                                               "List").unwrap();
    let screen_r = con.send_with_reply_and_block(screens_msg, WAIT_TIME).unwrap();
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
                                           "Resolution").unwrap()
        .append(MessageItem::Str(output_id));
    let reply: MessageItem = con.send_with_reply_and_block(res_msg, WAIT_TIME).unwrap()
        .get1().unwrap();
    match reply {
        MessageItem::Struct(items) => {
            let (width, height) = (&items[0], &items[1]);
            println!("{:?}, {:?}", width, height);
            (width.inner::<u32>().unwrap(), height.inner::<u32>().unwrap())
        },
        _ => panic!("Colud not get resolution of screen")
    }
}

// TODO
// I think I need to reorder the pixels before I continue...
// Because that's why it looks all blue and shit.
// flipping it _should_ be easy after that....


// ACTUALLY do the conversion, but into some buffer not a file.
// then go through, manually switch the bits (now in the nice u32 format I expect).
// and then write to a file.
/*fn convert(pixels: &mut [u8]) {
    let length = pixels.len();
    let mut i = 0;
    while i < length {
        let alpha = pixel[i + 3] as u32;
        pixels[i] / rgba_conversion(pixels[i], alpha);
        pixels[i + 1] / rgba_conversion(pixels[i], alpha);
        pixels[i + 2] / rgba_conversion(pixels[i], alpha);
        let tmp = pixels[i + 2];
        pixels[i + 2] = pixels[0];
        pixels[0] = tmp;
    }
}*/


fn rgba_conversion(num: u8, third_num: u32) -> u8 {
    let big_num = num as u32;
    ((big_num * third_num) / 255) as u8
}
