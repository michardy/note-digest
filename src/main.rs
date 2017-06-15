extern crate image;
extern crate image_base64;
extern crate cgi;


use std::fs::File;
use std::path::Path;
use std::env;
use std::io;//::{self, Write};
use image::GenericImage;

struct ImgBlob {
	blob_type: u8, //0: object, 1: line
	top_right: [u32; 2],
	middle: [u32; 2],
	text: String,
	bitmap: Vec <Vec <bool>>
}

impl ImgBlob {
	fn from_top_right(x:u32, y:u32, claim: &mut Vec <Vec <bool>>) -> ImgBlob{
		let mut bitmap: Vec <Vec <bool>> = Vec::new();
		let mut queue: Vec <[u32, 2]> = Vec::new();
		claim[x][y] = true;
		ImgBlob {
			blob_type: 0,
			top_right: [x, y],
			middle: [x+5, y+5],
			text: String::from(""),
			bitmap: Vec::new()
		}
	}
}

struct Section {
	Heading: ImgBlob,
	blobs: Vec <ImgBlob>
}

struct Chapter {
	heading: ImgBlob,
	blobs: Vec <ImgBlob>,
	Sections: Vec <Section>
}

fn main() {
	let mut claimed: Vec <Vec <bool>> = Vec::new();
	print!("first unwrap: ");
	let img = image::open(&Path::new("target.jpg")).unwrap();
	println!("completed");
	let mut imgbuf = image::ImageBuffer::new(img.width(), img.height());
	for y in 0..img.height() {
		let mut row:Vec <bool> = Vec::new();
		for x in 0..img.width() {
			row.push(false);
		}
		claimed.push(row);
	}
	let rgbimg = img.to_rgb();
	for (x, y, pixel) in rgbimg.enumerate_pixels() {
		imgbuf.put_pixel(x, y, image::Luma([if (pixel[0] > 210) & (pixel[1] <= 210) & (pixel[2] <= 210) {255} else {0}]));
	}
	print!("second unwrap: ");
	let ref mut fout = File::create(&Path::new("outr.png")).unwrap();
	println!("completed");
	let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);
}
