use std::io::BufRead;

pub struct Input<B> {
    inner: B,
    buffer: String,
}

impl<B: BufRead> Input<B> {
    pub fn new(inner: B) -> Self {
        Self {
            inner,
            buffer: String::new(),
        }
    }
    pub fn line(&mut self) -> Line {
        self.buffer.clear();
        self.inner.read_line(&mut self.buffer).unwrap();
        Line {
            split: self.buffer.split_whitespace(),
        }
    }
}

pub struct Line<'a> {
    split: std::str::SplitWhitespace<'a>,
}

impl<'a> Line<'a> {
    pub fn next(&mut self) -> Option<String> {
        self.split.next().map(|x| x.to_string())
    }
}

// fn main() -> Result<(), Box<dyn Error>> {
//     let input = stdin();
//     let mut input = Input::new(BufReader::new(input.lock()));

//     let test_cases = input.line().next();

//     for _ in 0..test_cases {
//         let (n, k) = input.line().pair();

//         let mut best = 0;
//         for _ in 0..n {
//             let mut line = input.line();
//             let sum = (0..k).map(|_| line.next()).sum();
//             if sum > best {
//                 best = sum;
//             }
//         }
//         println!("{}", best);
//     }
//     Ok(())
// }
