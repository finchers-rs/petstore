use std::fmt;
use std::error::Error as StdError;

use finchers::Endpoint;
use finchers::endpoint::{body, ok, path};
use finchers::endpoint::method::{delete, get, post, put};

use finchers::contrib::json::Json;
use finchers::contrib::urlencoded::{self, queries_opt, Form, FromUrlEncoded};

use model::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    GetPet(u64),
    AddPet(Pet),
    UpdatePet(Pet),
    DeletePet(u64),
    FindPetsByStatuses(FindPetsByStatusesParam),
    FindPetsByTags(FindPetsByTagsParam),
    UpdatePetViaForm(u64, UpdatePetParam),

    // store APIs
    GetInventory,
    AddOrder(Order),
    DeleteOrder(u64),
    FindOrder(u64),

    // users APIs
    AddUser(User),
    AddUsersViaList(Vec<User>),
    DeleteUser(String),
    GetUser(String),
    UpdateUser(User),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FindPetsByStatusesParam {
    pub status: Vec<Status>,
}

impl FromUrlEncoded for FindPetsByStatusesParam {
    fn from_urlencoded(iter: urlencoded::Parse) -> Result<Self, urlencoded::UrlDecodeError> {
        let mut status = None;
        for (key, value) in iter {
            match &*key {
                "status" => {
                    status = Some(value
                        .split(",")
                        .map(|s| s.parse())
                        .collect::<Result<_, _>>()
                        .map_err(urlencoded::UrlDecodeError::other)?)
                }
                s => return Err(urlencoded::UrlDecodeError::invalid_key(s.to_string())),
            }
        }
        Ok(FindPetsByStatusesParam {
            status: status.ok_or_else(|| urlencoded::UrlDecodeError::missing_key("status"))?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FindPetsByTagsParam {
    pub tags: Vec<String>,
}

impl FromUrlEncoded for FindPetsByTagsParam {
    fn from_urlencoded(iter: urlencoded::Parse) -> Result<Self, urlencoded::UrlDecodeError> {
        let mut tags = None;
        for (key, value) in iter {
            match &*key {
                "tags" => tags = Some(value.split(",").map(ToOwned::to_owned).collect()),
                s => return Err(urlencoded::UrlDecodeError::invalid_key(s.to_string())),
            }
        }
        Ok(FindPetsByTagsParam {
            tags: tags.ok_or_else(|| urlencoded::UrlDecodeError::missing_key("tags"))?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdatePetParam {
    pub name: Option<String>,
    pub status: Option<Status>,
}

impl FromUrlEncoded for UpdatePetParam {
    fn from_urlencoded(iter: urlencoded::Parse) -> Result<Self, urlencoded::UrlDecodeError> {
        let mut name = None;
        let mut status = None;
        for (key, value) in iter {
            match &*key {
                "name" => name = Some(value.into_owned()),
                "status" => status = Some(value.parse().map_err(urlencoded::UrlDecodeError::other)?),
                s => return Err(urlencoded::UrlDecodeError::invalid_key(s.to_string())),
            }
        }
        Ok(UpdatePetParam { name, status })
    }
}

#[derive(Debug)]
pub enum EndpointError {
    FromFramework(Box<StdError + 'static>),
    MissingQuery,
}

impl EndpointError {
    pub fn from<E: StdError + 'static>(err: E) -> Self {
        EndpointError::FromFramework(Box::new(err))
    }
}

impl fmt::Display for EndpointError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EndpointError::FromFramework(ref e) => e.fmt(f),
            EndpointError::MissingQuery => f.write_str("missing query string"),
        }
    }
}

impl StdError for EndpointError {
    fn description(&self) -> &str {
        "during parsing incoming HTTP request"
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            EndpointError::FromFramework(ref e) => Some(&**e),
            _ => None,
        }
    }
}

pub use self::Request::*;


pub fn petstore_endpoint() -> impl Endpoint<Item = Request, Error = EndpointError> + 'static {
    // TODO: upload image
    let pets = endpoint![
        get("pet")
            .with(path().map_err(EndpointError::from))
            .map(|id| GetPet(id)),
        post("pet")
            .with(body().map_err(EndpointError::from))
            .map(|Json(pet)| AddPet(pet)),
        put("pet")
            .with(body().map_err(EndpointError::from))
            .map(|Json(pet)| UpdatePet(pet)),
        delete("pet")
            .with(path().map_err(EndpointError::from))
            .map(|id| DeletePet(id)),
        get("pet/findByStatus")
            .with(queries_opt().map_err(EndpointError::from))
            .and_then(|res| match res {
                Some(q) => Ok(FindPetsByStatuses(q)),
                None => Err(EndpointError::MissingQuery),
            }),
        get("pet/findByTags")
            .with(queries_opt().map_err(EndpointError::from))
            .and_then(|res| match res {
                Some(q) => Ok(FindPetsByTags(q)),
                None => Err(EndpointError::MissingQuery),
            }),
        post("pet")
            .with((
                path().map_err(EndpointError::from),
                body().map_err(EndpointError::from),
            ))
            .map(|(id, Form(param))| UpdatePetViaForm(id, param))
    ];

    let store = endpoint![
        get("store/inventory").with(ok(GetInventory)),
        post("store/order")
            .with(body().map_err(EndpointError::from))
            .map(|Json(order)| AddOrder(order)),
        delete("store/order")
            .with(path().map_err(EndpointError::from))
            .map(|id| DeleteOrder(id)),
        get("store/order")
            .with(path().map_err(EndpointError::from))
            .map(|id| FindOrder(id)),
    ];

    let users = endpoint![
        post("user")
            .with(body().map_err(EndpointError::from))
            .map(|Json(u)| AddUser(u)),
        post("user/createWithList")
            .with(body().map_err(EndpointError::from))
            .map(|Json(body)| AddUsersViaList(body)),
        post("user/createWithArray")
            .with(body().map_err(EndpointError::from))
            .map(|Json(body)| AddUsersViaList(body)),
        delete("user")
            .with(path().map_err(EndpointError::from))
            .map(|n| DeleteUser(n)),
        get("user")
            .with(path().map_err(EndpointError::from))
            .map(|n| GetUser(n)),
        put("user")
            .with(body().map_err(EndpointError::from))
            .map(|Json(u)| UpdateUser(u)),
    ];

    endpoint![pets, store, users,]
}

#[cfg(test)]
mod tests {
    use super::*;
    use finchers::http::HttpRequest;
    use finchers::test::EndpointTestExt;

    #[test]
    fn test_add_pet() {
        let request = HttpRequest::get("/pet/42")
            .body(Default::default())
            .unwrap();
        match petstore_endpoint().run(request) {
            Some(Ok(req)) => assert_eq!(req, GetPet(42)),
            _ => panic!(),
        }
    }

    #[test]
    fn test_find_pets_by_status() {
        let request = HttpRequest::get("/pet/findByStatus?status=available,adopted")
            .body(Default::default())
            .unwrap();
        assert_eq!(
            petstore_endpoint().run(request).map(|r| r.unwrap()),
            Some(FindPetsByStatuses(FindPetsByStatusesParam {
                status: vec![Available, Adopted],
            }))
        );
    }

    #[test]
    fn test_find_pets_by_tags() {
        let request = HttpRequest::get("/pet/findByTags?tags=cat,cute")
            .body(Default::default())
            .unwrap();
        assert_eq!(
            petstore_endpoint().run(request).map(|r| r.unwrap()),
            Some(FindPetsByTags(FindPetsByTagsParam {
                tags: vec!["cat".into(), "cute".into()],
            }))
        );
    }

    #[test]
    fn test_update_pet_via_form() {
        let request = HttpRequest::post("/pet/42")
            .body("name=Alice&status=available".into())
            .unwrap();
        assert_eq!(
            petstore_endpoint().run(request).map(|r| r.unwrap()),
            Some(UpdatePetViaForm(
                42,
                UpdatePetParam {
                    name: Some("Alice".into()),
                    status: Some(Available),
                }
            ))
        );
    }
}