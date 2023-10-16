mod input;

use base64::Engine;
use gloo::console;
use gloo::timers::callback::Interval;
use qrcode_generator::QrCodeEcc;
use yew::prelude::*;

pub enum Msg {
    StartInterval,
    Cancel,
    Tick,
    SetInput(String),
}

pub struct App {
    encoder: ur::Encoder<'static>,
    interval: Option<Interval>,
    current_part: Option<String>,
    input: String,
}

impl App {
    fn cancel(&mut self) {
        self.interval = None;
        self.current_part = None;
        self.encoder = ur::Encoder::bytes(b"placeholder", MAX_FRAGMENT_SIZE).unwrap();
        self.input = String::new();
    }
}

const MAX_FRAGMENT_SIZE: usize = 50;

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            encoder: ur::Encoder::bytes(b"placeholder", MAX_FRAGMENT_SIZE).unwrap(),
            interval: None,
            current_part: None,
            input: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StartInterval => {
                let handle = {
                    let link = ctx.link().clone();
                    Interval::new(1000, move || link.send_message(Msg::Tick))
                };
                self.interval = Some(handle);
                true
            }
            Msg::Cancel => {
                self.cancel();
                console::warn!("Canceled!");
                true
            }
            Msg::Tick => {
                self.current_part = Some(self.encoder.next_part().unwrap());
                true
            }
            Msg::SetInput(s) => {
                self.encoder = ur::Encoder::bytes(s.as_bytes(), MAX_FRAGMENT_SIZE).unwrap();
                self.input = s;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let has_job = self.interval.is_some();
        let qrcode_rendered = self.current_part.as_ref().map_or_else(
            || html! {},
            |part| {
                let qr = base64::prelude::BASE64_STANDARD
                    .encode(qrcode_generator::to_png_to_vec(part, QrCodeEcc::Low, 1024).unwrap());
                html! {
                    <div id="wrapper">
                    <div id="qrcode">
                    <img src= { format!("data:image/png;base64,{qr}") } width=300 />
                    </div>
                </div>
                }
            },
        );
        let part = self.current_part.as_ref().map_or_else(
            || {
                html! {
                    <></>
                }
            },
            |part| {
                html! {
                    <div id="part">
                        <code>{ part.to_string() }</code>
                    </div>
                }
            },
        );
        let on_change = ctx.link().callback(Msg::SetInput);
        html! {
            <>
                <h1>{ "Uniform Resources Demo" }</h1>
                <h4>{ "Enter the text you would like to transmit and click Start" }</h4>
                <div>
                    <crate::input::TextInput {on_change} value={self.input.clone()} />
                    <p></p>
                </div>
                <div id="buttons">
                    <button disabled={has_job} onclick={ctx.link().callback(|_| Msg::StartInterval)}>
                        { "Start" }
                    </button>
                    <button disabled={!has_job} onclick={ctx.link().callback(|_| Msg::Cancel)}>
                        { "Cancel" }
                    </button>
                </div>
                { qrcode_rendered }
                <p></p>
                { part }
            </>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
