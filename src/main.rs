#![crate_name = "notedigest"]
/// Application that converts handwritten notes into organized html pages.

extern crate image;
extern crate dirs_next;
extern crate uuid;

use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::io::Write;
use std::io::Read;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::fs::OpenOptions;
use image::GenericImageView;
use image::DynamicImage::ImageLumaA8;
use uuid::Uuid;

/// The location where a list of already imported files may be found
const IMPORTED: &'static str = "./.imported";

/// The location where the organized notes should be written to
const OUT_PATH: &'static str = "Documents/Notebook/";

/// Minimum value for a channel to be considered on
const MIN_THRESH: u8 = 120;

/// Maximum value for a channel to be considered off
const MAX_THRESH: u8 = 126;

/// Minimum width to heigth ratio for object to be considered a line
const LINE_RATIO: f32 = 7.0;

/// Defines red channel index
const RED: u8 = 0;
/// Defines green channel index
const GREEN: u8 = 1;
/// Defines blue channel index
const BLUE: u8 = 2;

//should be a trait.  I am not sure how to impliment one for only Vec <Vec <bool>> and not Vec <T>
/** Inserts boolean into `Vec <Vec <bool>>` at specified point.  If the point does not exist the vector is expanded.

 # Arguments

 * `y` - A 64 bit integer with the row to insert relative to the top right corner.

 * `x` - A 64 bit integer with the collum to insert relative to the top right corner.

 * `value` - The boolean value you wish to insert

 * `img` - The 2d boolean vector.  Must be `&mut`
 */
fn boundless_insert(
	y: i64,
	x: i64,
	value: bool,
	img: &mut Vec<Vec <bool>>
) {
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
	for _ in 0..img[0].len() {
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

#[derive(Debug, Clone, PartialEq)]
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
	fn from_top_left(
		x: usize,
		y: usize,
		claim: &mut Vec <Vec <bool>>,
		img: &Vec <Vec <bool>>
	) -> Option<ImgBlob>{
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
			if
				(tempy < img.len()) &&
				(tempx < img[0].len()) &&
				(tempy > 0) && (tempx > 0)
			{
				if img[tempy][tempx] && !(claim[tempy][tempx]){
					boundless_insert(
						(tempy as i64)-(top as i64),
						(tempx as i64)-(left as i64),
						true,
						&mut bitmap
					);
					if tempx < left {
						left = tempx;
					}
					if tempy < top {
						top = tempy;
					}
					claim[tempy][tempx] = true;
					if tempy > top {
						queue.push([tempy-1, tempx-1]);
						queue.push([tempy-1, tempx]);
						queue.push([tempy-1, tempx+1]);
					}
					queue.push([tempy, tempx-1]);
					queue.push([tempy+1, tempx-1]);
					queue.push([tempy+1, tempx]);
					queue.push([tempy+1, tempx+1]);
					queue.push([tempy, tempx+1]);
				}
			}
			queue.remove(0);
		}
		if (
			bitmap[0].len() + bitmap.len() > 16) ||
			(bitmap.len() > 8) && (bitmap[0].len() > 8
		) {
			Some(ImgBlob {
				blob_type:
					if
						(bitmap[0].len() as f32 / bitmap.len() as f32) > LINE_RATIO &&
						bitmap[0].len() > 60
					{1} else {0},
				top_left: [left, top],
				bottom_right: [left+bitmap[0].len(), top+bitmap.len()],
				bitmap: bitmap
			})
		} else {
			None
		}
	}
	fn new() -> ImgBlob {
		ImgBlob {
			bitmap: Vec::new(),
			blob_type: 0,
			bottom_right: [0, 0],
			top_left: [0, 0]
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
	fn clump_update(blob:ImgBlob, t: u8, tn: u8, clumps: &mut Vec <Clump>){
		let clen = clumps.len();
		if clen > 0 {
			if t == clumps[clen-1].ctype {
				clumps[clen-1].blobs.push(blob);
			} else {
				match t {
					RED   => if ((blob.bottom_right[0] - blob.top_left[0]) +
						(blob.bottom_right[1] - blob.top_left[1]) > 50) &&
						tn == RED
					{
						clumps.push(
							Clump {
								ctype: t,
								blobs: vec![blob]
							}
						);
					},
					GREEN => if tn == GREEN {
						clumps.push(
							Clump {
								ctype: t,
								blobs: vec![blob]
							}
						);
					},
					_     => clumps.push(
						Clump {
							ctype: t,
							blobs: vec![blob]
						}
					),
				}
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
			let high_color = Page::get_highest(rpos, gpos, bpos);
			let blob = match high_color {
				RED   => rblobs.remove(0),
				GREEN => gblobs.remove(0),
				BLUE  => bblobs.remove(0),
				_ => panic!("Invalid clump type(>2)"),
			};
			let next_highest = Page::get_highest(rpos, gpos, bpos);
			Clump::clump_update(
				blob,
				high_color,
				next_highest,
				&mut clumps
			);
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
						thresh[y as usize][x as usize] =
							if
								(pixel[0] > MIN_THRESH) &&
								(pixel[1] <= MAX_THRESH) &&
								(pixel[2] <= MAX_THRESH)
							{true} else {false};
					}
				},
				GREEN => {
					for (x, y, pixel) in rgbimg.enumerate_pixels() {
						thresh[y as usize][x as usize] =
						if
							(pixel[1] > MIN_THRESH) &&
							(pixel[0] <= MAX_THRESH) &&
							(pixel[2] <= MAX_THRESH)
						{true} else {false};
					}
				},
				BLUE => {
					for (x, y, pixel) in rgbimg.enumerate_pixels() {
						thresh[y as usize][x as usize] =
							if
								(pixel[2] > MIN_THRESH) &&
								(pixel[1] <= MAX_THRESH) &&
								(pixel[0] <= MAX_THRESH)
							{true} else {false};
					}
				},
				_ => panic!("Invalid color")
			}
			for y in 0..thresh.len() {
				for x in 0..thresh[0].len() {
					if thresh[y][x] && !claimed[y][x] {
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
		let img = image::open(&Path::new(&path)).unwrap();
		//img = img.adjust_contrast(-22f32);
		let mut row:Vec <bool> = Vec::new();
		for _ in 0..img.width() {
			row.push(false);
		}
		for _ in 0..img.height() {
			claimed.push(row.clone());
			thresh.push(row.clone());
		}
		let mut rgbimg = img.to_rgb8();
		rgbimg.filter();
		thresh_and_blob(
			&rgbimg,
			RED,
			&mut claimed,
			&mut thresh,
			&mut rblobs
		);
		thresh_and_blob(
			&rgbimg,
			GREEN,
			&mut claimed,
			&mut thresh,
			&mut gblobs
		);
		thresh_and_blob(
			&rgbimg,
			BLUE,
			&mut claimed,
			&mut thresh,
			&mut bblobs
		);
		Page::from_blobs(
			rblobs,
			gblobs,
			bblobs,
			[img.width(), img.height()]
		)
	}
}

#[derive(Clone)]
/// Representation of a heading.
struct Heading {
	id: Uuid,
	number: u8, /// Heading number.
	subject: Content
}

impl Heading {
	fn new() -> Heading {
		Heading {
			id: Uuid::new_v4(),
			number:3,
			subject: Content::empty()
		}
	}
}

/// Definition or important idea
#[derive(Clone)]
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
	subject: Content, // Just the header
	extension: Content // just the content
}

impl Idea {
	fn new() -> Idea {
		Idea {
			id: Uuid::new_v4(),
			top_pix: 0,
			top_precent: 0.0,
			left_pix: 0,
			left_precent: 0.0,
			width_pix: 0,
			width_precent: 0.0,
			height_pix: 0,
			height_precent: 0.0,
			subject: Content::empty(), // Just the header
			extension: Content::empty() // just the content
		}
	}
	fn update_size_pos(&mut self, dim: [u32; 2]) {
		if self.subject.top_pix < self.extension.top_pix {
			self.top_pix = self.subject.top_pix;
		} else {
			self.top_pix = self.extension.top_pix;
		}
		if self.subject.left_pix < self.extension.left_pix {
			self.left_pix = self.subject.left_pix;
		} else {
			self.left_pix = self.extension.left_pix;
		}
		if
			self.subject.top_pix + self.subject.height_pix >
			self.extension.top_pix + self.extension.height_pix
		{
			self.height_pix = (
				self.subject.top_pix + self.subject.height_pix
			) - self.top_pix;
		} else {
			self.height_pix = (
				self.extension.top_pix + self.extension.height_pix
			) - self.top_pix;
		}
		if
			self.subject.left_pix + self.subject.width_pix >
			self.extension.left_pix + self.extension.width_pix
		{
			self.width_pix = (
				self.subject.left_pix + self.subject.width_pix
			) - self.left_pix;
		} else {
			self.width_pix = (
				self.extension.left_pix + self.extension.width_pix
			) - self.left_pix;
		}
		self.left_precent = (self.left_pix as f64) / (dim[0] as f64);
		self.top_precent = (self.top_pix as f64) / (dim[1] as f64);
		self.width_precent = (self.width_pix as f64) / (dim[0] as f64);
		self.height_precent = (self.height_pix as f64) / (dim[1] as f64);
	}
}

/// Content cluster
#[derive(Clone)]
struct Content {
	id: Uuid,
	top_pix: u32,
	top_total_pix: u32, // workaround to fix index out of bounds error caused when writing image out
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
	fn update_size_pos(&mut self, dim: [u32; 2], cur_height: i64) {
		let mut top: i64 = (<u32>::max_value()) as i64;
		let mut left: u32 = <u32>::max_value();
		let mut bottom: u32 = 0;
		let mut right: u32 = 0;
		assert!(
			&self.blobs.len() > &0,
			"Output Generation: Error empty content object"
		);
		for b in &self.blobs {
			if b.top_left[1] < top as usize {
				top = b.top_left[1] as i64;
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
		self.top_pix = top as u32;
		self.top_total_pix = (top + cur_height) as u32;
		// The height percents should not be final and should probably not be set here
		self.top_precent = (self.top_total_pix as f64) / ((dim[1] as i64 + cur_height) as f64);
		self.left_pix = left;
		self.left_precent = (left as f64) / (dim[0] as f64);
		self.width_pix = right - left;
		self.width_precent = (self.width_pix as f64) / (dim[0] as f64);
		self.height_pix = (bottom as i64 - top) as u32;
		self.height_precent = (self.height_pix as f64) / ((dim[1] as i64 + cur_height) as f64);
	}
	fn update_top(&mut self, cur_height: &i64) {
		self.top_precent = (self.top_total_pix as f64) / (*cur_height as f64);
		self.height_precent = (self.height_pix as f64) / (*cur_height as f64);
	}
	fn new(blobs: Vec <ImgBlob>, dim: [u32; 2], cur_height: i64) -> Content {
		let mut out = Content {
			id: Uuid::new_v4(),
			top_pix: 0u32,
			top_total_pix: 0u32,
			top_precent: 0f64,
			left_pix: 0u32,
			left_precent: 0f64,
			width_pix: 0u32,
			width_precent: 0f64,
			height_pix: 0u32,
			height_precent: 0f64,
			blobs: blobs.clone()
		};
		out.update_size_pos(dim, cur_height);
		out
	}
	fn empty() -> Content {
		Content {
			id: Uuid::new_v4(),
			top_pix: 0u32,
			top_total_pix: 0u32,
			top_precent: 0f64,
			left_pix: 0u32,
			left_precent: 0f64,
			width_pix: 0u32,
			width_precent: 0f64,
			height_pix: 0u32,
			height_precent: 0f64,
			blobs: Vec::new()
		}
	}
	fn to_image(&self) -> image::ImageBuffer<image::LumaA<u8>, Vec<u8>> {
		let mut imgbuf = image::ImageBuffer::<image::LumaA<u8>, Vec<u8>>::new(
			self.width_pix as u32, self.height_pix as u32
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
}

/// Objects holding `Heading`, `Idea`, and `Content` objects
#[derive(Clone)]
struct Chapter {
	id: Uuid,
	heading: Heading,
	sub_headings: Vec <Heading>,
	ideas: Vec <Idea>,
	content: Vec <Content>,
	writeable: bool,
	height_precent: f64,
	cur_height: i64 // use of i64 rather than u64 allows future negative starting value to componsate against mid page start
}

impl Chapter {
	/// Create a `Chapter` object
	fn new() -> Chapter {
		Chapter {
			id: Uuid::new_v4(),
			heading: Heading::new(),
			sub_headings: Vec::new(),
			ideas: Vec::new(),
			content: Vec::new(),
			writeable: false,
			height_precent: 0.0,
			cur_height: 0
		}
	}
	/// Blanks a `Chapter` object.
	/// Used to avoid scope problems that would arise due to initializing a new object in a subroutine.
	fn blank(&mut self) {
		self.id = Uuid::new_v4();
		self.heading = Heading::new();
		self.sub_headings = Vec::new();
		self.ideas = Vec::new();
		self.content = Vec::new();
		self.writeable = false;
		self.height_precent = 0.0;
		self.cur_height = 0;
	}
	/// Adds heading to table of contents
	fn add_to_toc(head: &Heading, parent: &PathBuf, pid: Uuid) {
		let mut f = File::open(parent.join("index.html"))
			.expect( // This should never happen as we have already verified the file exists
				"Output Generation: could not find created table of contents"
			);
		let mut contents = String::new();
		f.read_to_string(&mut contents)
			.expect("Output Generation: error reading created table of contents");
		contents = contents.replace(
			"<!-- NEXT CHAPTER -->",
			&format!(
				"<a href=\"{}/index.html\" class=\"head tc h1\">\n\t\t\t\t<img src=\"{}/img/t{}.png\"/>\n\t\t\t</a><br/>\n\t\t\t<!-- NEXT CHAPTER -->",
				pid.to_simple().to_string(),
				pid.to_simple().to_string(),
				head.id.to_simple().to_string()
			)
		);
		let mut file = File::create(parent.join("index.html"))
			.expect(
				"Output Generation: error recreating table of contents"
			);
		writeln!(file, "{}", contents)
			.expect(
				"Output Generation: error rewriting table of contents"
			);
	}
	/// Adds definition to review
	fn add_to_review(def: &Idea, parent: &PathBuf, pid: Uuid) {
		let mut f = File::open(parent.join("review.html"))
			.expect( // This should never happen as we have already verified the file exists
				"Output Generation: could not find created review"
			);
		let mut contents = String::new();
		f.read_to_string(&mut contents)
			.expect("Output Generation: error reading created review");
		contents = contents.replace(
			"<!-- NEXT DEFINITION -->",
			&format!(
				"<div class=\"def-parent\">\n\t\t\t\t<a href=\"{}/index.html#dh{}\">\n\t\t\t\t\t<img class=\"defi expandable\" src=\"{}/img/dh{}.png\"></img></a>\n\t\t\t\t<button class=\"expander\" onclick=\"toggle_sub(this)\">◀</button><br/>\n\t\t\t\t<img class=\"defi\" src=\"{}/img/dc{}.png\" style=\"display:none;\"/>\n\t\t\t</div>\n\t\t\t<!-- NEXT DEFINITION -->",
				pid.to_simple().to_string(),
				def.id.to_simple().to_string(),
				pid.to_simple().to_string(),
				def.id.to_simple().to_string(),
				pid.to_simple().to_string(),
				def.id.to_simple().to_string()
			)
		);
		let mut file = File::create(parent.join("review.html"))
			.expect(
				"Output Generation: error recreating review"
			);
		writeln!(file, "{}", contents)
			.expect(
				"Output Generation: error rewriting review"
			);
	}
	/// Writes a chapter object out
	fn add_chapter(&mut self) {
		fn assemble_path() -> PathBuf {
			let dir: PathBuf;
			match dirs_next::home_dir() {
				Some(path) => dir = path,
				None => panic!(
					"Output Generation: system lacks valid home directory"
				),
			}
			dir.as_path().join(Path::new(OUT_PATH))
		}
		fn setup_dirs(comp_out: &PathBuf) {
			fs::create_dir_all(comp_out).expect(
				"Output Generation: error creating root path"
			);
			let mut file = File::create(
				comp_out.join("index.html")
			).expect(
				"Output Generation: error creating root index"
			);
			writeln!(file, include_str!("templates/table/index.html"))
				.expect(
					"Output Generation: error writing to root index"
				);
			file = File::create(
				comp_out.join("review.html")
			).expect(
				"Output Generation: error creating review"
			);
			writeln!(file, include_str!("templates/table/review.html"))
				.expect(
					"Output Generation: error writing to review"
				);
			file = File::create(
				comp_out.join("static.css")
			).expect(
				"Output Generation: error creating root style"
			);
			writeln!(file, "{}", include_str!("templates/table/static.css"))
				.expect(
					"Output Generation: error writing to root style"
				);
			file = File::create(
				comp_out.join("hue.svg")
			).expect(
				"Output Generation: error creating root color profile"
			);
			writeln!(file, include_str!("templates/table/hue.svg")).expect(
				"Output Generation: error writing to root color profile"
			);
			file = File::create(
				comp_out.join("fullscreen-op.svg")
			).expect(
				"Output Generation: error creating root fullscreen"
			);
			writeln!(file, include_str!("templates/table/fullscreen-op.svg"))
				.expect(
					"Output Generation: error writing to root fullscreen"
				);
			file = File::create(
				comp_out.join("util.js")
			).expect(
				"Output Generation: error creating root utilities"
			);
			writeln!(file, "{}", include_str!("templates/table/util.js"))
				.expect(
					"Output Generation: error writing to root utilities"
				);
		}
		let comp_out = assemble_path();
		if !Path::new(&(
			comp_out.join("index.html")
		)).exists() {
			setup_dirs(&comp_out);
		}
		let ch_path = comp_out.join(&self.id.to_simple().to_string());
		fs::create_dir(&ch_path)
			.expect("Output Generation: error creating chapter path");
		fs::create_dir(
			ch_path.join("img")
		).expect("Output Generation: error creating chapter image path");
		let mut out = String::from(
			include_str!("template_fragments/chapter/index.html0")
		);
		let mut gencss = String::from(
			include_str!("template_fragments/chapter/gen.css1")
		);
		out += &(
			"<img class=\"head h1\" id=\"".to_string()+
			&self.heading.id.to_simple().to_string()+
			&"\" src=\"img/t".to_string()+
			&self.heading.id.to_simple().to_string()+
			&".png\"/>".to_string()
		);
		self.heading.subject.update_top(&self.cur_height);
		gencss += &(
			"#t".to_string()+
			&self.heading.id.to_simple().to_string()+
			&"{\n\ttop:".to_string()+
			&(self.heading.subject.top_precent*(100 as f64)).to_string()+
			&"%;\n\tleft:".to_string()+
			&(self.heading.subject.left_precent*(100 as f64)).to_string()+
			&"%;\n\twidth:".to_string()+
			&(self.heading.subject.width_precent*(100 as f64)).to_string()+
			&"%;\nposition:absolute;\n}\n".to_string()
		);
		Chapter::add_to_toc(
			&self.heading,
			&comp_out,
			self.id
		);
		out += include_str!("template_fragments/chapter/index.html1");
		let _ = ImageLumaA8(
			self.heading.subject.to_image()
		).save(
			ch_path.join(
				"img/t".to_string()+
				&self.heading.id.to_simple().to_string()+
				&".png".to_string()
			)
		);
		for head in &mut self.sub_headings {
			let _ = ImageLumaA8(
				head.subject.to_image()
			).save(
				ch_path.join(
					"img/h".to_string()+
					&head.id.to_simple().to_string()+
					&".png".to_string()
				)
			);
			out += &(
				if head.number == 2 {
					"\t\t\t<img class=\"head h2\" id=\"h"
				} else {
					"\t\t\t<img class=\"head h3\" id=\"h"
				}.to_string()+
				&head.id.to_simple().to_string()+
				&"\"".to_string()+
				&" src=\"img/h".to_string()+
				&head.id.to_simple().to_string()+
				&".png\"/>\n".to_string()
			);
			head.subject.update_top(&self.cur_height);
			gencss += &(
				"#h".to_string()+
				&head.id.to_simple().to_string()+
				&"{\n\ttop:".to_string()+
				&(head.subject.top_precent*(100 as f64)).to_string()+
				&"%;\n\tleft:".to_string()+
				&(head.subject.left_precent*(100 as f64)).to_string()+
				&"%;\n\twidth:".to_string()+
				&(head.subject.width_precent*(100 as f64)).to_string()+
				&"%;\nposition:absolute;\n}\n".to_string()
			);
		}
		for cont in &mut self.content {
			let _ = ImageLumaA8(
				cont.to_image()
			).save(
				ch_path.join(
					"img/c".to_string()+
					&cont.id.to_simple().to_string()+
					&".png".to_string()
				)
			);
			out += &(
				"\t\t\t<img class=\"cont\" id=\"c".to_string()+
				&cont.id.to_simple().to_string()+
				&"\"".to_string()+
				&" src=\"img/c".to_string()+
				&cont.id.to_simple().to_string()+
				&".png\"/>\n".to_string()
			);
			cont.update_top(&self.cur_height);
			gencss += &(
				"#c".to_string()+
				&cont.id.to_simple().to_string()+
				&"{\n\ttop:".to_string()+
				&(cont.top_precent*(100 as f64)).to_string()+
				&"%;\n\tleft:".to_string()+
				&(cont.left_precent*(100 as f64)).to_string()+
				&"%;\n\twidth:".to_string()+
				&(cont.width_precent*(100 as f64)).to_string()+
				&"%;\nposition:absolute;\n}\n".to_string()
			);
		}
		for idea in &mut self.ideas {
			let _ = ImageLumaA8(
				idea.subject.to_image()
			).save(
				ch_path.join(
					"img/dh".to_string()+
					&idea.id.to_simple().to_string()+
					&".png".to_string()
				)
			);
			out += &(
				"\t\t\t<img class=\"defi h2\" id=\"dh".to_string()+
				&idea.id.to_simple().to_string()+
				&"\"".to_string()+
				&" src=\"img/dh".to_string()+
				&idea.id.to_simple().to_string()+
				&".png\"/>\n".to_string()
			);
			idea.subject.update_top(&self.cur_height);
			gencss += &(
				"#dh".to_string()+
				&idea.id.to_simple().to_string()+
				&"{\n\ttop:".to_string()+
				&(idea.subject.top_precent*(100 as f64)).to_string()+
				&"%;\n\tleft:".to_string()+
				&(idea.subject.left_precent*(100 as f64)).to_string()+
				&"%;\n\twidth:".to_string()+
				&(idea.subject.width_precent*(100 as f64)).to_string()+
				&"%;\nposition:absolute;\n}\n".to_string()
			);
			let _ = ImageLumaA8(
				idea.extension.to_image()
			).save(
				ch_path.join(
					"img/dc".to_string()+
					&idea.id.to_simple().to_string()+
					&".png".to_string()
				)
			);
			out += &(
				"\t\t\t<img class=\"defi\" id=\"dc".to_string()+
				&idea.id.to_simple().to_string()+
				&"\"".to_string()+
				&" src=\"img/dc".to_string()+
				&idea.id.to_simple().to_string()+
				&".png\"/>\n".to_string()
			);
			idea.extension.update_top(&self.cur_height);
			gencss += &(
				"#dc".to_string()+
				&idea.id.to_simple().to_string()+
				&"{\n\ttop:".to_string()+
				&(idea.extension.top_precent*(100 as f64)).to_string()+
				&"%;\n\tleft:".to_string()+
				&(idea.extension.left_precent*(100 as f64)).to_string()+
				&"%;\n\twidth:".to_string()+
				&(idea.extension.width_precent*(100 as f64)).to_string()+
				&"%;\nposition:absolute;\n}\n".to_string()
			);
			Chapter::add_to_review(
				&idea,
				&comp_out,
				self.id
			);
		}
		out += include_str!("template_fragments/chapter/index.html2");
		gencss =
			"\tpadding-bottom:".to_string()+
			&(self.height_precent*(100 as f64)).to_string()+
			&"%;\n".to_string()+
			&gencss
		;
		gencss = String::from(
			include_str!("template_fragments/chapter/gen.css0")
		) + &gencss;
		let ref mut file = File::create(
			ch_path.join("index.html")
		).unwrap();
		writeln!(file, "{}", out)
			.expect("Chapter output: error creating index");
		let ref mut file_gencss = File::create(
			ch_path.join("gen.css")
		).unwrap();
		writeln!(file_gencss, "{}", gencss)
			.expect("Chapter output: error creating index");
		let ref mut file_scss = File::create(
			ch_path.join("static.css")
		).unwrap();
		writeln!(
			file_scss,
			"{}",
			include_str!("templates/chapter/static.css")
		).expect("Chapter output: error creating static CSS");
		let ref mut file_fscr = File::create(
			ch_path.join("fullscreen-op.svg")
		).unwrap();
		writeln!(
			file_fscr,
			"{}",
			include_str!("templates/chapter/fullscreen-op.svg")
		).expect("Chapter output: error creating fullscreen");
		let ref mut file_hue = File::create(
			ch_path.join("hue.svg")
		).unwrap();
		writeln!(
			file_hue,
			"{}",
			include_str!("templates/chapter/hue.svg")
		).expect("Chapter output: error creating hue");
		let ref mut file_util = File::create(
			ch_path.join("util.js")
		).unwrap();
		writeln!(
			file_util,
			"{}",
			include_str!("templates/chapter/util.js")
		).expect("Chapter output: error creating util");
	}
}

/// Get a vector of image paths to import from a user.
fn get_images() -> Vec <String> {
	/// Get vector containing already imported images from Imported.
	fn get_imported_images() -> Vec <String> {
		if Path::new(IMPORTED).exists() {
			let mut list: Vec <String> = Vec::new();
			let f = (File::open(IMPORTED)).unwrap();
			let file = BufReader::new(&f);
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
	fn parse_input(
		uin: String,
		mpaths: Vec <String>,
		new: &mut Vec <String>
	) -> Vec <String> {
		let mut selected: Vec <String> = Vec::new();
		let stringified: Vec <String> = uin.split(' ')
			.map(|x| x.to_string()).collect();
		for sel in stringified {
			if sel == "+" {
				selected.append(new);
			} else if sel.to_string().contains("-") {
				let numbers: Vec <String> = uin.split('-')
					.map(|x| x.to_string()).collect();
				let start = numbers[0].parse::<usize>().unwrap();
				let end = numbers[1].parse::<usize>().unwrap();
				for i in start..(end+1) {
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
	let imported: Vec <String> = get_imported_images();
	for p in paths {
		let path = p.unwrap().path();
		if !(path.extension() == None) {
			//The next line needs to be cleaned up.  It is written like this to appease the borrow checker
			if
				path.extension().unwrap() == "png" ||
				path.extension().unwrap() == "jpg" ||
				path.extension().unwrap() == "bpm" ||
				path.extension().unwrap() == "gif"
			{
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
	uin = uin.trim().to_string();
	parse_input(uin, mpaths, &mut new)
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
		chapter.content.push(Content::new(clump.blobs, page.dimensions, chapter.cur_height));
	} else {
		*destroyed += clump.blobs.len();
	}
}

/// Add definition objects to chapter or destroy them because they lack a chapter.
fn add_definition(
	clump: Clump,
	page: &Page,
	chapter: &mut Chapter,
	destroyed: &mut usize,
	started: bool
) {
	fn is_underlined(blob: ImgBlob, line: &ImgBlob) -> bool {
		((blob.top_left[0] as i64 - line.top_left[0] as i64) > -50) && // Make -50 proportional
		((line.bottom_right[0] as i64 - blob.bottom_right[0] as i64) > -50) &&
		(blob.bottom_right[1] < line.bottom_right[1])
	}
	if started {
		let mut line: ImgBlob = ImgBlob::new();
		let mut name: Vec<ImgBlob> = Vec::new();
		let mut cont: Vec<ImgBlob> = Vec::new();
		for i in 0..clump.blobs.len() {
			if clump.blobs[i].blob_type == 1 {
				line = clump.blobs[i].clone();
				break;
			}
		}
		if line.bottom_right[1] == 0 {
			*destroyed += clump.blobs.len();
			return;
		}
		for i in 0..clump.blobs.len() {
			if clump.blobs[i] != line {
				if is_underlined(clump.blobs[i].clone(), &line) {
					name.push(clump.blobs[i].clone());
				} else {
					cont.push(clump.blobs[i].clone());
				}
			}
		}
		let mut idea = Idea::new();
		if name.len() == 0 {
			*destroyed += clump.blobs.len();
			return;
		}
		idea.subject = Content::new(name, page.dimensions, chapter.cur_height);
		if cont.len() > 0 {
			idea.extension = Content::new(cont, page.dimensions, chapter.cur_height);
			idea.update_size_pos(page.dimensions); // probably unused
		}
		chapter.ideas.push(idea);
	} else {
		*destroyed += clump.blobs.len();
	}
}

trait Sub {
	fn sub(self, other: [usize;2]) -> [usize; 2];
}

///Difference between 2D usize array
impl Sub for [usize; 2] {
	fn sub(self, other: [usize;2]) -> [usize; 2] {
		if (self[0] > other[0]) && (self[1] > other[1]) {
			[self[0]-other[0], self[1]-other[1]]
		} else if (self[0] > other[0]) && (self[1] < other[1]) {
			[self[0]-other[0], other[1]-self[1]]
		} else if (self[0] < other[0]) && (self[1] > other[1]) {
			[other[0]-self[0], self[1]-other[1]]
		} else{
			[other[0]-self[0], other[1]-self[1]]
		}
	}
}

trait Filter {
	fn filter(&mut self);
}

impl Filter for image::RgbImage {
	fn filter(&mut self) {
		for (_, _, pixel) in self.enumerate_pixels_mut() {
			if pixel[0] > pixel[1] && pixel[0] > pixel[2] {
				let avg = ((pixel[1] as u16+pixel[2] as u16)/2u16) as u8;
				if pixel[0] - avg > 70 {
					pixel[0] = 255u8;
					pixel[1] = 0u8;
					pixel[2] = 0u8;
				} else {
					pixel[0] = 255u8;
					pixel[1] = 255u8;
					pixel[2] = 255u8;
				}
			} else if pixel[1] > pixel[0] && pixel[1] > pixel[2] {
				let avg = ((pixel[0] as u16+pixel[2] as u16)/2u16) as u8;
				if pixel[1] - avg > 20 {
					pixel[0] = 0u8;
					pixel[1] = 255u8;
					pixel[2] = 0u8;
				} else {
					pixel[0] = 255u8;
					pixel[1] = 255u8;
					pixel[2] = 255u8;
				}
			} else if pixel[2] > pixel[0] && pixel[2] > pixel[1] {
				let avg = ((pixel[0] as u16+pixel[1] as u16)/2u16) as u8;
				let avg2 = ((pixel[0] as u16+pixel[1] as u16+pixel[2] as u16)/3u16) as u8;
				if avg2 < 155 && pixel[2] - avg > 60 {
					pixel[0] = 0u8;
					pixel[1] = 0u8;
					pixel[2] = 255u8;
				} else {
					pixel[0] = 255u8;
					pixel[1] = 255u8;
					pixel[2] = 255u8;
				}
			} else {
				pixel[0] = 255u8;
				pixel[1] = 255u8;
				pixel[2] = 255u8;
			}
		}
	}
}

/// Create new chapter, add heading objects to chapter, or destroy them because they lack a chapter.
fn add_heading(
	clump: Clump,
	page: &Page,
	chapter: &mut Chapter,
	destroyed: &mut usize,
	created: &mut usize,
	started: &mut bool
) {
	let mut i: usize = 0;
	let mut linemode: i8 = -1;
	let mut past = [0usize; 2];
	let mut head: Heading = Heading::new();
	while i < clump.blobs.len() {
		let blob = clump.blobs[i].clone();
		if blob.blob_type == 1 {
			// TODO: reduce cyclomatic complexity
			if linemode==1 {
				// 1/17 of width and 1/22 height off acceptable
				let diff = blob.top_left.sub(past);
				if
					(diff[0] as f32) < 1f32/4f32*(page.dimensions[0] as f32) &&
					(diff[1] as f32) < 1f32/20f32*(page.dimensions[1] as f32)
				{
					if *started {
						chapter.cur_height += page.dimensions[1] as i64;
						(chapter.clone()).add_chapter();
						*created += 1;
					}
					chapter.blank();
					head.number = 1;
					head.subject.update_size_pos(page.dimensions, chapter.cur_height);
					chapter.heading = head.clone();
					head = Heading::new();
					chapter.height_precent +=
						(page.dimensions[1] as f64)/
						(page.dimensions[0] as f64);
					*started = true;
					linemode = -1;
				} else {
					*destroyed += 1;
				}
			} else if linemode == 0 {
				head.number = 2;
				linemode = 1;
				past = blob.top_left;
			} else {
				*destroyed += 1;
			}
		} else {
			if linemode < 1 {
				if linemode == -1 {
					linemode = 0;
				}
				head.subject.blobs.push(blob);
			} else {
				assert!(
					head.number == 2,
					"Found heading.number of {}. Expected 2",
					head.number
				);
				if *started {
					head.subject.update_size_pos(page.dimensions, chapter.cur_height);
					chapter.sub_headings.push(head.clone());
					linemode = 0;
					head = Heading::new();
					head.subject.blobs.push(blob);
				} else {
					*destroyed += head.subject.blobs.len() + 1;
				}
			}
		}
		i += 1;
	}
	if linemode != -1 {
		assert!(
			head.number != 1,
			"Found heading.number of 1. Expected 2 or 3"
		);
		if *started {
			head.subject.update_size_pos(page.dimensions, chapter.cur_height);
			chapter.sub_headings.push(head);
		} else {
			*destroyed += head.subject.blobs.len();
		}
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
		pages.push(Page::from_path(img.clone()));
		if !fs::metadata(IMPORTED).is_ok() {
			File::create(IMPORTED).unwrap();
		}
		let mut file = OpenOptions::new()
			.write(true)
			.append(true)
			.open(IMPORTED)
			.unwrap();
		match writeln!(file, "{}", img) {
			Ok(_) => (),
			Err(_) => println!("Warning: could not create .imported file")
		}
	}
	print!("\r◑: Dividing by chapter");
	std::io::stdout().flush().ok().expect("Could not flush STDOUT!");
	let mut chapter: Chapter = Chapter::new();
	let mut started = false;
	let mut created_chapters = 0;
	let mut destroyed: usize = 0;
	for p in pages {
		chapter.height_precent +=
			(p.dimensions[1] as f64)/(p.dimensions[0] as f64);
		let mut i: usize = 0;
		while i < p.clumps.len() {
			match p.clumps[i].ctype {
				RED   => add_heading( // Heading(s) of some type
					p.clumps[i].clone(),
					&p,
					&mut chapter,
					&mut destroyed,
					&mut created_chapters,
					&mut started
				),
				GREEN => add_definition( // Defintions(s) of some type
					p.clumps[i].clone(),
					&p,
					&mut chapter,
					&mut destroyed,
					started
				),
				BLUE  => add_content( // Content
					p.clumps[i].clone(),
					&p,
					&mut chapter,
					&mut destroyed,
					started
				),
				_ => panic!("Invalid Content")
			};
			i += 1;
		}
		chapter.cur_height += p.dimensions[1] as i64;
	}
	if chapter.heading.subject.blobs.len() > 0 {
		chapter.add_chapter();
		created_chapters += 1;
	}
	print!("\r◕: Writing            ");
	std::io::stdout().flush().ok().expect("Could not flush STDOUT!");
	println!("\r●: Done               ");
	println!(
		"{} chapters added.  {} orphaned objects destroyed",
		created_chapters,
		destroyed
	);
}
