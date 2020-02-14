use yew::{html, Component, ComponentLink, Html, InputData, ShouldRender};
use yew::services::fetch::{FetchService, Request, Response, FetchTask};
use yew::format::{Json, Nothing};
use failure::Error;
use web_sys;
use serde::{Serialize, Deserialize};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[derive(Debug)]
pub struct App {
    link: ComponentLink<Self>,
    fetch_service: FetchService,
    fetching: bool,
    state: State,
    ft: Option<FetchTask>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    start: String,
    stop: String,
    week_day: String,
    code: String,
    memo: String,
}

pub enum Msg {
    StartTime(String),
    StopTime(String),
    Code(String),
    Memo(String),
    FetchData,
    FetchReady(Result<String, Error>),
    PostData,
    PostSuccess,
    Failure,
}

#[derive(Debug)]
pub struct State {
    new_entry: Entry,
    message: String,
}

impl App {
    fn get_hello(&mut self) -> FetchTask {
        let request = Request::get("http://127.0.0.1:3030")
            .body(Nothing)
            .expect("Failed to build get request.");
        let cb = self.link.callback(move |response: Response<Result<String, Error>>| {
            let (meta, body) = response.into_parts();
            if meta.status.is_success() {
                Msg::FetchReady(body)
            } else {
                Msg::Failure
            }
        });
        self.fetch_service.fetch(request, cb)
    }
    fn post_entry(&mut self) -> FetchTask {
        let request = Request::post("http://127.0.0.1:3030/new_entry")
            .header("Content-Type", "application/json")
            .body(Json(&self.state.new_entry))
            .expect("Failed to build get request.");

        let cb = self.link.callback(| response: Response<Result<String, Error>>| {
            if response.status().is_success() {
                Msg::PostSuccess
            } else {
                log!("{}", response.status());
                Msg::Failure
            }
        });
        self.fetch_service.fetch(request, cb)
    }
}

impl Component for App {
    // Some details omitted. Explore the examples to see more.

    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Set up app state
        let new_entry = Entry {
            start: String::new(),
            stop: String::new(),
            week_day: String::new(),
            code: String::new(),
            memo: String::new(),
        };
        let state = State { new_entry, message: String::from("Message") };
        App { 
            link,
            state,
            fetch_service: FetchService::new(),
            fetching: false,
            ft: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StartTime(start) => {
                self.state.new_entry.start = start;
                false
            },
            Msg::StopTime(stop) => {
                self.state.new_entry.stop = stop;
                self.state.new_entry.week_day = String::from("Fri");
                false
            },
            Msg::Code(code) => {
                self.state.new_entry.code = code;
                false
            },
            Msg::Memo(memo) => {
                self.state.new_entry.memo = memo;
                false
            },
            Msg::FetchData => {
                self.fetching = true;
                self.ft = Some(self.get_hello());
                true
            },
            Msg::PostData => {
                self.ft = Some(self.post_entry());
                true
            },
            Msg::FetchReady(response) => {
                self.fetching = false;
                self.state.message = response.map(|data| data).unwrap();
                true
            },
            Msg::PostSuccess => {
                log!("Sucessful post!");
                true
            },
            Msg::Failure => {
                self.state.message = "Failed!".into();
                true
            }
        }
    }

    fn view(&self) -> Html {
        let start = self.link.callback(|e: InputData| Msg::StartTime(e.value));
        let stop = self.link.callback(|e: InputData| Msg::StopTime(e.value));
        let code = self.link.callback(|e: InputData| Msg::Code(e.value));
        let memo = self.link.callback(|e: InputData| Msg::Memo(e.value));
        let submit = self.link.callback(|_| Msg::PostData);

        html! {
            // Render your model here
            <div class="container">
                <div class="mx-auto" style="width: 200px;">
                    <h2>{"Timecard"}</h2>
                </div>
                    <div class="row">
                        <div class="col">
                            <h3>{"New Entry"}</h3>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-sm-3">
                            <h3>{"Start"}</h3>
                            <input type="text" oninput=start></input>
                        </div>
                        <div class="col-sm-3">
                            <h3>{"Stop"}</h3>
                            <input type="text" oninput=stop></input>
                        </div>
                        <div class="col-sm-3">
                            <h3>{"Code"}</h3>
                            <input type="text" oninput=code></input>
                        </div>
                        <div class="col-sm-3">
                            <h3>{"Memo"}</h3>
                            <input type="text" oninput=memo></input>
                        </div>
                    </div>
                <br/>
                <br/>
                <div class="mx-auto" style="width: 200px;">
                    <button class="btn btn-primary btn-lg" onclick=submit>{ "Submit" }</button>
                </div>
                <br/>
                <br/>
                <h1>{&self.state.message}</h1>
                <br/>
            </div>
        }
    }
}