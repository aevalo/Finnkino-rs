#[macro_use]
extern crate rocket;
#[macro_use]
extern crate derive_builder;

mod finnkino;
use finnkino::rocket::get_xml;

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

#[launch]
async fn rocket() -> _ {
  let area_xml = get_xml("https://www.finnkino.fi/xml/TheatreAreas").await;
  println!("{:?}", area_xml);
  rocket::build().mount("/", routes![index])
}
