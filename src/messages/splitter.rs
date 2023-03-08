use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read},
    path::Path,
};

use crate::Signal;

use super::signal::{MAX_SIGNAL_BUF_SIZE, MAX_FILE_PART_DATA_SIZE};

pub struct Splitter<T: BufRead> {
    /// indentifier for the file partitioning operation
    filename: String,
    /// index pointing to the start of the next split
    pos: u64,
    /// underlying reader for the splitting operation
    reader: T,
}

impl<T: BufRead> Iterator for Splitter<T> {
    type Item = Signal;

    fn next(&mut self) -> Option<Self::Item> {
        // obtain a buffer from the reader
        let data = self.reader.fill_buf().map_or_else(|_| None, Some)?;

        // map to the max size
        let slice = match data.len() {
            0 => return None,
            1..=MAX_SIGNAL_BUF_SIZE => data,
            _ => &data[0..MAX_FILE_PART_DATA_SIZE],
        };

        let op_start = self.pos;
        let slice_len = slice.len();
        let vec = Vec::from(slice);

        // consume buffer from the reader and advance splitter internal pos
        self.reader.consume(slice_len);
        self.pos += slice_len as u64;
        Some(Signal::FilePart(self.filename.clone(), op_start, vec))
    }
}

impl<T: BufRead> Splitter<T> {
    pub fn read_file<P: AsRef<Path>>(path: P) -> std::io::Result<Splitter<BufReader<File>>> {
        let filename = path.as_ref().to_string_lossy().into_owned();

        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);

        Ok(Splitter {
            filename,
            pos: 0,
            reader,
        })
    }

    pub fn read_buf(reader: T, ident: String) -> Splitter<T> {
        Splitter {
            filename: ident,
            pos: 0,
            reader,
        }
    }
}

impl<T: Read> Splitter<BufReader<T>> {
    pub fn read(reader: T, ident: String) -> Splitter<BufReader<T>> {
        Splitter {
            filename: ident,
            pos: 0,
            reader: BufReader::with_capacity(MAX_FILE_PART_DATA_SIZE, reader),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Seek, Write};

    use crate::{
        messages::{
            signal::{MAX_FILE_PART_DATA_SIZE, MAX_SIGNAL_BUF_SIZE},
            Drain,
        },
        Signal,
    };

    use super::Splitter;

    #[test]
    fn split_read_vec() {
        let data = vec![42; 10000];
        let splitter = Splitter::read(&*data, "ident".to_string());

        let mut agg = Vec::new();
        for msg in splitter {
            assert!(matches!(msg, Signal::FilePart(_, _, _)));
            let Signal::FilePart(ident, pos, buf) = msg.clone() else {panic!("cant");};

            assert_eq!(&ident, "ident");
            assert_eq!(pos, agg.len() as u64);
            assert!(buf.len() <= MAX_FILE_PART_DATA_SIZE);
            let mut writer = Vec::new();
            let mut ser_buf = [0u8; MAX_SIGNAL_BUF_SIZE];
            let ser = msg.serialize_to(&mut writer, &mut ser_buf);
            assert!(ser.is_ok(), "serialization failure");

            agg.extend_from_slice(&buf);
        }

        assert_eq!(agg, data);
    }

    #[test]
    fn split_read_tmpfile() {
        let data = vec![42u8; 10000];
        let mut file = tempfile::tempfile().expect("cannot create tempfile");
        file.write_all(&data).expect("could not write to tempfile");
        file.rewind().expect("could not rewind file");

        let splitter = Splitter::read(file, "ident".to_string());

        let mut agg = Vec::new();
        for msg in splitter {
            assert!(matches!(msg, Signal::FilePart(_, _, _)));
            let Signal::FilePart(ident, pos, buf) = msg.clone() else {panic!("cant");};

            assert_eq!(&ident, "ident");
            assert_eq!(pos, agg.len() as u64);
            assert!(buf.len() <= MAX_FILE_PART_DATA_SIZE);
            let mut writer = Vec::new();
            let mut ser_buf = [0u8; MAX_SIGNAL_BUF_SIZE];
            let ser = msg.serialize_to(&mut writer, &mut ser_buf);
            assert!(ser.is_ok(), "serialization failure");

            agg.extend_from_slice(&buf);
        }

        assert_eq!(agg, data);
    }
}
