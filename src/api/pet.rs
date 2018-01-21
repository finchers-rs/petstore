use finchers::{Endpoint, Handler};
use finchers::contrib::urlencoded::{self, FromUrlEncoded};
use model::{Pet, Status};
use error::EndpointError;
use petstore::{Petstore, PetstoreError};
use self::Request::*;
use self::Response::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    GetPet(u64),
    AddPet(Pet),
    UpdatePet(Pet),
    DeletePet(u64),
    FindPetsByStatuses(FindPetsByStatusesParam),
    FindPetsByTags(FindPetsByTagsParam),
    // UpdatePetViaForm(u64, UpdatePetParam),
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
pub enum Response {
    ThePet(Pet),
    PetCreated(u64),
    Pets(Vec<Pet>),
    PetDeleted,
}

mod imp {
    use super::*;
    use api::common::*;

    impl IntoResponse for Response {
        fn into_response(self) -> HyperResponse {
            match self {
                ThePet(pet) => json_response(&pet),
                PetCreated(id) => json_response(&id).with_status(StatusCode::Created),
                Pets(id) => json_response(&id),
                PetDeleted => no_content(),
            }
        }
    }
}

// TODO: upload image
pub fn endpoint() -> impl Endpoint<Item = Request, Error = EndpointError> + Clone + 'static {
    use finchers::endpoint::prelude::*;
    use finchers::contrib::json::json_body;
    use finchers::contrib::urlencoded::{queries_req, Form};

    endpoint("pet").with(choice![
        get(path()).map(GetPet),
        post(json_body()).map(AddPet),
        put(json_body()).map(UpdatePet),
        delete(path()).map(DeletePet),
        get("findByStatus")
            .with(queries_req())
            .map(FindPetsByStatuses),
        get("findByTags").with(queries_req()).map(FindPetsByTags),
        //post((path().from_err::<EndpointError>(), body().from_err()))
        //    .map(|(id, Form(param))| UpdatePetViaForm(id, param))
    ])
}

impl Handler<Request> for Petstore {
    type Item = Response;
    type Error = PetstoreError;
    type Result = Result<Option<Self::Item>, Self::Error>;

    fn call(&self, request: Request) -> Self::Result {
        match request {
            GetPet(id) => self.get_pet(id).map(|p| p.map(ThePet)),
            AddPet(pet) => self.add_pet(pet).map(|id| Some(PetCreated(id))),
            UpdatePet(pet) => self.update_pet(pet).map(|pet| Some(ThePet(pet))),
            DeletePet(id) => self.delete_pet(id).map(|_| Some(PetDeleted)),
            FindPetsByStatuses(param) => self.get_pets_by_status(param.status)
                .map(|pets| Some(Pets(pets))),
            FindPetsByTags(param) => self.find_pets_by_tag(param.tags)
                .map(|pets| Some(Pets(pets))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
