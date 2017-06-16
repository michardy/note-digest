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
		let mut ttx = x;
		while ttx < 0 {
			img[i].insert(0, false);
			ttx += 1;
		}
		assert!(((x < 0) && (ttx == 0)) || (x == ttx), "Expected ttx = zero or ttx = x.  Found {}", ttx);
		while img[i].len() <= ttx as usize {
			img[i].push(false);
		}
		tx = ttx
	}
	let mut row:Vec <bool> = Vec::new();
	for i in 0..img[0].len() {
		row.push(false);
	}
	while ty < 0 {
		img.insert(0, row.clone());
		ty += 1;
	}
	assert!(((y < 0) && (ty == 0)) || (y == ty), "Expected ty = zero or ty = y.  Found {}", ty);
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
		//let mut imgbuf = image::ImageBuffer::new(2550u32, 3300u32);
		let mut right = x;
		let mut top = y; //pretty sure that this is not supposed to change and can be eliminated
		let mut bitmap: Vec <Vec <bool>> = Vec::new();
		let mut queue: Vec <[usize; 2]> = Vec::new();
		claim[y][x] = true;
		//imgbuf.put_pixel(x as u32, y as u32, image::Rgb([0, 255, 0]));
		queue.push([y+1, x]);
		queue.push([y, x+1]);
		bitmap.push(vec![true]);
		while queue.len() > 0 {
			let tempx = queue[0][1];
			let tempy = queue[0][0];
			if (tempy < img.len()) && (tempx < img[0].len()) && (tempy > 0) && (tempx > 0) {
				if img[tempy][tempx] && !(claim[tempy][tempx]){
					boundless_insert(((tempy as i64)-(top as i64)), ((tempx as i64)-(right as i64)), true, &mut bitmap);
					if tempx < right {
						right = tempx;
					}
					if tempy < top {
						top = tempy;
					}
					//imgbuf.put_pixel(tempx as u32, tempy as u32, image::Rgb([0, 255, 0]));
					claim[tempy][tempx] = true;
					if queue[0][1] > top {
						queue.push([tempy-1, tempx]);
					}
					queue.push([tempy, tempx-1]);
					queue.push([tempy+1, tempx]);
					queue.push([tempy, tempx+1]);
				}/* else if !(claim[tempy][tempx]) {
					imgbuf.put_pixel(tempx as u32, tempy as u32, image::Rgb([255, 0, 0]));
				}*/
			}/* else {
				imgbuf.put_pixel(tempx as u32, tempy as u32, image::Rgb([255, 0, 0]));
			}*/
			queue.remove(0);
		}
		if (bitmap[0].len() + bitmap.len() > 6) && (bitmap.len() > 3) && (bitmap[0].len() > 3) {
			/*let mut imgbuf = image::ImageBuffer::new((bitmap[0].len()) as u32, (bitmap.len()) as u32);
			for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
				*pixel = image::Luma([if bitmap[y as usize][x as usize] {255} else {0}]);
			}
			let xdim = &*bitmap[0].len().to_string();
			let ydim = &*bitmap.len().to_string();
			let ref mut fout = File::create(&Path::new(&(String::from("outg")+xdim+"x"+ydim+".png"))).unwrap();
			let _ = image::ImageRgb8(imgbuf).save(fout, image::PNG);*/
			println!("Found: {} x {}", bitmap[0].len(), bitmap.len());
			Some(ImgBlob {
				blob_type: 0,
				top_right: [right, top],
				middle: [right+(bitmap[0].len()/2), top+(bitmap.len()/2)],
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
	//let mut rblobs: Vec <ImgBlob> = Vec::new();
	let mut gblobs: Vec <ImgBlob> = Vec::new();
	//let mut bblobs: Vec <ImgBlob> = Vec::new();
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
	//let mut imgbuf = image::ImageBuffer::new(img.width(), img.height());
	/*for (x, y, pixel) in rgbimg.enumerate_pixels() {
		thresh[y as usize][x as usize] = if (pixel[0] > 140) && (pixel[1] <= 170) && (pixel[2] <= 170) {true} else {false};
		imgbuf.put_pixel(x, y, image::Luma([if (pixel[0] > 140) & (pixel[1] <= 170) & (pixel[2] <= 170) {255} else {0}]));
	}
	let ref mut fout = File::create(&Path::new("outr.png")).unwrap();
	let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);
	for y in 0..thresh.len() {
		for x in 0..thresh[0].len() {
			if thresh[y][x] {
				match ImgBlob::from_top_right(x, y, &mut claimed, &thresh){
					Some(o) => rblobs.push(o),
					None => {},
				}
			}
		}
	}
	println!("Finished R");*/
	for (x, y, pixel) in rgbimg.enumerate_pixels() {
		thresh[y as usize][x as usize] = if (pixel[1] > 140) && (pixel[0] <= 170) && (pixel[2] <= 170) {true} else {false};
	}
	for y in 0..thresh.len() {
		for x in 0..thresh[0].len() {
			//imgbuf.put_pixel(x as u32, y as u32, image::Luma([if thresh[y][x] {255} else {0}]));
			if thresh[y][x] {
				match ImgBlob::from_top_right(x, y, &mut claimed, &thresh){
					Some(o) => gblobs.push(o),
					None => {},
				}
			}
		}
	}
	/*let ref mut fout = File::create(&Path::new("outg.png")).unwrap();
	let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);*/
	let mut imgbuf = image::ImageBuffer::new(img.width(), img.height());
	for i in 0..gblobs.len() {
		let xoff = gblobs[i].top_right[0];
		let yoff = gblobs[i].top_right[1];
		for y in 0..gblobs[i].bitmap.len() {
			for x in 0..gblobs[i].bitmap[y].len() {
				imgbuf.put_pixel((x+xoff) as u32, (y+yoff) as u32, image::Luma([if (gblobs[i].bitmap[y][x]) {255} else {0}]));
			}
		}
	}
	let ref mut fout = File::create(&Path::new("reconstructg.png")).unwrap();
	let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);
	println!("Finished G");
	/*for (x, y, pixel) in rgbimg.enumerate_pixels() {
		thresh[y as usize][x as usize] = if (pixel[2] > 140) && (pixel[1] <= 170) && (pixel[0] <= 170) {true} else {false};
	}
	for y in 0..thresh.len() {
		for x in 0..thresh[0].len() {
			imgbuf.put_pixel(x as u32, y as u32, image::Luma([if thresh[y][x] {255} else {0}]));
			if thresh[y][x] {
				match ImgBlob::from_top_right(x, y, &mut claimed, &thresh){
					Some(o) => bblobs.push(o),
					None => {},
				}
			}
		}
	}
	let ref mut fout = File::create(&Path::new("outb.png")).unwrap();
	let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);
	println!("Finished B");*/
	
}
