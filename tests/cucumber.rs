use cucumber_rust::{Context, Cucumber, World, async_trait, t};
use hyper::{Response, Body};
use serde_json::Value;
use somebdd::state::{EventHandlerState, RunStats, RunEventHandler, print_test_results, write_result_file};
use somebdd::api::{ApiContext};
use std::{convert::Infallible};
use std::env;

pub struct MyWorld {
    base_url: Option<String>,
    url: Option<String>,
    last_response: Option<Box<Response<Body>>>,
    last_content_response: Option<Value>
}

mod test_steps {
    use super::*;
    use crate::MyWorld;    
    use std::{collections::HashMap};
    use serde_json::Value;
    use somebdd::api::{get, post, get_content_as_string, ApiContext};
    use cucumber_rust::{Steps};
    use spectral::{self, asserting, boolean::BooleanAssertions};

    #[async_trait(?Send)]
    impl World for MyWorld {
        type Error = Infallible;

        async fn new() -> std::result::Result<Self, Infallible> {
            Ok(Self {
                base_url: None,
                url: None,
                last_response: None,
                last_content_response: None,
            })
        }
    }

    impl MyWorld {
        fn set_url_with_path(&mut self, path: &str) {
            if let Some(base_url) = self.base_url.clone() {
                let url = base_url + path;
                self.url = Some(url.to_owned());
            }
        }
    }

    pub fn steps() -> Steps<MyWorld> {
        let mut builder: Steps<MyWorld> = Steps::new();

        builder.given("request is not authenticated", |mut world: crate::MyWorld, _ctx| {
            let api_context  = _ctx.get::<ApiContext>().unwrap();
            world.base_url = Some(api_context.get_public_api_url());
            world
        });

        builder.given("request is authenticated", |mut world: crate::MyWorld, _ctx| {
            let api_context  = _ctx.get::<ApiContext>().unwrap();
            world.base_url = Some(api_context.get_private_api_url());
            world
        });

        builder.when_async("system time is requested", t!(|mut world: crate::MyWorld, _ctx|{
            world.set_url_with_path("Time");
            let url = &world.url.take().unwrap();
            let response = get(url, HashMap::default()).await;
            if response.is_ok() {
                world.last_response = Some(Box::new(response.unwrap()))
            }
            world
        }));

        builder.when_regex_async("asset pair information is requested for (.*) and (.*)", t!(|mut world: crate::MyWorld, _ctx|{
            world.set_url_with_path("AssetPairs");
            let pair = _ctx.matches[1].to_owned() + &_ctx.matches[2].to_owned();
            let mut params: HashMap<&str, &str> = HashMap::default();
            params.insert("pair", &pair);
            let url = &world.url.take().unwrap();
            let response = get(url, params).await;
            if response.is_ok() {
                world.last_response = Some(Box::new(response.unwrap()))
            }
            world
        }));

        builder.when_async("all current open orders are requested", t!(|mut world: crate::MyWorld, _ctx|{
            let mut params: HashMap<&str, &str> = HashMap::default();
            let api_context  = _ctx.get::<ApiContext>().unwrap();
            let nonce = &ApiContext::get_nonce().to_string();
            params.insert("nonce", nonce);
            params.insert("otp", &api_context.otp);
            world.set_url_with_path("OpenOrders");
            let response = post(&world.url.clone().unwrap(), params, &api_context, nonce).await;
            if response.is_ok() {
                world.last_response = Some(Box::new(response.unwrap()))
            }
            world
        }));

        builder.then_async("gets successful response as json", t!(|mut world: crate::MyWorld, _ctx| {
            asserting(&"request was successful").that(&world.last_response.is_some()).is_true();
            let response_option = world.last_response.take();
            let response = response_option.unwrap();
            let content_type_header = response.headers().get("Content-Type");
            asserting(&"response contains header Content-Type").that(&content_type_header.is_some()).is_true();
            asserting(&"Content-Type is application/json").that(&content_type_header.unwrap().to_str().unwrap()).is_equal_to("application/json; charset=utf-8");
            let raw_content = get_content_as_string(response).await.expect("Impossible to get content as ut8");
            let content: Value = serde_json::from_str(raw_content.as_str()).expect("Impossible to get content as json");
            world.last_content_response = Some(content);
            world
        }));

        builder.then("response contains error list as empty", |world: crate::MyWorld, _ctx| {            
            let content = world.last_content_response.clone().unwrap();
            let errors= content["error"].as_array().expect("Impossible to get error property as array");
            asserting(&"error property is empty").that(&errors.len()).is_equal_to(0) ;
            world
        });

        builder.then("response contains order list as empty", |world: crate::MyWorld, _ctx| {            
            let content = world.last_content_response.clone().unwrap();
            let result = content["result"].as_object().unwrap();
            let open_orders = result.get("open").unwrap().as_object().unwrap();             
            asserting(&"open orders list is empty").that(&open_orders.keys().len()).is_equal_to(0);
            world
        });
        
        builder.then_regex("response only contains asset pair information (.*)", |world: crate::MyWorld, _ctx| {
            let content = world.last_content_response.clone().unwrap();
            let property_name = _ctx.matches[1].to_owned();
            let result = content["result"].as_object().expect("Impossible to get result object from response");
            result.get(&property_name).expect(format!("Impossible to get property '{}'", property_name).as_str());
            asserting(&"result only contains one property").that(&result.len()).is_equal_to(1);
            world
        });

        builder.then_regex("asset pair information for (.*) and (.*) as (.*) is as expected", |world: crate::MyWorld, _ctx| {
            let first_currency = _ctx.matches[1].to_owned();
            let second_currency = _ctx.matches[2].to_owned();
            let pair_id = _ctx.matches[3].to_owned();
            let content = world.last_content_response.clone().unwrap();
            let result = content["result"].as_object().unwrap();
            let pair = result.get(&pair_id).unwrap().as_object().unwrap();

            let expected_string_properties = vec![
                "altname",
                "wsname",
                "aclass_base",
                "base",
                "aclass_quote",
                "quote",
                "lot",
                "fee_volume_currency",
                "ordermin"
            ];

            let expected_numeric_properties = vec![
                "pair_decimals",
                "lot_decimals",
                "lot_multiplier",
                "margin_call",
                "margin_stop",
            ];

            let expected_array_properties = vec![
                "leverage_buy",
                "leverage_sell",
                "fees",
                "fees_maker",
            ];

            let expected_properties = [
                expected_string_properties.clone(),
                expected_numeric_properties.clone(),
                expected_array_properties.clone()
            ].concat();
            
            expected_properties.iter().for_each(|property|{
                asserting(format!("contains property {}", property).as_str()).that(&pair.contains_key(*property)).is_true();
            });

            expected_string_properties.iter().for_each(|property|{
                asserting(format!("property {} value is string type", property).as_str()).that(&pair.get(*property).unwrap().is_string()).is_true();
            });

            expected_numeric_properties.iter().for_each(|property|{
                asserting(format!("property {} value is numeric type", property).as_str()).that(&pair.get(*property).unwrap().is_number()).is_true();
            });

            expected_array_properties.iter().for_each(|property|{
                asserting(format!("property {} value is array type", property).as_str()).that(&pair.get(*property).unwrap().is_array()).is_true();
            });

            asserting(&"altname contains the expected value").that(&pair.get("altname").unwrap().as_str().unwrap()).is_equal_to((first_currency.to_owned() + &second_currency).as_str());
            asserting(&"wsname contains the expected value").that(&pair.get("wsname").unwrap().as_str().unwrap()).is_equal_to(format!("{}/{}", first_currency, second_currency).as_str());

            world
        });

        builder
    }
}

#[tokio::main]
async fn main() {
    
    let set_and_run_world = |world: Cucumber<MyWorld>, api_host: String, api_key: String, secret_key: String, otp: String|{
        world
        .context(Context::new().add(ApiContext::new(api_key, api_host, secret_key, otp)))
        .features(&["./features"])
        .steps(test_steps::steps())
        .enable_capture(true)
    };
    
    let params: Vec<String> = env::args().skip(1).collect();
    let host = match params.get(0) {
        Some(h) => h.to_owned(),
        _ => panic!("You must provide the API host as first parameter")
    };

    let api_key = match params.get(1) {
        Some(k) => k.to_owned(),
        _ => panic!("You must provide the API Key as second parameter")
    };

    let secret_key = match params.get(2) {
        Some(k) => k.to_owned(),
        _ => panic!("You must provide the Secret Key as third parameter")
    };

    let otp = match params.get(3) {
        Some(p) => p.to_owned(),
        _ => panic!("You must provide the otp as fourth parameter")
    };

    match params.get(4) {
        Some(filename) => {
            let event_handler = RunEventHandler::default();
            let world = Cucumber::with_handler(event_handler.clone());
            let result = set_and_run_world(world, host, api_key, secret_key, otp).run().await;
            let state: EventHandlerState = event_handler.state.lock().unwrap().clone();    
            let stats = RunStats::new(&result, &state);
            print_test_results(&stats);
            write_result_file(filename, &stats);
            let code = if result.failed() { 1 } else { 0 };
            std::process::exit(code);
        },
        _ => {
            let world = Cucumber::<MyWorld>::new();
            set_and_run_world(world, host, api_key, secret_key, otp).run_and_exit().await;
        }
    };
}
