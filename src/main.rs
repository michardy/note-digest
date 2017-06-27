extern crate image;
extern crate uuid;


use std::fs;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::io;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use image::GenericImage;
use uuid::Uuid;

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
	while img.len() <= ty as usize {
		img.push(row.clone())
	}
	img[ty as usize][tx as usize] = value;
}

#[derive(Debug, Clone)]
struct ImgBlob {
	blob_type: u8, //0: object, 1: line
	top_left: [usize; 2],
	bottom_right: [usize; 2],
	bitmap: Vec <Vec <bool>>
}

impl ImgBlob {
	fn from_top_left(x: usize, y: usize, claim: &mut Vec <Vec <bool>>, img: &Vec <Vec <bool>>) -> Option<ImgBlob>{
		let mut left = x;
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
					boundless_insert(((tempy as i64)-(top as i64)), ((tempx as i64)-(left as i64)), true, &mut bitmap);
					if tempx < left {
						left = tempx;
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
				top_left: [left, top],
				bottom_right: [left+bitmap[0].len(), top+bitmap.len()],
				bitmap: bitmap
			})
		} else {
			None
		}
	}
}

struct Page {
	rblobs: Vec <ImgBlob>,
	gblobs: Vec <ImgBlob>,
	bblobs: Vec <ImgBlob>,
	dimensions: [u32; 2]
}

impl Page {
	fn from_path(path: String) -> Page {
		let mut claimed: Vec <Vec <bool>> = Vec::new();
		let mut thresh: Vec <Vec <bool>> = Vec::new();
		let mut rblobs: Vec <ImgBlob> = Vec::new();
		let mut gblobs: Vec <ImgBlob> = Vec::new();
		let mut bblobs: Vec <ImgBlob> = Vec::new();
		let img = image::open(&Path::new(&path)).unwrap();
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
					match ImgBlob::from_top_left(x, y, &mut claimed, &thresh){
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
					match ImgBlob::from_top_left(x, y, &mut claimed, &thresh){
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
					match ImgBlob::from_top_left(x, y, &mut claimed, &thresh){
						Some(o) => bblobs.push(o),
						None => {},
					}
				}
			}
		}
		println!("Finished B");
		Page {
			rblobs: rblobs,
			gblobs: gblobs,
			bblobs: bblobs,
			dimensions: [img.width(), img.height()]
		}
	}
}

#[derive(Clone)]
struct Heading {
	number: u8,
	top_pix: u32,
	top_precent: f64,
	left_pix: u32,
	left_precent: f64,
	width_pix: u32,
	width_precent: f64,
	height_pix: u32,
	height_precent: f64,
	lines: Vec <ImgBlob>,
	blobs: Vec <ImgBlob>,
}

impl Heading {
	fn new() -> Heading {
		Heading {
			number: 0,
			top_pix: 0u32,
			top_precent: 0f64,
			left_pix: 0u32,
			left_precent: 0f64,
			width_pix: 0u32,
			width_precent: 0f64,
			height_pix: 0u32,
			height_precent: 0f64,
			lines: Vec::new(),
			blobs: Vec::new()
		}
	}
	fn heading_one(top: ImgBlob, bottom: ImgBlob) -> Heading {
		let mut lines: Vec <ImgBlob> = Vec::new();
		let mut blobs: Vec <ImgBlob> = Vec::new();
		lines.push(top);
		lines.push(bottom);
		Heading {
			number: 0,
			top_pix: 0u32,
			top_precent: 0f64,
			left_pix: 0u32,
			left_precent: 0f64,
			width_pix: 0u32,
			width_precent: 0f64,
			height_pix: 0u32,
			height_precent: 0f64,
			lines: lines,
			blobs: blobs
		}
	}
	fn heading_two(top: ImgBlob, ) -> Heading {
		let mut lines: Vec <ImgBlob> = Vec::new();
		let mut blobs: Vec <ImgBlob> = Vec::new();
		lines.push(top);
		Heading {
			number: 1,
			top_pix: 0u32,
			top_precent: 0f64,
			left_pix: 0u32,
			left_precent: 0f64,
			width_pix: 0u32,
			width_precent: 0f64,
			height_pix: 0u32,
			height_precent: 0f64,
			lines: lines,
			blobs: blobs
		}
	}
	fn update_size_pos(&mut self, page: &Page) {
		let mut top: u32 = 0;
		let mut left: u32 = 0;
		let mut bottom: u32 = 0;
		let mut right: u32 = 0;
		for b in &self.blobs {
			if b.top_left[1] > top as usize {
				top = b.top_left[1] as u32;
			}
			if b.top_left[0] > left as usize {
				left = b.top_left[0] as u32;
			}
			if b.bottom_right[1] > bottom as usize {
				bottom = b.bottom_right[1] as u32;
			}
			if b.bottom_right[0] > right as usize {
				right = b.bottom_right[0] as u32;
			}
		}
		for l in &self.lines {
			if l.top_left[1] > top as usize {
				top = l.top_left[1] as u32;
			}
			if l.top_left[0] > left as usize {
				left = l.top_left[0] as u32;
			}
			if l.bottom_right[1] > bottom as usize {
				bottom = l.bottom_right[1] as u32;
			}
			if l.bottom_right[0] > right as usize {
				right = l.bottom_right[0] as u32;
			}
		}
		self.top_pix = top;
		self.top_precent = (top as f64) / (page.dimensions[1] as f64);
		self.left_pix = left;
		self.left_precent = (left as f64) / (page.dimensions[0] as f64);
		self.width_pix = bottom - top;
		self.width_precent = (self.width_pix as f64) / (page.dimensions[1] as f64);
		self.left_pix = right - left;
		self.left_precent = (self.left_pix as f64) / (page.dimensions[0] as f64);
	}
	fn cluster(blobs: &mut Vec <ImgBlob>, page: Page) -> Vec <Heading> {
		let mut b: usize = 0;
		let mut out: Vec <Heading> = Vec::new();
		let mut curcluster: Vec <ImgBlob> = Vec::new();
		while b < blobs.len() {
			let mut ci: usize = 0;
			curcluster.push(blobs.remove(b));
			while ci < curcluster.len() {
				let mut s: usize = 0;
				while s < blobs.len() {
					let x: f64 = curcluster[ci].top_left[0] as f64 - blobs[s].top_left[0] as f64;
					let y: f64 = curcluster[ci].top_left[1] as f64 - blobs[s].top_left[1] as f64;
					if (x.abs().powf(2.0) + y.abs().powf(2.0)).sqrt() < (0.09)*(page.dimensions[1] as f64) {
						curcluster.push(blobs.remove(s));
					} else {
						s += 1
					}
				}
				ci += 1;
			}
			let mut head = Heading {
				number: 2,
				top_pix: 0u32,
				top_precent: 0f64,
				left_pix: 0u32,
				left_precent: 0f64,
				width_pix: 0u32,
				width_precent: 0f64,
				height_pix: 0u32,
				height_precent: 0f64,
				lines: Vec::new(),
				blobs: curcluster.clone()
			};
			head.update_size_pos(&page);
			out.push(head);
		}
		out
	}
}

struct Idea {
	id: Uuid,
	top_pix: u32,
	top_precent: f64,
	left_pix: u32,
	left_precent: f64,
	width_pix: u32,
	width_precent: f64,
	height_pix: u32,
	height_precent: f64,
	heading: Heading,
	blobs: Vec <ImgBlob>
}

impl Idea {
	//fn cluster() -> Vec <Idea> {
	//	
	//}
}

struct Content {
	id: Uuid,
	top_pix: u32,
	top_precent: f64,
	left_pix: u32,
	left_precent: f64,
	width_pix: u32,
	width_precent: f64,
	height_pix: u32,
	height_precent: f64,
	blobs: Vec <ImgBlob>
}

impl Content {
	//fn cluster() -> Vec <Content> {
	//	
	//}
}
	
struct Chapter {
	heading: Heading,
	sub_headings: Vec <Heading>,
	ideas: Vec <Idea>,
	content: Vec <Content>
}

impl Chapter {
	fn new() -> Chapter {
		Chapter {
			heading: Heading::new(),
			sub_headings: Vec::new(),
			ideas: Vec::new(),
			content: Vec::new()
		}
	}
}

fn get_images() -> Vec <String> {
	fn get_imported_images() -> Vec <String> {
		if Path::new(IMPORTED).exists() {
			let mut list: Vec <String> = Vec::new();
			let f = (File::open(IMPORTED)).unwrap();
			let mut file = BufReader::new(&f);
			for line in file.lines() {
				let templ = line.unwrap();
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

fn get_blob_type(blobs: &Vec <ImgBlob>, index: usize) -> u8 {
	match blobs.get(index) {
		Some(b) => b.blob_type,
		None => 255
	}
}

fn get_head_height(heads: &Vec <Heading>, index: usize) -> usize {
	match heads.get(index) {
		Some(h) => h.lines[0].top_left[1],
		None => 255
	}
}

fn add_chapter(chapter: Chapter) {
	fn setup_dirs() {
		fs::create_dir_all(OUT_PATH);
	}
	//Probably not the solution
	//should probably do page leve clumping calculate pos from that and assemble later
	/*fn blobs_to_image(blobs: Vec <ImgBlob>) -> image::ImageBuffer {
		let mut imgbuf = image::ImageBuffer::new(imgx, imgy);
	}*/
	fn update_toc() {
		
	}
	fn make_chapter() {
		
	}
	let cuid = Uuid::new_v4();
	if !Path::new(OUT_PATH).exists() {
		setup_dirs();
	}
}

fn main() {
	//iterate through images pulling out blobs
	//iterate through pages parsing blobs and creating chapters
	let selected = get_images();
	let mut pages: Vec <Page> = Vec::new();
	for img in selected {
		pages.push(Page::from_path(img));
	}
	let mut chapter: Chapter = Chapter::new();
	let mut started = false;
	let mut created_chapters = 0;
	let mut destroyed: usize = 0;
	for mut p in pages {
		let mut headings: Vec <Heading> = Vec::new();
		let mut headings1: Vec <Heading> = Vec::new();
		let mut headings2: Vec <Heading> = Vec::new();
		let mut i: usize = 0;
		while i < p.rblobs.len() {
			if p.rblobs[i].blob_type == 1u8 && get_blob_type(&p.rblobs, i+1) == 1u8 {
				headings1.push(Heading::heading_one(p.rblobs.remove(i), p.rblobs.remove(i)));
				i -= 1;
				if i >= p.rblobs.len() {
					break;
				}
			} else if p.rblobs[i].blob_type == 1 {
				headings2.push(Heading::heading_two(p.rblobs.remove(i)));
				i -= 1;
			}
			i += 1;
		}
		let mut h2: usize = 0;
		while h2 < headings2.len() {
			let mut b: usize = 0;
			let mut previous: Vec <ImgBlob> = Vec::new();
			let mut current: Vec <ImgBlob> = Vec::new();
			while b < p.rblobs.len() {
				if p.rblobs[b].top_left[1] < ((headings2[h2].lines[0].top_left[1]) - (1/22*p.dimensions[1]) as usize) {
					headings2[h2].blobs.push(p.rblobs.remove(b));
				}
				b += 1;
			}
			h2 += 1;
		}
		let mut h: usize = 0;
		while h < headings1.len() {
			let mut b: usize = 0;
			let mut previous: Vec <ImgBlob> = Vec::new();
			let mut current: Vec <ImgBlob> = Vec::new();
			let mut sub: Vec <Heading> = Vec::new();
			while b < p.rblobs.len() {
				if p.rblobs[b].top_left[1] < ((headings1[h].lines[0].top_left[1]) - (1/22*p.dimensions[1]) as usize) {
					headings1[h].blobs.push(p.rblobs[b].clone());
				} else if (p.rblobs[b].top_left[1] > headings1[h].lines[0].top_left[1]) && (p.rblobs[b].top_left[1] < ((get_head_height(&headings, h+1) as usize) - (1/22*p.dimensions[1]) as usize)){
					current.push(p.rblobs[b].clone());
				} else {
					previous.push(p.rblobs[b].clone());
				}
				b += 1;
			}
			(&mut headings1[h]).update_size_pos(&p);
			let mut h2: usize = 0;
			while h2 < headings2.len() {
				if (headings2[h2].top_pix > (headings1[h].lines[0].top_left[1] as u32)) && (headings2[h2].top_pix < ((get_head_height(&headings, h+1) as usize) - (1/22*p.dimensions[1]) as usize) as u32){
					(&mut headings2[h2]).update_size_pos(&p);
					sub.push(headings2[h2].clone())
				}
				h2 += 1;
			}
			if started {
				//chapter.blobs.append(&mut previous);
			} else {
				destroyed += previous.len();
			}
			h += 1;
		}
	}
	println!("{} chapters added.  {} orphaned objects destroyed", created_chapters, destroyed)
}
