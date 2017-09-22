#![crate_name = "notedigest"]
/// Application that converts handwritten notes into organized html pages.

extern crate image;
extern crate uuid;


use std::fs;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::io;
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use image::GenericImage;
use uuid::Uuid;

/// The location where a list of already imported files may be found
const IMPORTED: &'static str = "./.imported";

/// The location where the organized notes should be written to
const OUT_PATH: &'static str = "~/Documemts/Notebook";

/// Minimum value for a channel to be considered on
const MIN_THRESH: u8 = 140;

/// Maximum value for a channel to be considered off
const MAX_THRESH: u8 = 170;

const RED: u8 = 0;
const GREEN: u8 = 1;
const BLUE: u8 = 2;

//should be a trait.  I am not sure how to impliment one for only Vec <Vec <bool>> and not Vec <T>
/** Inserts boolean into `Vec <Vec <bool>>` at specified point.  If the point does not exist the vector is expanded.

 # Arguments

 * `y` - A 64 bit integer with the row to insert relative to the top right corner.

 * `x` - A 64 bit integer with the collum to insert relative to the top right corner.

 * `value` - The boolean value you wish to insert

 * `img` - The 2d boolean vector.  Must be `&mut`
 */
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
/// Monochrome image fragment
struct ImgBlob {
	/// Is it a line?
	/// 0: object, 1: line
	blob_type: u8,

	/// Top left coordinate. (collum first then row)
	top_left: [usize; 2],

	/// Bottom right coordinate. (collum first then row)
	bottom_right: [usize; 2],

	/// 2d array of booleans representing monochrome image fragment
	bitmap: Vec <Vec <bool>>
}

impl ImgBlob {
	/// Checks using a floodfill if an image blob can be started at a coordinate.
	/// If one can be found it returns it.
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

#[derive(Clone)]
/// A group of blob objects along with the color information for the blobs
struct Clump {
	/// What color is it? Uses color constants
	ctype: u8,

	/// Blob objects
	blobs: Vec <ImgBlob>
}

impl Clump {
	/// Add an `ImgBlob` object to an array of clumps.
	/// If the blob is not of the same type as the current clump create a new clump.
	/// Otherwise add it to the current clump.
	fn clump_update(blob:ImgBlob, t: u8, clumps: &mut Vec <Clump>){
		let clen = clumps.len();
		if clen > 0 {
			if t == clumps[clen-1].ctype {
				clumps[clen-1].blobs.push(blob);
			} else {
				clumps.push(
					Clump {
						ctype: t,
						blobs: vec![blob]
					}
				);
			}
		} else {
			clumps.push(
				Clump {
					ctype: t,
					blobs: vec![blob]
				}
			);
		}
	}

	/// Convert a clump to a grayscale image object.  Used for debugging
	fn to_image(&self, w: u32, h: u32) -> image::ImageBuffer<image::LumaA<u8>, Vec<u8>> {
		let mut imgbuf = image::ImageBuffer::<image::LumaA<u8>, Vec<u8>>::new(
			w, h
		);
		for b in &self.blobs {
			let xoff = b.top_left[0] as u32;
			let yoff = b.top_left[1] as u32;
			let mut y: usize = 0;
			while y < b.bitmap.len() {
				let mut x: usize = 0;
				while x < b.bitmap[y].len() {
					if b.bitmap[y][x] {
						imgbuf.put_pixel(
							(x as u32)+xoff,
							(y as u32)+yoff,
							image::LumaA([0, 255])
						);
					}
					x += 1;
				}
				y += 1;
			}
		}
		imgbuf
	}
}

/// Representation of a page.  Holds clumps and original page dimensions.
struct Page {
	/// Vector of `Clump` objects
	clumps: Vec <Clump>,

	/// Dimensions of original page
	dimensions: [u32; 2]
}

impl Page {
	fn get_highest(a:usize, b:usize, c:usize) -> u8 {
		if a <= b && a <= c {
			0
		} else if a > b && b <= c {
			1
		} else {
			2
		}
	}

	/// Convert three channels of `ImgBlob` vectors to clumps in `Page` objects.
	fn from_blobs(
		mut rblobs: Vec <ImgBlob>,
		mut gblobs: Vec <ImgBlob>,
		mut bblobs: Vec <ImgBlob>,
		dimensions: [u32; 2]
	) -> Page {
		print!("\r◔: Clustering objects ");
		std::io::stdout().flush().ok().expect("Could not flush STDOUT!");
		let mut clumps = Vec::new();
		let mut rpos;
		let mut gpos;
		let mut bpos;
		while rblobs.len() + gblobs.len() + bblobs.len() > 0 {
			if rblobs.len() > 0 {
				rpos = rblobs[0].top_left[1];
			} else {
				rpos = <usize>::max_value();
			}
			if gblobs.len() > 0 {
				gpos = gblobs[0].top_left[1];
			} else {
				gpos = <usize>::max_value();
			}
			if bblobs.len() > 0 {
				bpos = bblobs[0].top_left[1];
			} else {
				bpos = <usize>::max_value();
			}
			match Page::get_highest(rpos, gpos, bpos) {
				0 => Clump::clump_update(rblobs.remove(0), 0, &mut clumps),
				1 => Clump::clump_update(gblobs.remove(0), 1, &mut clumps),
				2 => Clump::clump_update(bblobs.remove(0), 2, &mut clumps),
				_ => panic!("Invalid clump type(>2)"),
			};
		}
		Page {
			clumps: clumps,
			dimensions: dimensions
		}
	}

	/// Create a `Page` object from a file path.
	fn from_path(path: String) -> Page {
		fn thresh_and_blob(
			rgbimg: &image::RgbImage,
			channel: u8,
			claimed: &mut Vec <Vec <bool>>,
			thresh: &mut Vec <Vec <bool>>,
			blobs: &mut Vec <ImgBlob>
		) {
			match channel {
				RED => {
					for (x, y, pixel) in rgbimg.enumerate_pixels() {
						thresh[y as usize][x as usize] = if (pixel[0] > MIN_THRESH) && (pixel[1] <= MAX_THRESH) && (pixel[2] <= MAX_THRESH) {true} else {false};
					}
				},
				GREEN => {
					for (x, y, pixel) in rgbimg.enumerate_pixels() {
						thresh[y as usize][x as usize] = if (pixel[1] > MIN_THRESH) && (pixel[0] <= MAX_THRESH) && (pixel[2] <= MAX_THRESH) {true} else {false};
					}
				},
				BLUE => {
					for (x, y, pixel) in rgbimg.enumerate_pixels() {
						thresh[y as usize][x as usize] = if (pixel[2] > MIN_THRESH) && (pixel[1] <= MAX_THRESH) && (pixel[0] <= MAX_THRESH) {true} else {false};
					}
				},
				_ => panic!("Invalid color")
			}
			for y in 0..thresh.len() {
				for x in 0..thresh[0].len() {
					if thresh[y][x] {
						match ImgBlob::from_top_left(x, y, claimed, &thresh){
							Some(o) => blobs.push(o),
							None => {},
						}
					}
				}
			}
		}
		let mut claimed: Vec <Vec <bool>> = Vec::new();
		let mut thresh: Vec <Vec <bool>> = Vec::new();
		let mut rblobs: Vec <ImgBlob> = Vec::new();
		let mut gblobs: Vec <ImgBlob> = Vec::new();
		let mut bblobs: Vec <ImgBlob> = Vec::new();
		let mut img = image::open(&Path::new(&path)).unwrap();
		img = img.adjust_contrast(-22f32);
		let mut row:Vec <bool> = Vec::new();
		for x in 0..img.width() {
			row.push(false);
		}
		for y in 0..img.height() {
			claimed.push(row.clone());
			thresh.push(row.clone());
		}
		let rgbimg = img.to_rgb();
		thresh_and_blob(&rgbimg, RED, &mut claimed, &mut thresh, &mut rblobs);
		thresh_and_blob(&rgbimg, GREEN, &mut claimed, &mut thresh, &mut gblobs);
		thresh_and_blob(&rgbimg, BLUE, &mut claimed, &mut thresh, &mut bblobs);
		Page::from_blobs(
			rblobs,
			gblobs,
			bblobs,
			[img.width(), img.height()]
		)
	}
}

#[derive(Clone)]
/// Representation of a heading for a section object.  Shown in tables.
struct Subject {
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

#[derive(Clone)]
/// Representation of the remainder of the content in a object.  Hidden by default.
struct Extension {
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

#[derive(Clone)]
/// Representation of a heading.
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
	subject: Subject
}

/*impl Heading {
	fn update_size_pos(&mut self, page: &Page) {
		let mut top: u32 = <u32>::max_value();
		let mut left: u32 = <u32>::max_value();
		let mut bottom: u32 = 0;
		let mut right: u32 = 0;
		for b in &self.blobs {
			if b.top_left[1] < top as usize {
				top = b.top_left[1] as u32;
			}
			if b.top_left[0] < left as usize {
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
			if l.top_left[1] < top as usize {
				top = l.top_left[1] as u32;
			}
			if l.top_left[0] < left as usize {
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
		self.width_pix = right - left;
		self.width_precent = (self.width_pix as f64) / (page.dimensions[1] as f64);
		self.height_pix = bottom - top;
		self.height_precent = (self.left_pix as f64) / (page.dimensions[0] as f64);
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
	fn to_image(&self) -> image::ImageBuffer<image::LumaA<u8>, Vec<u8>> {
		let mut imgbuf = image::ImageBuffer::<image::LumaA<u8>, Vec<u8>>::new(
			(self.width_pix as u32), (self.height_pix as u32)
		);
		for b in &self.blobs {
			let xoff = (b.top_left[0] as u32) - self.left_pix;
			let yoff = (b.top_left[1] as u32) - self.top_pix;
			let mut y: usize = 0;
			while y < b.bitmap.len() {
				let mut x: usize = 0;
				while x < b.bitmap[y].len() {
					if b.bitmap[y][x] {
						imgbuf.put_pixel(
							(x as u32)+xoff,
							(y as u32)+yoff,
							image::LumaA([0, 255])
						);
					}
					x += 1;
				}
				y += 1;
			}
		}
		imgbuf
	}
}*/

/// Definition or important idea
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
	subject: Subject, // Just the header
	extension: Extension // just the content
}

impl Idea {
	//fn cluster() -> Vec <Idea> {
	//	
	//}
}

/// Content cluster
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
	fn update_size_pos(&mut self, dim: [u32; 2]) {
		let mut top: u32 = <u32>::max_value();
		let mut left: u32 = <u32>::max_value();
		let mut bottom: u32 = 0;
		let mut right: u32 = 0;
		for b in &self.blobs {
			if b.top_left[1] < top as usize {
				top = b.top_left[1] as u32;
			}
			if b.top_left[0] < left as usize {
				left = b.top_left[0] as u32;
			}
			if b.bottom_right[1] > bottom as usize {
				bottom = b.bottom_right[1] as u32;
			}
			if b.bottom_right[0] > right as usize {
				right = b.bottom_right[0] as u32;
			}
		}
		self.top_pix = top;
		self.top_precent = (top as f64) / (dim[1] as f64);
		self.left_pix = left;
		self.left_precent = (left as f64) / (dim[0] as f64);
		self.width_pix = right - left;
		self.width_precent = (self.width_pix as f64) / (dim[1] as f64);
		self.height_pix = bottom - top;
		self.height_precent = (self.left_pix as f64) / (dim[0] as f64);
	}
	fn new(blobs: Vec <ImgBlob>, dim: [u32; 2]) -> Content {
		let mut out = Content {
			id: Uuid::new_v4(),
			top_pix: 0u32,
			top_precent: 0f64,
			left_pix: 0u32,
			left_precent: 0f64,
			width_pix: 0u32,
			width_precent: 0f64,
			height_pix: 0u32,
			height_precent: 0f64,
			blobs: blobs.clone()
		};
		out.update_size_pos(dim);
		out
	}
}

/// Objects holding `Heading`, `Idea`, and `Content` objects
struct Chapter {
	//heading: Heading,
	sub_headings: Vec <Heading>,
	ideas: Vec <Idea>,
	content: Vec <Content>
}

impl Chapter {
	/// Create a `Chapter` object
	fn new() -> Chapter {
		Chapter {
			//heading: Heading::new(),
			sub_headings: Vec::new(),
			ideas: Vec::new(),
			content: Vec::new()
		}
	}
}

/// Get a vector of image paths to import from a user.
fn get_images() -> Vec <String> {
	/// Get vector containing already imported images from Imported.
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
	/// Get user selections as a `Vec <String>`
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
	print!("select: ");
	std::io::stdout().flush().ok().expect("Could not flush STDOUT!");
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

/// Add content objects to chapter or destroy them because they lack a chapter.
fn add_content(
	clump: Clump,
	page: &Page,
	chapter: &mut Chapter,
	destroyed: &mut usize,
	started: bool
) {
	if started {
		chapter.content.push(Content::new(clump.blobs, page.dimensions));
	} else {
		*destroyed += 1;
	}
}

/// Create new chapter, add heading objects to chapter, or destroy them because they lack a chapter.
fn add_heading(
	clump: Clump,
	page: &Page,
	chapter: &mut Chapter,
	destroyed: &mut usize,
	started: &mut bool
) {
	let i: usize = 0;
	while i < clump.blobs.len() {
		let blob = clump.blobs[i];
		if blob.blob_type == 1 {
			
		}
	}
	if *started {
		
	} else {
		
	}
}

/// Entry point to the program
fn main() {
	//iterate through images pulling out clumps
	//iterate through pages parsing clumps and creating chapters
	let selected = get_images();
	let mut pages: Vec <Page> = Vec::new();
	print!("○: Identifying objects");
	std::io::stdout().flush().ok().expect("Could not flush STDOUT!");
	for img in selected {
		pages.push(Page::from_path(img));
	}
	print!("\r◑: Dividing by chapter");
	std::io::stdout().flush().ok().expect("Could not flush STDOUT!");
	let mut chapter: Chapter = Chapter::new();
	let mut started = false;
	let mut created_chapters = 0;
	let mut destroyed: usize = 0;
	for mut p in pages {
		let mut headings: Vec <Heading> = Vec::new();
		let mut headings1: Vec <Heading> = Vec::new();
		let mut headings2: Vec <Heading> = Vec::new();
		let mut i: usize = 0;
		while i < p.clumps.len() {
			let img = (&p.clumps[i]).to_image(p.dimensions[0], p.dimensions[1]);
			let num = &i.to_string()[..];
			let ref mut fout = File::create(&Path::new(&(String::from("outC")+num+".png")[..])).unwrap();
			let _ = image::ImageLumaA8(img).save(fout, image::PNG);
			match p.clumps[i].ctype {
				RED   => {},//Heading(s) of some type
				GREEN => {},//Defintions(s) of some type
				BLUE  => add_content(p.clumps[i].clone(), &p, &mut chapter, &mut destroyed, started),//Content"
				_ => panic!("Invalid Content")
			};
			i += 1;
		}
	}
	print!("\r◕: Writing            ");
	std::io::stdout().flush().ok().expect("Could not flush STDOUT!");
	println!("\r●: Done               ");
	println!("{} chapters added.  {} orphaned objects destroyed", created_chapters, destroyed);
}
