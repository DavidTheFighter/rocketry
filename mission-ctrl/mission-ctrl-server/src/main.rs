use std::thread;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

use rocket::serde::{json::Json, Serialize};
use rocket::http::Header;
use rocket::{Request, Response, State};
use rocket::fairing::{Fairing, Info, Kind};

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct TelemetryData {
    igniter_gox_pressure: Vec<f32>,
    igniter_fuel_pressure: Vec<f32>,
    igniter_chamber_pressure: Vec<f32>,
    fuel_tank_pressure: Vec<f32>,
    ecu_board_temp: Vec<f32>,
    igniter_throat_temp: Vec<f32>,
}

struct InitData {
    init_timestamp: u128,
}

#[macro_use] extern crate rocket;

#[get("/telemetry")]
fn telemetry(init_data: &State<InitData>) -> Json<TelemetryData> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    let time = ((timestamp - init_data.init_timestamp) as f32) / 1000.0;    

    let gen_data = |s| {
        const NUM_SAMPLES: usize = 1000;
        let mut sensor_data = Vec::new();

        for i in 0..NUM_SAMPLES {
            let generated_value = (((i as f32) / ((NUM_SAMPLES as f32) / 10.0) + 1.57 * (s as f32) + time).sin() + 1.0) * 0.5 * 150.0;

            sensor_data.push(generated_value);
        }

        sensor_data
    };

    return Json(TelemetryData {
        igniter_gox_pressure: gen_data(0),
        igniter_fuel_pressure: gen_data(1),
        igniter_chamber_pressure: gen_data(2),
        fuel_tank_pressure: gen_data(3),
        ecu_board_temp: gen_data(4),
        igniter_throat_temp: gen_data(5),
    });
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

fn other() {
    loop {
        println!("Fishwings!");
        thread::sleep(Duration::from_millis(1000));
    }
}

#[launch]
fn rocket() -> _ {
    thread::spawn(other);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    
    rocket::build()
        .attach(CORS)
        .manage(InitData { init_timestamp: timestamp })
        .mount("/", routes![telemetry])
}