extern crate image;


use std::fs;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::io;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use image::GenericImage;

const IMPORTED: &'static str = "./.imported";
const OUT_PATH: &'static str = "~/Documemts/Notebook";

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
	bitmap: Vec <Vec <bool>>
}

impl ImgBlob {
	fn from_top_right(x: usize, y: usize, claim: &mut Vec <Vec <bool>>, img: &Vec <Vec <bool>>) -> Option<ImgBlob>{
		let mut right = x;
		let mut top = y; //pretty sure that this is not supposed to change and can be eliminated
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
					boundless_insert(((tempy as i64)-(top as i64)), ((tempx as i64)-(right as i64)), true, &mut bitmap);
					if tempx < right {
						right = tempx;
					}
					if tempy < top {
						top = tempy;
					}
					claim[tempy][tempx] = true;
					if queue[0][1] > top {
						queue.push([tempy-1, tempx]);
					}
					queue.push([tempy, tempx-1]);
					queue.push([tempy+1, tempx]);
					queue.push([tempy, tempx+1]);
				}
			}
			queue.remove(0);
		}
		if (bitmap[0].len() + bitmap.len() > 6) && (bitmap.len() > 3) && (bitmap[0].len() > 3) {
			println!("Found: {} x {}", bitmap[0].len(), bitmap.len());
			Some(ImgBlob {
				blob_type: if (bitmap[0].len() / bitmap.len()) > 10 {1} else {0},
				top_right: [right, top],
				middle: [right+(bitmap[0].len()/2), top+(bitmap.len()/2)],
				bitmap: bitmap
			})
		} else {
			None
		}
	}
}

struct Page {
	blobs: Vec <ImgBlob>,
	dimensions: [u32; 2]
}

struct Chapter {
	heading: ImgBlob,
	blobs: Vec <ImgBlob>,
	subchapters: Vec <Chapter>
}

fn get_images() -> Vec <String> {
	fn get_imported_images() -> Vec <String> {
		println!("test");
		if Path::new(IMPORTED).exists() {
			println!("test");
			let mut list: Vec <String> = Vec::new();
			let f = (File::open(IMPORTED)).unwrap();
			let mut file = BufReader::new(&f);
			for line in file.lines() {
				println!("test");
				let templ = line.unwrap();
				println!("{}", &templ);
				list.push(templ);
			}
			list
		} else {
			Vec::new()
		}
	}
	fn parse_input(uin: String, mpaths: Vec <String>, new: &mut Vec <String>) -> Vec <String> {
		let mut selected: Vec <String> = Vec::new();
		let stringified: Vec <String> = uin.split(' ').map(|x| x.to_string()).collect();
		for sel in stringified {
			if sel == "+" {
				selected.append(new);
			} else if sel.to_string().contains("-") {
				let numbers: Vec <String> = uin.split('-').map(|x| x.to_string()).collect();
				let start = numbers[0].parse::<usize>().unwrap();
				let end = numbers[1].parse::<usize>().unwrap();
				for i in start..end {
					selected.push(mpaths[i].clone());
				}
			} else {
				let i = sel.to_string().parse::<usize>().unwrap();
				selected.push(mpaths[i].clone());
			}
		}
		println!("{:?}", selected);
		selected
	}
	let paths = fs::read_dir("./").unwrap();
	let mut mpaths: Vec <String> = Vec::new();
	let mut new: Vec <String> = Vec::new();
	let mut imported: Vec <String> = get_imported_images();
	for p in paths {
		let path = p.unwrap().path();
		if !(path.extension() == None) {
			//The next line needs to be cleaned up.  It is written like this to appease the borrow checker
			if path.extension().unwrap() == "png" || path.extension().unwrap() == "jpg" || path.extension().unwrap() == "bpm" || path.extension().unwrap() == "gif" {
				mpaths.push(path.into_os_string().into_string().unwrap());//ugly hack but as_path().to_string() does not work
			}
		}
	}
	mpaths.sort();
	let mut fiter:usize = 0;
	for p in &mpaths {
		if !imported.contains(p) {
			print!("+");
			new.push(p.clone());//cannot pass borrowed var w/o cloning
		}
		println!("	{}:	{}", fiter, p);
		fiter += 1;
	}
	println!("Enter an number to select an image to import.  ");
	println!("Enter 5-6 to import images 5 through 6.  ");
	println!("Enter + to import the images you have not imported.  (These images are indicated in the list by + signs)");
	println!("Use spaces to seperate multiple selections.  ");
	println!("select: ");
	let mut uin = String::new();
	io::stdin().read_line(&mut uin).ok().expect("Error reading line");
	uin.pop();
	parse_input(uin, mpaths, &mut new)
}

fn main() {
	//iterate through images pulling out blobs
	//iterate through pages parsing blobs and creating chapters
	let selected = get_images();
	//need to run some kind of sort
	/*let mut claimed: Vec <Vec <bool>> = Vec::new();
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
		thresh[y as usize][x as usize] = if (pixel[0] > 140) && (pixel[1] <= 170) && (pixel[2] <= 170) {true} else {false};
	}
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
	println!("Finished R");
	for (x, y, pixel) in rgbimg.enumerate_pixels() {
		thresh[y as usize][x as usize] = if (pixel[1] > 140) && (pixel[0] <= 170) && (pixel[2] <= 170) {true} else {false};
	}
	for y in 0..thresh.len() {
		for x in 0..thresh[0].len() {
			if thresh[y][x] {
				match ImgBlob::from_top_right(x, y, &mut claimed, &thresh){
					Some(o) => gblobs.push(o),
					None => {},
				}
			}
		}
	}
	println!("Finished G");
	for (x, y, pixel) in rgbimg.enumerate_pixels() {
		thresh[y as usize][x as usize] = if (pixel[2] > 140) && (pixel[1] <= 170) && (pixel[0] <= 170) {true} else {false};
	}
	for y in 0..thresh.len() {
		for x in 0..thresh[0].len() {
			if thresh[y][x] {
				match ImgBlob::from_top_right(x, y, &mut claimed, &thresh){
					Some(o) => bblobs.push(o),
					None => {},
				}
			}
		}
	}
	println!("Finished B");*/
}
