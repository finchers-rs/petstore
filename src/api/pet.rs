use finchers::endpoint::prelude::*;
use finchers_json::json_body;
use finchers_urlencoded::{form_body, from_csv, queries_req};

use model::{Pet, PetId, Status};

#[derive(Debug, Deserialize)]
struct FindByStatusParam {
    #[serde(deserialize_with = "from_csv")]
    status: Vec<Status>,
}

#[derive(Debug, Deserialize)]
struct UpdatePetViaFormParam {
    name: Option<String>,
    status: Option<Status>,
}

pub fn api() -> impl Endpoint<Item = String> + 'static {
    let pet_id = path::<PetId>();
    let pet_body = json_body::<Pet>();

    endpoint("pet").with(choice![
        post(pet_body).map(|pet| format!("{:?}", pet)),
        put(pet_body).map(|pet| format!("{:?}", pet)),
        get("findByStatus")
            .with(queries_req::<FindByStatusParam>())
            .map(|queries| format!("{:?}", queries)),
        get(pet_id).map(|id| format!("{:?}", id)),
        post((pet_id, form_body::<UpdatePetViaFormParam>())).map(|param| format!("{:?}", param)),
        delete(pet_id).map(|id| format!("{:?}", id)),
    ])
}
