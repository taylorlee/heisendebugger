#![recursion_limit = "256"] // needed for html! macro expansion
#[macro_use]
extern crate yew;
extern crate num_complex;
extern crate serde;
extern crate serde_json;
extern crate stdweb;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate ndarray;


mod complex;
mod qvm;

use std::time::Duration;

use yew::html::{App, Html, KeyData};
use yew::{
    initialize, run_loop,
    services::{interval::IntervalService, Task},
};

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::document;
use stdweb::web::html_element::{InputElement, TextAreaElement};

pub struct Model {
    qvm: qvm::QVM,
    job: Option<Box<Task>>,
    running: bool,
}

pub enum Msg {
    Noop,
    Reset,
    Program,
    Gates,
    Start,
    Stop,
    Next,
    Prev,
}

pub struct Context {
    pub interval: IntervalService<Msg>,
}

fn main() {
    initialize();
    let mut app = App::new();
    let context = Context {
        interval: IntervalService::new(app.sender()),
    };
    let model = Model {
        job: None,
        qvm: qvm::QVM::new(),
        running: false,
    };
    app.mount(context, model, update, view);
    run_loop();
}

fn get_input(id: &str) -> String {
    let input: InputElement = document()
        .get_element_by_id(id)
        .unwrap()
        .try_into()
        .unwrap();
    input.raw_value()
}
fn get_text(id: &str) -> String {
    let input: TextAreaElement = document()
        .get_element_by_id(id)
        .unwrap()
        .try_into()
        .unwrap();
    input.value()
}
fn update(context: &mut Context, model: &mut Model, msg: Msg) {
    match msg {
        Msg::Noop => {}
        Msg::Reset => {
            model.qvm.reset();
        }
        Msg::Gates => {
            let gates = get_text("gates");
            model.qvm.set_gates(&gates);
        }
        Msg::Program => {
            let prog = get_input("program");
            model.qvm.update(prog);
        }
        Msg::Prev => {
            model.qvm.prev();
        }
        Msg::Next => {
            model.qvm.next();
        }
        Msg::Start => {
            let timeout = Duration::from_millis(500);
            let handle = context.interval.spawn(timeout, || Msg::Next);
            model.job = Some(Box::new(handle));
            model.running = true;
        }
        Msg::Stop => {
            if let Some(mut task) = model.job.take() {
                task.cancel();
            }
            model.job = None;
            model.running = false;
        }
    }
}
fn view(model: &Model) -> Html<Msg> {
    let pos = " ".repeat(model.qvm.counter) + "^";
    let program: String = model.qvm.program.iter().collect();
    let gates = model.qvm.show_gates();

    let controller = if model.running {
        html! {
        <button class="button", onclick=move|_| Msg::Stop,>{ "Stop" }</button> }
    } else {
        let keypress = |data: KeyData| {
            //warn!("{}",data.key);
            if data.key == "ArrowUp" {
                Msg::Next
            } else if data.key == "ArrowDown" {
                Msg::Prev
            } else {
                Msg::Program
            }
        };
        html! {
          <div>
            <div>{"gates"}</div>
            <textarea id="gates", cols=30, rows=25, onkeypress=move|_| Msg::Gates,>{gates} </textarea>
            <br></br>
            <input id="program", type="text", onkeypress=keypress, value={&program},/><span>{"program"}</span>
            <br></br>
            <input type="text", disabled=true, value={&pos},/><span>{"position"}</span>
            <br></br>
            <button class="button", onclick=move|_| Msg::Reset,>{ "Reset" }</button>
            <button class="button", onclick=move|_| Msg::Prev,>{ "Prev" }</button>
            <button class="button", onclick=move|_| Msg::Next,>{ "Next" }</button>
          </div>
        }
    };

    html! {
        <div>
            <section class="section",>
              { controller }
              <span class=("tag","is-primary"),> {"counter: "} { model.qvm.counter } </span>
              <br></br>
              <span class=("tag","is-primary"),> { &model.qvm } </span>
              <br></br>

            </section>
        </div>
    }
}
