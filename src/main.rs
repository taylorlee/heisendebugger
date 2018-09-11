#![recursion_limit = "256"] // needed for html! macro expansion

#[macro_use]
extern crate yew;
extern crate stdweb;

mod qvm;
mod complex;

use std::time::Duration;

use yew::{initialize, run_loop, services::{Task, interval::IntervalService}};
use yew::{html::{App, Html, KeyData}};

use stdweb::web::document;
use stdweb::web::html_element::InputElement;
use stdweb::unstable::TryInto;

pub struct Model {
    qvm: qvm::QVM, 
    job: Option<Box<Task>>,
    running: bool,
}

pub enum Msg {
    Noop,
    Reset,
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

fn update(context: &mut Context, model: &mut Model, msg: Msg) {
    match msg {
        Msg::Noop => {}
        Msg::Reset => {
            let input: InputElement = document()
                .get_element_by_id("program")
                .unwrap()
                .try_into()
                .unwrap();
            let program = input.value().try_into().unwrap();
            model.qvm.reset(program);
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
    let program: String = model
        .qvm
        .program
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if i == model.qvm.counter {
                c.to_ascii_uppercase()
            } else {
                *c
            }
        })
        .collect();

    let controller = if model.running {
        html! {
          <button class="button", onclick=move|_| Msg::Stop,>{ "Stop" }</button>
        }
    } else {
        let key = |data: KeyData| {
            if data.key == "Enter" {
                Msg::Reset
            } else {
                Msg::Noop
            }
        };

        html! {
          <div>
              <input id="program", type="text", onkeypress=key, value={&program},/>
              <button class="button", onclick=move|_| Msg::Reset,>{ "Reset" }</button>
              <button class="button", onclick=move|_| Msg::Start,>{ "Start" }</button>
              <button class="button", onclick=move|_| Msg::Prev,>{ "Prev" }</button>
              <button class="button", onclick=move|_| Msg::Next,>{ "Next" }</button>
          </div>
        }
    };

    html! {
        <div>
            <section class="section",>
              <span class=("tag","is-primary"),> {"program: "} { &program } </span>
              <br></br>
              <span class=("tag","is-primary"),> {"counter: "} { model.qvm.counter } </span>
              <br></br>
              <span class=("tag","is-primary"),> { model.qvm.qb.0.repr() } </span>
              <br></br>
              <span class=("tag","is-primary"),> { model.qvm.qb.1.repr() } </span>
              <br></br>
              { controller }
            </section>
        </div>
    }
}


