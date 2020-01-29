use std::io::Error;
use std::fmt;
use std::io::ErrorKind;

pub trait Message {
    fn new(buf: Vec<u8>) -> Result<Self, Error> where Self: Sized;
    fn bytes(&self) -> &[u8];
}

#[derive(Clone)]
pub enum MessageType {
    Test(TestMessage),
}

#[derive(Clone)]
pub struct TestMessage {
    data: String
}

impl TestMessage {

    pub fn new(data: String) -> Self {
        Self {data}
    }

}

impl Message for TestMessage {

    fn new(buf: Vec<u8>) -> Result<Self, Error> {

        let data = String::from_utf8(buf);

        match data {
            Ok(x) => Ok(Self {data: x}),
            Err(_) => Err(Error::new(ErrorKind::InvalidData,"Invalid data"))
        }

    }

    fn bytes(&self) -> &[u8] {
        self.data.as_bytes()
    }

}

impl fmt::Display for TestMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}