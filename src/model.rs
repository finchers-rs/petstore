use std::fmt;
use std::io;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Category {
    pub id: Option<u64>,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Inventory {
    pub available: u32,
    pub pending: u32,
    pub adopted: u32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Order {
    pub id: Option<u64>,
    pub pet_id: Option<u64>,
    pub quantity: Option<u64>,
    pub ship_date: Option<String>,
    pub status: Option<OrderStatus>,
    pub complete: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum OrderStatus {
    Placed,
    Approved,
    Delivered,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::OrderStatus::*;
        match *self {
            Placed => f.write_str("placed"),
            Approved => f.write_str("approved"),
            Delivered => f.write_str("delivered"),
        }
    }
}

impl FromStr for OrderStatus {
    // TODO: replace to appropriate error type
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::OrderStatus::*;
        match s {
            "placed" => Ok(Placed),
            "approved" => Ok(Approved),
            "delivered" => Ok(Delivered),
            s => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("`{}' is invalid order status", s),
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Pet {
    pub id: Option<u64>,
    pub name: String,
    pub photo_urls: Vec<String>,
    pub category: Option<Category>,
    pub tags: Option<Vec<Tag>>,
    pub status: Option<Status>,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum Status {
    Available,
    Pending,
    Adopted,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Status::*;
        match *self {
            Available => f.write_str("available"),
            Pending => f.write_str("pending"),
            Adopted => f.write_str("adopted"),
        }
    }
}

impl FromStr for Status {
    // TODO: replace to appropriate error type
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Status::*;
        match s {
            "available" => Ok(Available),
            "pending" => Ok(Pending),
            "adopted" => Ok(Adopted),
            s => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("`{}' is invalid status", s),
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Tag {
    pub id: Option<u64>,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct User {
    pub id: Option<u64>,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub password: String,
    pub phone: Option<String>,
}

pub use self::OrderStatus::*;
pub use self::Status::*;
