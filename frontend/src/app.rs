use yew::{html, Component, ComponentLink, Html, ShouldRender};

use backend::db::{NewEntry};

#[derive(Debug)]
pub struct App {
    link: ComponentLink<Self>,
    state: State,
}

pub enum Msg {
    Submit,
}

#[derive(Debug)]
pub struct State {
    new_entry: NewEntry,
}

impl Component for App {
    // Some details omitted. Explore the examples to see more.

    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Set up app state
        let new_entry = NewEntry::new();
        let state = State { new_entry };
        App { link, state }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Submit => {
                // Post request to REST api
                true
            }
        }
    }

    fn view(&self) -> Html {
        let submit = self.link.callback(|_| Msg::Submit);

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
                        <input type="text" value=self.state.new_entry.start></input>
                    </div>
                    <div class="col-sm-3">
                        <h3>{"Stop"}</h3>
                        <input type="text" value=self.state.new_entry.stop></input>
                    </div>
                    <div class="col-sm-3">
                        <h3>{"Code"}</h3>
                        <input type="text" value=self.state.new_entry.code></input>
                    </div>
                    <div class="col-sm-3">
                        <h3>{"Memo"}</h3>
                        <input type="text" value=self.state.new_entry.memo></input>
                    </div>
                </div>
                <br/>
                <br/>
                <div class="mx-auto" style="width: 200px;">
                    <button class="btn btn-primary btn-lg" onclick=submit>{ "Submit" }</button>
                </div>
            </div>
        }
    }
}