#![recursion_limit = "256"] // needed for html! macro expansion
#![allow(dead_code)]
#![allow(unused_imports)]

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

use yew::html::{App, Html};
use yew::{initialize, run_loop};

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::document;
use stdweb::web::html_element::{InputElement, TextAreaElement};

enum State {
    Ready,
    Editing,
    Error,
}
pub struct Model {
    qvm: qvm::QVM,
    gate_state: State,
    gate_edit: String,
    program_state: State,
    program_edit: String,
}

pub enum Msg {
    Noop,
    Reset,
    Next,
    Prev,

    EditProgram,
    SaveProgram,

    EditGates,
    SaveGates,
}

struct Context {}

fn main() {
    initialize();
    let mut app = App::new();
    let model = Model {
        qvm: qvm::QVM::new(),
        gate_state: State::Ready,
        gate_edit: "".to_string(),
        program_state: State::Ready,
        program_edit: "".to_string(),

    };
    app.mount(Context {}, model, update, view);
    run_loop();
}

fn get_text(id: &str) -> String {
    let input: TextAreaElement = document()
        .get_element_by_id(id)
        .unwrap()
        .try_into()
        .unwrap();
    input.value()
}
fn update(_: &mut Context, model: &mut Model, msg: Msg) {
    match msg {
        Msg::Noop => {}
        Msg::Reset => {
            model.qvm.reset();
        }
        Msg::EditGates => {
            model.gate_state = State::Editing;
        }
        Msg::SaveGates => {
            let gates = get_text("gates");
            model.gate_state = if model.qvm.set_gates(&gates) {
                State::Ready
            } else {
                model.gate_edit = gates;
                State::Error
            };
        }
        Msg::EditProgram => {
            model.program_state = State::Editing;
        }
        Msg::SaveProgram => {
            let prog = get_text("program");
            model.program_state = if model.qvm.update(&prog) {
                State::Ready
            } else {
                model.program_edit = prog;
                State::Error
            };
        }
        Msg::Prev => {
            model.qvm.prev();
        }
        Msg::Next => {
            model.qvm.next();
        }
    }
}
fn view(model: &Model) -> Html<Msg> {
    let gates = match model.gate_state {
        State::Ready => html! {
            <button class="button", onclick=move|_| Msg::EditGates,>{"Edit Gates"}</button>
        },
        State::Editing => html! {
            <div>
                <button class="button", onclick=move|_| Msg::SaveGates,>{"Save Gates"}</button>
                <textarea id="gates", cols=30, rows=25, onkeypress=move|_| Msg::EditGates,>{model.qvm.show_gates()} </textarea>
            </div>
        },
        State::Error => html! {
            <div>
                <div>{"ERROR!"}</div>
                <button class="button", onclick=move|_| Msg::SaveGates,>{"Save Gates"}</button>
                <textarea id="gates", cols=30, rows=25, onkeypress=move|_| Msg::EditGates,>{&model.gate_edit}</textarea>
            </div>
        },
    };
    let program = match model.program_state {
        State::Ready => html! {
            <button class="button", onclick=move|_| Msg::EditProgram,>{"Edit Program"}</button>
        },
        State::Editing => html! {
            <div>
                <button class="button", onclick=move|_| Msg::SaveProgram,>{"Save Program"}</button>
                <textarea id="program", cols=30, rows=25, onkeypress=move|_| Msg::EditProgram,>{model.qvm.read_program()} </textarea>
            </div>
        },
        State::Error => html! {
            <div>
                <div>{"ERROR!"}</div>
                <button class="button", onclick=move|_| Msg::SaveProgram,>{"Save Program"}</button>
                <textarea id="progam", cols=30, rows=25, onkeypress=move|_| Msg::EditProgram,>{&model.program_edit}</textarea>
            </div>
        },
    };


    html! {
        <div>
            <section class="section",>
                { gates }
                <br></br>
                { program }
                <button class="button", onclick=move|_| Msg::Reset,>{ "Reset" }</button>
                <button class="button", onclick=move|_| Msg::Prev,>{ "Prev" }</button>
                <button class="button", onclick=move|_| Msg::Next,>{ "Next" }</button>
                <span class=("tag","is-primary"),> {"counter: "} { model.qvm.counter } </span>
                <br></br>
                <span class=("tag","is-primary"),> { &model.qvm } </span>
                <br></br>
            </section>
        </div>
    }
}
