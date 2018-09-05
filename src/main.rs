#![recursion_limit = "256"] // needed for html! macro expansion

#[macro_use]
extern crate yew;

mod qvm;
mod complex;

use std::time::Duration;
use yew::{initialize, run_loop, html::{App, Html}, services::{Task, interval::IntervalService}};

pub struct Model {
    qvm: qvm::QVM, 
    job: Option<Box<Task>>,
}

pub enum Msg {
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
    };
    app.mount(context, model, update, view);
    run_loop();
}

fn update(context: &mut Context, model: &mut Model, msg: Msg) {
    match msg {
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
        }
        Msg::Stop => {
            if let Some(mut task) = model.job.take() {
                task.cancel();
            }
            model.job = None;
        }
    }
}

fn view(model: &Model) -> Html<Msg> {
    html! {
        <div>
            <section class="section",>
              <span class=("tag","is-primary"),> {"counter: "} { model.qvm.counter } </span>
              <br></br>
              <span class=("tag","is-primary"),> {"a: "} { model.qvm.qb.0.repr() } </span>
              <br></br>
              <span class=("tag","is-primary"),> {"b: "} { model.qvm.qb.1.repr() } </span>
              <br></br>
              <button class="button", onclick=move|_| Msg::Prev,>{ "Prev" }</button>
              <button class="button", onclick=move|_| Msg::Next,>{ "Next" }</button>
              <button class="button", onclick=move|_| Msg::Start,>{ "Start" }</button>
              <button class="button", onclick=move|_| Msg::Stop,>{ "Stop" }</button>
            </section>
        </div>
    }
}


