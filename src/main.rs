#![recursion_limit = "256"] // needed for html! macro expansion

#[macro_use]
extern crate yew;

//mod qvm;

use std::time::Duration;
use yew::{initialize, run_loop, html::{App, Html}, services::{Task, interval::IntervalService}};

pub struct Model {
    clock: u64,
    job: Option<Box<Task>>,
}

pub enum Msg {
    Start,
    Stop,
    Step,
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
        clock: 0,
        job: None,
    };
    app.mount(context, model, update, view);
    run_loop();
}

fn update(context: &mut Context, model: &mut Model, msg: Msg) {
    match msg {
        Msg::Step => {
            model.clock += 1;
        }
        Msg::Start => {
            let timeout = Duration::from_millis(500);
            let handle = context.interval.spawn(timeout, || Msg::Step);
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
              <span class=("tag","is-primary"),> { model.clock } </span>
              <button class="button", onclick=move|_| Msg::Step,>{ "Step" }</button>
              <button class="button", onclick=move|_| Msg::Start,>{ "Start" }</button>
              <button class="button", onclick=move|_| Msg::Stop,>{ "Stop" }</button>
            </section>
        </div>
    }
}


