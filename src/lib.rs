use serde::{Deserialize, Serialize};
use cucumber_rust::event::{CucumberEvent, FailureKind, FeatureEvent, ScenarioEvent, StepEvent, StepFailureKind};
use cucumber_rust::{EventHandler, RunResult};
use std::sync::{Arc, Mutex};

pub mod state {
    use super::*;

    #[derive(Default, Clone)]
    pub struct RunEventHandler {
        pub state: Arc<Mutex<EventHandlerState>>,
    }

    #[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum StatResult {
        Passed,
        Failed,
        Skipped
    }

    #[derive(Default, Clone, Serialize, Deserialize)]
    pub struct FeatureStats {
        pub name: String,
        pub scenarios: Vec<ScenarioStats>
    }

    #[derive(Default, Clone, Serialize, Deserialize)]
    pub struct StepStats {
        pub name: String,
        pub keyword: String,
        pub result: Option<StatResult>
    }

    #[derive(Default, Clone, Serialize, Deserialize)]
    pub struct ScenarioStats {
        pub name: String,
        pub steps: Vec<StepStats>,
        pub result: Option<StatResult>
    }

    #[derive(Default, Clone)]
    pub struct EventHandlerState {
        pub features: Vec<FeatureStats>
    }

    #[derive(Default, Clone, Serialize, Deserialize)]
    pub struct RunStats {
        pub total_featuress: u32,
        pub total_scenarios: u32,
        pub skipped_scenarios: u32,
        pub passed_scenarios: u32,
        pub failed_scenarios: u32,
        pub features: Vec<FeatureStats>
    }

    impl RunStats {
        pub fn new(result: &RunResult, state: &EventHandlerState) -> Self {
            Self {
                total_featuress: result.features.total,
                total_scenarios: result.scenarios.total,
                skipped_scenarios: result.scenarios.skipped,
                passed_scenarios: result.scenarios.passed,
                failed_scenarios: result.scenarios.failed,
                features: state.features.clone()
            }
        }
    }

    impl std::fmt::Display for StatResult {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let output = match self {
                &Self::Failed => "Failed",
                &Self::Passed => "Passed",
                &Self::Skipped => "Skipped"
            };

            write!(f, "{}", output)
        }
    }

    impl std::fmt::Display for StepStats {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Step: '{} {}'; Status: '{}'", self.keyword, self.name, self.result.clone().unwrap_or(state::StatResult::Skipped))
        }
    }

    impl std::fmt::Display for ScenarioStats {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Scenario: '{}'; Status: '{}'", self.name, self.result.clone().unwrap_or(state::StatResult::Skipped))
        }
    }

    impl std::fmt::Display for FeatureStats {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Feature: '{}'; Status: '{}'", self.name, self.get_result().unwrap_or(state::StatResult::Skipped))
        }
    }

    impl FeatureStats {
        fn new(name: String) -> Self {
            Self {
                name,
                scenarios: Vec::default()
            }
        }

        fn get_scenario(&mut self, scenario_name: String) -> Option<&mut ScenarioStats> {
            self.scenarios.iter_mut().find(|s|s.name == scenario_name)
        }

        pub fn get_result(&self) -> Option<StatResult> {
            match self.scenarios.len() {
                0 => None,
                _ => {
                    let all_stats: Vec<StatResult> = self.scenarios.iter().map(|s|s.result.clone().unwrap_or(StatResult::Skipped)).collect();
                    if all_stats.iter().all(|s|*s == StatResult::Passed) {
                        Some(StatResult::Passed)
                    } else if all_stats.iter().any(|s|*s == StatResult::Failed) {
                        Some(StatResult::Failed)
                    } else {
                        Some(StatResult::Skipped)
                    }
                }
            }
        }
    }

    impl ScenarioStats {
        fn new(name: String) -> Self {
            Self {
                name,
                steps: Vec::default(),
                result: None
            }
        }

        fn get_step(&mut self, step_keyword: String, step_name: String) -> Option<&mut StepStats> {
            self.steps.iter_mut().find(|s|s.name == step_name && s.keyword == step_keyword)
        }
    }

    impl StepStats {
        fn new(name: String, keyword: String) -> Self {
            Self {
                name,
                keyword,
                result: None
            }
        }
    }

    impl EventHandlerState {

        fn add_feature(&mut self, feature_name: String) {
            self.features.push(FeatureStats::new(feature_name));
        }

        fn add_scenario(&mut self, feature_name: String, scenario_name: String) {
            self.get_feature(feature_name.to_owned())
                .expect(format!("Feature '{}' not found while adding scenario", feature_name).as_str())
                .scenarios.push(ScenarioStats::new(scenario_name));
        }

        fn add_step(&mut self, feature_name: String, scenario_name: String, step_name: String, step_keyword: String) {
            self.get_feature(feature_name.to_owned())
                .expect(format!("Feature '{}' not found while adding step", feature_name).as_str())
                .get_scenario(scenario_name.to_owned())
                    .expect(format!("Scenario '{}' not found  while adding step", scenario_name).as_str())
                    .steps.push(StepStats::new(step_name, step_keyword));
        }

        fn set_scenario_result(&mut self, feature_name: String, scenario_name: String, result: StatResult) {
            self.get_feature(feature_name.to_owned())
                .expect(format!("Feature '{}' not found while setting result to scenario", feature_name).as_str())
                .get_scenario(scenario_name.to_owned())
                    .expect(format!("Scenario '{}' not found while setting result to scenario", scenario_name).as_str())
                    .result = Some(result);
        }

        fn set_step_result(&mut self, feature_name: String, scenario_name: String, step_name: String, step_keyword: String, result: StatResult) {
            self.get_feature(feature_name.to_owned())
                .expect(format!("Feature '{}' not found while setting result to step", feature_name).as_str())
                .get_scenario(scenario_name.to_owned())
                    .expect(format!("Scenario '{}' not found while setting result to step", scenario_name).as_str())
                    .get_step(step_keyword.to_owned(), step_name.to_owned())
                        .expect(format!("Step '{} {}' not found while setting result to step", step_keyword, step_name).as_str())
                        .result = Some(result);
        }

        fn get_feature(&mut self, feature_name: String) -> Option<&mut FeatureStats> {
            self.features.iter_mut().find(|f|f.name == feature_name)
        }
    }

    impl EventHandler for RunEventHandler {
        fn handle_event(&mut self, event: &CucumberEvent) {
            let mut state = self.state.lock().unwrap();

            match event {

                CucumberEvent::Feature(
                    _feature,
                    FeatureEvent::Scenario(_scenario, ScenarioEvent::Failed(FailureKind::Panic)),
                ) => state.set_scenario_result(_feature.name.to_owned(), _scenario.name.to_owned(), StatResult::Failed),

                CucumberEvent::Feature(
                    _feature,
                    FeatureEvent::Scenario(_scenario, ScenarioEvent::Passed),
                ) => state.set_scenario_result(_feature.name.to_owned(), _scenario.name.to_owned(), StatResult::Passed),

                CucumberEvent::Feature(
                    ref _feature,
                    FeatureEvent::Scenario(ref _scenario, ScenarioEvent::Skipped),
                ) => state.set_scenario_result(_feature.name.to_owned(), _scenario.name.to_owned(), StatResult::Skipped),

                CucumberEvent::Feature(
                    _feature,
                    FeatureEvent::Scenario(
                        _scenario,
                        ScenarioEvent::Step(_step, StepEvent::Failed(StepFailureKind::Panic(_, _))),
                    ),
                ) => {
                    state.set_step_result(_feature.name.to_owned(), _scenario.name.to_owned(), _step.value.to_owned(), _step.keyword.to_owned(), StatResult::Failed);
                },

                CucumberEvent::Feature(
                    _feature,
                    FeatureEvent::Scenario(
                        _scenario,
                        ScenarioEvent::Step(_step, StepEvent::Failed(StepFailureKind::TimedOut)),
                    ),
                ) => state.set_step_result(_feature.name.to_owned(), _scenario.name.to_owned(), _step.value.to_owned(), _step.keyword.to_owned(), StatResult::Failed),

                CucumberEvent::Feature(
                    _feature,
                    FeatureEvent::Scenario(_scenario, ScenarioEvent::Step(_step, StepEvent::Passed(_))),
                ) => state.set_step_result(_feature.name.to_owned(), _scenario.name.to_owned(), _step.value.to_owned(), _step.keyword.to_owned(), StatResult::Passed),

                CucumberEvent::Feature(_feature, FeatureEvent::Scenario(_scenario, ScenarioEvent::Step(_step, StepEvent::Starting)))
                    => state.add_step(_feature.name.to_owned(), _scenario.name.to_owned(), _step.value.to_owned(), _step.keyword.to_owned()),

                CucumberEvent::Feature(_feature, FeatureEvent::Starting)
                    => state.add_feature(_feature.name.to_owned()),

                CucumberEvent::Feature(ref _feature, FeatureEvent::Scenario(ref _scenario, ScenarioEvent::Starting(_)))
                    => state.add_scenario(_feature.name.to_owned(), _scenario.name.to_owned()),

                _ => {}
            }
        }
    }

    pub fn print_test_results(stats: &RunStats) {
        println!("");
        println!("Result overview!");
        println!("------------------------------------------------");
        println!("Total features: {}", stats.total_featuress);
        println!("Total scenarions: {}", stats.total_scenarios);
        println!("Skipped scenarios: {}", stats.skipped_scenarios);
        println!("Passed scenarios: {}", stats.passed_scenarios);
        println!("Failed scenarios: {}", stats.failed_scenarios);
        println!("------------------------------------------------");

        if stats.features.is_empty() {
            return;
        }

        println!("");
        println!("Feature overview:");
        println!("------------------------------------------------");

        stats.features.iter().for_each(|f|{
            println!("{}", f);
            println!("");
            f.scenarios.iter().for_each(|sc|{
                println!("\t{}", sc);
                sc.steps.iter().for_each(|st|{
                    println!("\t\t{}", st);
                });
                println!("");
            });
        });

        println!("");
    }

    pub fn write_result_file(filename: &String, stats: &RunStats) {
        let _ = std::fs::create_dir("./out");
        let output = serde_json::to_string(&stats).unwrap();
        let _ = std::fs::write(String::from("./out/") + filename, output);
    }
}

pub mod api {

    use std::time::UNIX_EPOCH;
    use std::{collections::HashMap, time::SystemTime};
    use std::string::FromUtf8Error;
    use hyper::{Body, Client, Request, Response, Uri};
    use hyper_tls::HttpsConnector;
    use hmac::{Hmac, Mac, NewMac};
    use sha2::Digest;
    
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
    type HmacSha512 = Hmac<crypto_hashes::sha2::Sha512>;

    pub struct ApiContext {
        pub otp: String,
        api_host: String,
        api_key: String,
        secret_key: String,
    }
    
    impl ApiContext {

        pub fn new(api_key: String, api_host: String, secret_key: String, otp: String) -> Self {
            Self {
                api_key,
                api_host,
                secret_key,
                otp
            }
        }

        pub fn get_public_api_url(&self) -> String {
            format!("https://{}/0/public/", self.api_host)
        }
    
        pub fn get_private_api_url(&self) -> String {
            format!("https://{}/0/private/", self.api_host)
        }
    
        pub fn get_nonce() -> u64 {
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            timestamp.as_secs()        
        }
    }

    pub async fn get(url: &str, params: HashMap<&str, &str>) ->  Result<Response<Body>> {    
        let uri = get_url_and_query_string(url, &params);
        let request = Request::builder()
            .uri(uri)
            .method("GET")
            .header("User-Agent", "bdd-awesome-agent/1.0")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::default())
            .unwrap();
    
        let https = HttpsConnector::new();
        let https_client = Client::builder().build::<_, hyper::Body>(https);
        let response = https_client.request(request).await?;
        Ok(response)
    }

    pub async fn post(url: &str, params: HashMap<&str, &str>, api_context: &ApiContext, nonce: &str) ->  Result<Response<Body>> {        
        let uri: Uri = url.parse().unwrap();
        let url_encoded_params = get_url_encoded_params(&params);
        let api_sign = get_api_sign(nonce, uri.path(), &api_context.secret_key, url_encoded_params.as_str());
            
        let request = Request::builder()
            .uri(uri.to_owned())
            .method("POST")
            .header("User-Agent", "bdd-awesome-agent/1.0")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("API-Key", &api_context.api_key)
            .header("API-Sign", api_sign) 
            .body(Body::from(url_encoded_params))
            .unwrap();
               
        let https = HttpsConnector::new();
        let https_client = Client::builder().build::<_, hyper::Body>(https);
        let response = https_client.request(request).await?;
        Ok(response)
    }
    
    pub async fn get_content_as_string(response: Box<Response<Body>>) -> std::result::Result<String, FromUtf8Error> {
        let body_content = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let content = String::from_utf8(body_content.into_iter().collect())?;
        Ok(content)
    }

    fn get_url_and_query_string(url: &str, params: &HashMap<&str, &str>) -> String {
        let mut uri = url.to_string();
        if !params.is_empty() {
            uri += "?";
            uri += &get_url_encoded_params(params);
        }

        uri
    }

    fn get_url_encoded_params(params: &HashMap<&str, &str>) -> String {
        params
            .iter()
            .enumerate()
            .map(|(index, (key, value))|if index == 0 { format!("{}={}", key, value) } else { format!("&{}={}", key, value) })
            .fold(String::default(), |a,b| a + &b)
    }
    
    fn get_api_sign(nonce: &str, uri_path: &str, secret_key: &str, url_encoded_params: &str) -> String {
        let sha256 = sha2::Sha256::digest((nonce.to_string() + url_encoded_params).as_bytes());
        let mut sha512_params: Vec<u8> = Vec::from(uri_path.as_bytes());
        sha512_params.extend_from_slice(sha256.as_slice());

        let secret_key_bytes = base64::decode(secret_key).unwrap();
        let mut mac = HmacSha512::new_from_slice(&secret_key_bytes).unwrap();
        mac.update(&sha512_params);
        
        let result = mac.finalize().into_bytes();
        base64::encode(result)
    }
}
