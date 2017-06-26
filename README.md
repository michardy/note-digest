# note-digest
A system for taking hand written color coded notes and converting them into an organized digital notebook.  The system can automaticaly split notes into chapters, identify headings and identify important definitions and ideas.  The headings and definitions are added to a table of contents.  

# Warning:
I am still learning rust. This code is probably a little questionable.  

# Completion:
- [ ] File selection
  - [x] Displays only supported files
  - [x] Allows for the selection of a of multiple images
  - [ ] New image identification
    - [x] Allows you to select all new images
    - [ ] Records images after use
- [x] Image processing
  - [x] Seperating headings notes and definitions
  - [x] Thresholding
  - [x] Identifying objects with floodfill
  - [x] Line detection
- [ ] Parsing objects
  - [ ] Heading identification
    - [x] Heading 1
      - [x] Double line detection
      - [x] Title text identification
    - [ ] Heading 2
      - [x] Single line detection
      - [x] Title text identification
    - [ ] Heading 3
      - [ ] Title text identification
  - [ ] Definition identification
    - [ ] Definition header identification
  - [x] Splitting chapters by occurences of heading 1
 - [ ] Exporting chapters
  - [ ] Adding to table of contents
    - [ ] Adding chapter titles
    - [ ] Adding definitions
      - [ ] Enable showing/hiding defintions
      - [ ] Enable expanding defintions
  - [ ] Saving images
  - [ ] Positioning Images
- [ ] Organized code
