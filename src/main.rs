#![recursion_limit = "256"] // needed for html! macro expansion

#[macro_use]
extern crate yew;

//mod qvm;

use std::time::Duration;
use yew::prelude::*;
use yew::services::{interval::IntervalService, Task};


pub struct Model {
    clock: u64,
    job: Option<Box<Task>>,
}

pub enum Msg {
    Start,
    Stop,
    Step,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model {
            clock: 0,
            job: None,
            interval: IntervalService::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Step => {
                self.clock += 1;
            }
            Msg::Start => {
                let timeout = Duration::from_millis(500);
                let handle = self.interval.spawn(timeout, || Msg::Step);
                self.job = Some(Box::new(handle));
            }
            Msg::Stop => {
                if let Some(mut task) = self.job.take() {
                    task.cancel();
                }
                self.job = None;
            }
        }
    }
}

fn view(&model) -> Html<Self> {
        html! {
            <div>
                <section class="section",>
                  <span class=("tag","is-primary"),> { self.clock } </span>
                  <button class="button", onclick=move|_| Msg::Step,>{ "Step" }</button>
                  <button class="button", onclick=move|_| Msg::Start,>{ "Start" }</button>
                  <button class="button", onclick=move|_| Msg::Stop,>{ "Stop" }</button>
                </section>
            </div>
        }
    }
}

fn main() {
    yew::initialize();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}
