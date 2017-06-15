extern crate image;
extern crate image_base64;
extern crate cgi;


use std::fs::File;
use std::path::Path;
use std::env;
use std::io;//::{self, Write};
use image::GenericImage;

//should be a trait.  I am not sure how to impliment one for only Vec <Vec <bool>> and not Vec <T>
fn boundless_insert(y: i64, x: i64, value: bool, img: &mut Vec<Vec <bool>>) {
	let mut tx = x;
	let mut ty = y;
	for i in 0..img.len(){
		while tx < 0 {
			img[i].insert(0, false);
			tx += 1;
		}
		while img[i].len() <= tx as usize {
			img[i].push(false);
		}
	}
	let mut row:Vec <bool> = Vec::new();
	for i in 0..img[0].len() {
		row.push(false);
	}
	while ty < 0 {
		img.insert(0, row.clone());
		ty += 1;
	}
	while img.len() <= ty as usize {
		img.push(row.clone())
	}
	img[ty as usize][tx as usize] = value;
}

struct ImgBlob {
	blob_type: u8, //0: object, 1: line
	top_right: [usize; 2],
	middle: [usize; 2],
	text: String,
	bitmap: Vec <Vec <bool>>
}

impl ImgBlob {
	fn from_top_right(x: usize, y: usize, claim: &mut Vec <Vec <bool>>, img: &Vec <Vec <bool>>) -> Option<ImgBlob>{
		let mut bitmap: Vec <Vec <bool>> = Vec::new();
		let mut queue: Vec <[usize; 2]> = Vec::new();
		claim[y][x] = true;
		queue.push([y+1, x]);
		queue.push([y, x+1]);
		bitmap.push(vec![true]);
		while queue.len() > 0 {
			let tempx = queue[0][1];
			let tempy = queue[0][0];
			if (tempy < img.len()) && (tempx < img[0].len()) && (tempy > 0) && (tempx > 0) {
				if img[tempy][tempx] && !(claim[tempy][tempx]){
					boundless_insert(((tempy as i64)-(y as i64)), ((tempx as i64)-(x as i64)), true, &mut bitmap);
					claim[tempy][tempx] = true;
					if queue[0][1] > y {
						queue.push([tempy-1, tempx]);
					}
					queue.push([tempy, tempx-1]);
					queue.push([tempy+1, tempx]);
					queue.push([tempy, tempx+1]);
				}
			}
			queue.remove(0);
		}
		if bitmap[0].len() + bitmap.len() > 6 {
			println!("Found: {} x {}", bitmap[0].len(), bitmap.len());
			Some(ImgBlob {
				blob_type: 0,
				top_right: [x, y],
				middle: [x+(bitmap[0].len()/2), y+(bitmap.len()/2)],
				text: String::from(""),
				bitmap: bitmap
			})
		} else {
			None
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
	let mut thresh: Vec <Vec <bool>> = Vec::new();
	let mut rblobs: Vec <ImgBlob> = Vec::new();
	let mut gblobs: Vec <ImgBlob> = Vec::new();
	let mut bblobs: Vec <ImgBlob> = Vec::new();
	let img = image::open(&Path::new("target.jpg")).unwrap();
	let mut row:Vec <bool> = Vec::new();
	for x in 0..img.width() {
		row.push(false);
	}
	for y in 0..img.height() {
		claimed.push(row.clone());
		thresh.push(row.clone());
	}
	let rgbimg = img.to_rgb();
	for (x, y, pixel) in rgbimg.enumerate_pixels() {
		thresh[y as usize][x as usize] = if (pixel[0] > 210) && (pixel[1] <= 210) && (pixel[2] <= 210) {true} else {false};
	}
	for y in 0..thresh.len() {
		for x in 0..thresh[0].len() {
			if thresh[y][x] && (thresh[y+1][x] || thresh[y][x+1]) {
				match ImgBlob::from_top_right(x, y, &mut claimed, &thresh){
					Some(o) => rblobs.push(o),
					None => {},
				}
			}
		}
	}
	println!("Finished R");
	for (x, y, pixel) in rgbimg.enumerate_pixels() {
		thresh[y as usize][x as usize] = if (pixel[1] > 210) && (pixel[0] <= 210) && (pixel[2] <= 210) {true} else {false};
	}
	for y in 0..thresh.len() {
		for x in 0..thresh[0].len() {
			if thresh[y][x] && (thresh[y+1][x] || thresh[y][x+1]) {
				match ImgBlob::from_top_right(x, y, &mut claimed, &thresh){
					Some(o) => gblobs.push(o),
					None => {},
				}
			}
		}
	}
	println!("Finished G");
	for (x, y, pixel) in rgbimg.enumerate_pixels() {
		thresh[y as usize][x as usize] = if (pixel[2] > 210) && (pixel[1] <= 210) && (pixel[0] <= 210) {true} else {false};
	}
	for y in 0..thresh.len() {
		for x in 0..thresh[0].len() {
			if thresh[y][x]  && (thresh[y+1][x] || thresh[y][x+1]) {
				match ImgBlob::from_top_right(x, y, &mut claimed, &thresh){
					Some(o) => bblobs.push(o),
					None => {},
				}
			}
		}
	}
	println!("Finished B");
}
