use super::PetId;

define_id!(OrderId: u64);

type DateTime = ::chrono::DateTime<::chrono::Local>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    id: Option<OrderId>,
    pet_id: Option<PetId>,
    quantity: Option<u32>,
    ship_date: Option<DateTime>,
    status: Option<OrderStatus>,
    complete: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderStatus {
    Placed,
    Approved,
    Delivered,
}
