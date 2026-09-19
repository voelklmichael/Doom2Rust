//! Safe view over a Doom "patch" lump: an 8-byte header (width, height,
//! left/top offset, all little-endian i16), `width` little-endian i32 column
//! offsets, then the columns themselves as runs of "posts"
//! (`topdelta, length, pad, pixels[length], pad`) terminated by `topdelta == 0xff`.
use alloc::rc::Rc;

#[derive(Clone)]
pub struct Patch {
    data: Rc<[u8]>,
}

pub struct Post<'a> {
    pub topdelta: usize,
    pub pixels: &'a [u8],
}

pub struct Posts<'a> {
    data: &'a [u8],
    offset: usize,
}

impl Patch {
    pub fn new(data: Rc<[u8]>) -> Self {
        Patch { data }
    }

    fn i16_at(&self, offset: usize) -> i32 {
        i16::from_le_bytes([self.data[offset], self.data[offset + 1]]) as i32
    }

    pub fn width(&self) -> i32 {
        self.i16_at(0)
    }

    pub fn height(&self) -> i32 {
        self.i16_at(2)
    }

    pub fn leftoffset(&self) -> i32 {
        self.i16_at(4)
    }

    pub fn topoffset(&self) -> i32 {
        self.i16_at(6)
    }

    pub fn columnofs(&self, column: i32) -> usize {
        let o = 8 + 4 * column as usize;
        i32::from_le_bytes([
            self.data[o],
            self.data[o + 1],
            self.data[o + 2],
            self.data[o + 3],
        ]) as usize
    }

    pub fn posts(&self, column: i32) -> Posts<'_> {
        Posts {
            data: &self.data,
            offset: self.columnofs(column),
        }
    }
}

impl<'a> Iterator for Posts<'a> {
    type Item = Post<'a>;

    fn next(&mut self) -> Option<Post<'a>> {
        let topdelta = self.data[self.offset];
        if topdelta == 0xff {
            return None;
        }
        let length = self.data[self.offset + 1] as usize;
        let start = self.offset + 3;
        let pixels = &self.data[start..start + length];
        self.offset += length + 4;
        Some(Post {
            topdelta: topdelta as usize,
            pixels,
        })
    }
}
