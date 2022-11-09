use {
    serde::{Deserialize, Serialize},
    std::{
        fmt,
        io::{self, BufRead, Write},
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Response(Response),
    Notification(Notification),
}

impl From<Request> for Message {
    fn from(request: Request) -> Message {
        Message::Request(request)
    }
}

impl From<Response> for Message {
    fn from(response: Response) -> Message {
        Message::Response(response)
    }
}

impl From<Notification> for Message {
    fn from(notification: Notification) -> Message {
        Message::Notification(notification)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(untagged)]
pub enum RequestId {
    Num(i32),
    Str(String),
}

impl From<i32> for RequestId {
    fn from(id: i32) -> RequestId {
        RequestId::Num(id)
    }
}

impl From<String> for RequestId {
    fn from(id: String) -> RequestId {
        RequestId::Str(id)
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Num(it) => fmt::Display::fmt(it, f),
            // Use debug here, to make it clear that `92` and `"92"` are
            // different, and to reduce WTF factor if the sever uses `" "` as an
            // ID.
            Self::Str(it) => fmt::Debug::fmt(it, f),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub id: RequestId,
    pub method: String,
    #[serde(default = "serde_json::Value::default")]
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    // JSON RPC allows this to be null if it was impossible
    // to decode the request's id. Ignore this special case
    // and just die horribly.
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub method: String,
    #[serde(default = "serde_json::Value::default")]
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub params: serde_json::Value,
}

impl Message {
    pub fn read(r: &mut impl BufRead) -> io::Result<Option<Message>> {
        let text = match read_msg_text(r)? {
            None => return Ok(None),
            Some(text) => text,
        };
        let msg = serde_json::from_str(&text)?;
        Ok(Some(msg))
    }

    pub fn write(self, w: &mut impl Write) -> io::Result<()> {
        #[derive(Serialize)]
        struct JsonRpc {
            jsonrpc: &'static str,
            #[serde(flatten)]
            msg: Message,
        }
        let text = serde_json::to_string(&JsonRpc {
            jsonrpc: "2.0",
            msg: self,
        })?;

        write_msg_text(w, &text)
    }
}

impl Request {
    pub fn new<P: Serialize>(id: RequestId, method: String, params: P) -> Request {
        Request {
            id,
            method,
            params: serde_json::to_value(params).unwrap(),
        }
    }
}

impl Notification {
    pub fn new(method: String, params: impl Serialize) -> Notification {
        Notification {
            method,
            params: serde_json::to_value(params).unwrap(),
        }
    }
    pub(crate) fn is_exit(&self) -> bool {
        self.method == "exit"
    }
}

fn read_msg_text(inp: &mut dyn BufRead) -> io::Result<Option<String>> {
    fn invalid_data(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, error)
    }
    macro_rules! invalid_data {
        ($($tt:tt)*) => (invalid_data(format!($($tt)*)))
    }

    let mut size = None;
    let mut buf = String::new();
    loop {
        buf.clear();
        if inp.read_line(&mut buf)? == 0 {
            return Ok(None);
        }
        if !buf.ends_with("\r\n") {
            return Err(invalid_data!("malformed header: {:?}", buf));
        }
        let buf = &buf[..buf.len() - 2];
        if buf.is_empty() {
            break;
        }
        let mut parts = buf.splitn(2, ": ");
        let header_name = parts.next().unwrap();
        let header_value = parts
            .next()
            .ok_or_else(|| invalid_data!("malformed header: {:?}", buf))?;
        if header_name == "Content-Length" {
            size = Some(header_value.parse::<usize>().map_err(invalid_data)?);
        }
    }
    let size: usize = size.ok_or_else(|| invalid_data!("no Content-Length"))?;
    let mut buf = buf.into_bytes();
    buf.resize(size, 0);
    inp.read_exact(&mut buf)?;
    let buf = String::from_utf8(buf).map_err(invalid_data)?;
    Ok(Some(buf))
}

fn write_msg_text(out: &mut dyn Write, msg: &str) -> io::Result<()> {
    write!(out, "Content-Length: {}\r\n\r\n", msg.len())?;
    out.write_all(msg.as_bytes())?;
    out.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Message, Notification, Request, RequestId};

    #[test]
    fn shutdown_with_explicit_null() {
        let text = "{\"jsonrpc\": \"2.0\",\"id\": 3,\"method\": \"shutdown\", \"params\": null }";
        let msg: Message = serde_json::from_str(text).unwrap();

        assert!(
            matches!(msg, Message::Request(req) if req.id == 3.into() && req.method == "shutdown")
        );
    }

    #[test]
    fn shutdown_with_no_params() {
        let text = "{\"jsonrpc\": \"2.0\",\"id\": 3,\"method\": \"shutdown\"}";
        let msg: Message = serde_json::from_str(text).unwrap();

        assert!(
            matches!(msg, Message::Request(req) if req.id == 3.into() && req.method == "shutdown")
        );
    }

    #[test]
    fn notification_with_explicit_null() {
        let text = "{\"jsonrpc\": \"2.0\",\"method\": \"exit\", \"params\": null }";
        let msg: Message = serde_json::from_str(text).unwrap();

        assert!(matches!(msg, Message::Notification(not) if not.method == "exit"));
    }

    #[test]
    fn notification_with_no_params() {
        let text = "{\"jsonrpc\": \"2.0\",\"method\": \"exit\"}";
        let msg: Message = serde_json::from_str(text).unwrap();

        assert!(matches!(msg, Message::Notification(not) if not.method == "exit"));
    }

    #[test]
    fn serialize_request_with_null_params() {
        let msg = Message::Request(Request {
            id: RequestId::from(3),
            method: "shutdown".into(),
            params: serde_json::Value::Null,
        });
        let serialized = serde_json::to_string(&msg).unwrap();

        assert_eq!("{\"id\":3,\"method\":\"shutdown\"}", serialized);
    }

    #[test]
    fn serialize_notification_with_null_params() {
        let msg = Message::Notification(Notification {
            method: "exit".into(),
            params: serde_json::Value::Null,
        });
        let serialized = serde_json::to_string(&msg).unwrap();

        assert_eq!("{\"method\":\"exit\"}", serialized);
    }
}
