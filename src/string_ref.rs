pub enum StringRef {
    Owned(String),
    Static(&'static str),
}

impl StringRef {
    pub fn as_str(&self) -> &str {
        match self {
            StringRef::Owned(s) => s.as_str(),
            StringRef::Static(s) => s,
        }
    }
}

impl From<String> for StringRef {
    fn from(s: String) -> Self {
        StringRef::Owned(s)
    }
}

impl From<&'static str> for StringRef {
    fn from(s: &'static str) -> Self {
        StringRef::Static(s)
    }
}

impl From<bluer::Error> for StringRef {
    fn from(err: bluer::Error) -> Self {
        StringRef::Owned(err.to_string())
    }
}

impl From<anyhow::Error> for StringRef {
    fn from(err: anyhow::Error) -> Self {
        StringRef::Owned(err.to_string())
    }
}

impl AsRef<str> for StringRef {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for StringRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Debug for StringRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringRef::Owned(s) => write!(f, "StringRef::Owned({:?})", s),
            StringRef::Static(s) => write!(f, "StringRef::Static({:?})", s),
        }
    }
}

impl Clone for StringRef {
    fn clone(&self) -> Self {
        match self {
            StringRef::Owned(s) => StringRef::Owned(s.clone()),
            StringRef::Static(s) => StringRef::Static(s),
        }
    }
}
