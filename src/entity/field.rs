pub trait Field {
    type Target;
    fn tantivy_val(&self) -> Self::Target;
}

/// Represents a tokenized String:
/// ```rust
/// TEXT | STORED
/// ```
pub struct Tokenized(String);
impl Field for Tokenized {
    type Target = String;
    fn tantivy_val(&self) -> Self::Target {
        self.0.clone()
    }
}
impl From<&str> for Tokenized {
    fn from(val: &str) -> Self {
        Tokenized(val.to_string())
    }
}
impl From<String> for Tokenized {
    fn from(val: String) -> Self {
        Tokenized(val)
    }
}

/// Represents an untokenized String:
/// ```rust
/// STRING | STORED
/// ```
pub struct Str(String);
impl Field for Str {
    type Target = String;
    fn tantivy_val(&self) -> Self::Target {
        self.0.clone()
    }
}
impl From<&str> for Str {
    fn from(val: &str) -> Self {
        Str(val.to_string())
    }
}
impl From<String> for Str {
    fn from(val: String) -> Self {
        Str(val)
    }
}

/// Represents an untokenized String stored for quick access:
/// ```rust
/// STRING | FAST | STORED
/// ```
pub struct FastStr(String);
impl Field for FastStr {
    type Target = String;
    fn tantivy_val(&self) -> Self::Target {
        self.0.clone()
    }
}

impl From<&str> for FastStr {
    fn from(val: &str) -> Self {
        FastStr(val.to_string())
    }
}
impl From<String> for FastStr {
    fn from(val: String) -> Self {
        FastStr(val)
    }
}

/// Represents a u64:
/// ```rust
/// STORED
/// ```
pub struct U64(u64);
impl Field for U64 {
    type Target = u64;
    fn tantivy_val(&self) -> Self::Target {
        self.0.clone()
    }
}

impl From<u64> for U64 {
    fn from(val: u64) -> Self {
        U64(val)
    }
}

/// Represents a u64 that is stored for quick access:
/// ```rust
/// FAST | STORED
/// ```
pub struct FastU64(u64);
impl Field for FastU64 {
    type Target = u64;
    fn tantivy_val(&self) -> Self::Target {
        self.0.clone()
    }
}

impl From<u64> for FastU64 {
    fn from(val: u64) -> Self {
        FastU64(val)
    }
}

/// Represents a f64:
/// ```rust
/// STORED
/// ```
pub struct F64(f64);
impl Field for F64 {
    type Target = f64;
    fn tantivy_val(&self) -> Self::Target {
        self.0.clone()
    }
}

impl From<f64> for F64 {
    fn from(val: f64) -> Self {
        F64(val)
    }
}

/// Represents a f64 that is stored for quick access:
/// ```rust
/// FAST | STORED
/// ```
pub struct FastF64(f64);
impl Field for FastF64 {
    type Target = f64;
    fn tantivy_val(&self) -> Self::Target {
        self.0.clone()
    }
}

impl From<f64> for FastF64 {
    fn from(val: f64) -> Self {
        FastF64(val)
    }
}

/// Represents a f32
/// 
/// This field as special as it is not actually stored in the search index.
pub struct Score(f32);
impl Field for Score {
    type Target = f32;
    fn tantivy_val(&self) -> Self::Target {
        self.0
    }
}
impl From<f32> for Score {
    fn from(val: f32) -> Self {
        Score(val)
    }
}

/// Represents a date:
/// ```rust
/// INDEXED | STORED
/// ```
pub struct Date(tantivy::DateTime);
impl Field for Date {
    type Target = tantivy::DateTime;
    fn tantivy_val(&self) -> Self::Target {
        self.0.clone()
    }
}

impl From<tantivy::DateTime> for Date {
    fn from(val: tantivy::DateTime) -> Self {
        Date(val)
    }
}
