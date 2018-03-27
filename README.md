# note-digest [![Build Status](https://travis-ci.org/michardy/note-digest.svg?branch=master)](https://travis-ci.org/michardy/note-digest) [![Build status](https://ci.appveyor.com/api/projects/status/6jxdfvg7hg8wg89f?svg=true)](https://ci.appveyor.com/project/michardy/note-digest) [![dependency status](https://deps.rs/repo/github/michardy/note-digest/status.svg)](https://deps.rs/repo/github/michardy/note-digest)
A system for taking hand written color coded notes and converting them into an organized digital notebook.  The system can automaticaly split notes into chapters, identify headings and identify important definitions and ideas.  The headings and definitions are added to a table of contents.  

# Warning:
I am still learning rust. This code is probably a little questionable.  

# Completion:
- [x] File selection
  - [x] Displays only supported files
  - [x] Allows for the selection of a of multiple images
  - [x] New image identification
    - [x] Allows you to select all new images
    - [x] Records images after use
- [x] Image processing
  - [x] Seperating headings notes and definitions
  - [x] Thresholding
  - [x] Identifying objects with floodfill
  - [x] Line detection
- [x] Object lumping
- [x] Parsing lumps
  - [x] Heading identification
    - [x] Heading 1
      - [x] Double line detection
      - [x] Title text identification
    - [x] Heading 2
      - [x] Single line detection
      - [x] Title text identification
    - [x] Heading 3
      - [x] Title text identification
  - [x] Definition identification
    - [x] Definition header identification
  - [x] Splitting chapters by occurences of heading 1
- [ ] Exporting chapters
  - [ ] Adding to table of contents
    - [X] Adding chapter titles
    - [ ] Adding definitions
      - [ ] Enable showing/hiding defintions
      - [ ] Enable expanding defintions
  - [X] Saving images
  - [X] Positioning Images
- [ ] Organized code
