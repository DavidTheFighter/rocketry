use rocket::serde::{Serialize, json::Json};


#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CommandResponse {
    text_response: String,
    success: bool,
}

#[post("/testvalve")]
pub fn testvalve() -> Json<CommandResponse> {
    Json(CommandResponse { 
        text_response: String::from("Test valve IG Fuel Main"), 
        success: true,
    })
}
