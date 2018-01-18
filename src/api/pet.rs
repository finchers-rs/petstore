use finchers::contrib::json::Json;
use finchers::contrib::urlencoded::{self, queries_req, Form, FromUrlEncoded};
use finchers::endpoint::{body, path, Endpoint};
use finchers::endpoint::method::{delete, get, post, put};
use finchers::handler::Handler;
use futures::Future;

use model::{Pet, Status};
use super::{Error, Petstore};

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    GetPet(u64),
    AddPet(Pet),
    UpdatePet(Pet),
    DeletePet(u64),
    FindPetsByStatuses(FindPetsByStatusesParam),
    FindPetsByTags(FindPetsByTagsParam),
    UpdatePetViaForm(u64, UpdatePetParam),
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

// TODO: upload image
pub fn endpoint() -> impl Endpoint<Item = Request, Error = Error> + Clone + 'static {
    use self::Request::*;

    endpoint![
        get("pet")
            .with(path().map_err(Error::endpoint))
            .map(|id| GetPet(id)),
        post("pet")
            .with(body().map_err(Error::endpoint))
            .map(|Json(pet)| AddPet(pet)),
        put("pet")
            .with(body().map_err(Error::endpoint))
            .map(|Json(pet)| UpdatePet(pet)),
        delete("pet")
            .with(path().map_err(Error::endpoint))
            .map(|id| DeletePet(id)),
        get("pet/findByStatus")
            .with(queries_req().map_err(Error::endpoint))
            .map(FindPetsByStatuses),
        get("pet/findByTags")
            .with(queries_req().map_err(Error::endpoint))
            .map(FindPetsByTags),
        post("pet")
            .with((
                path().map_err(Error::endpoint),
                body().map_err(Error::endpoint),
            ))
            .map(|(id, Form(param))| UpdatePetViaForm(id, param))
    ]
}

impl Handler<Request> for Petstore {
    type Item = super::PetstoreResponse;
    type Error = Error;
    type Future = super::PetstoreHandlerFuture;

    fn call(&self, request: Request) -> Self::Future {
        use self::Request::*;
        use super::PetstoreResponse::*;
        match request {
            GetPet(id) => self.db.get_pet(id).map(ThePet).into(),
            AddPet(pet) => self.db.add_pet(pet).map(PetCreated).into(),
            UpdatePet(pet) => self.db.update_pet(pet).map(|pet| ThePet(Some(pet))).into(),
            DeletePet(id) => self.db.delete_pet(id).map(|_| PetDeleted).into(),
            FindPetsByStatuses(param) => self.db.get_pets_by_status(param.status).map(Pets).into(),
            FindPetsByTags(param) => self.db.find_pets_by_tag(param.tags).map(Pets).into(),
            UpdatePetViaForm(id, param) => self.db
                .update_pet_name_status(id, param.name, param.status)
                .map(|pet| ThePet(Some(pet)))
                .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Request::*;

    use finchers::http::HttpRequest;
    use finchers::test::EndpointTestExt;
    use model::Status::*;

    #[test]
    fn test_add_pet() {
        let request = HttpRequest::get("/pet/42")
            .body(Default::default())
            .unwrap();
        match endpoint().run(request) {
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
            endpoint().run(request).map(|r| r.unwrap()),
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
            endpoint().run(request).map(|r| r.unwrap()),
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
            endpoint().run(request).map(|r| r.unwrap()),
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
