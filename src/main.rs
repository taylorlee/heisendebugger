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

//#[macro_use]
//extern crate ndarray;

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
}

#[derive(Clone, Copy)]
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
struct Editor {
    state: State,
    edit: String,
    error: bool,
}

pub struct Model {
    qvm: qvm::QVM,
    gates: Editor,
    program: Editor,
}

fn main() {
    initialize();
    let mut app = App::new();
    let mut model = Model {
        qvm: qvm::QVM::new(),
        gates: Editor {
            state: State::Ready,
            edit: "".to_string(),
            error: false,
        },
        program: Editor {
            state: State::Ready,
            edit: "
cnot 0 1
cnot 0 1
cnot 1 0
cnot 1 0
".to_string(),
            error: false,
        },
    };
    model.qvm.update(&model.program.edit);
    model.gates.edit = model.qvm.show_gates();
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
            model.gates.state = State::Editing;
        }
        Msg::SaveGates => {
            let gates = get_text("gates");
            let editor = &mut model.gates;
            editor.state = if model.qvm.set_gates(&gates) {
                editor.edit = gates;
                editor.error = false;
                State::Ready
            } else {
                editor.edit = gates;
                editor.error = true;
                State::Editing
            };
        }
        Msg::EditProgram => {
            model.program.state = State::Editing;
        }
        Msg::SaveProgram => {
            let prog = get_text("program");
            let editor = &mut model.program;
            editor.state = if model.qvm.update(&prog) {
                editor.edit = prog;
                editor.error = false;
                State::Ready
            } else {
                editor.edit = prog;
                editor.error = true;
                State::Editing
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
    let err = |editor: &Editor| if editor.error {
        "ERROR!"
    } else {
        ""
    };
    let gates = match model.gates.state {
        State::Ready => html! {
            <button class="button", onclick=move|_| Msg::EditGates,>{"Edit Gates"}</button>
        },
        State::Editing => html! {
            <div>
                <div>{err(&model.gates)}</div>
                <button class="button", onclick=move|_| Msg::SaveGates,>{"Save Gates"}</button>
                <textarea id="gates", cols=30, rows=25,>{&model.gates.edit} </textarea>
            </div>
        },
    };

    let program = match model.program.state {
        State::Ready => html! {
            <div>
                <button class="button", onclick=move|_| Msg::EditProgram,>{"Edit Program"}</button>
                <br></br>
                <textarea disabled=true, id="program", cols=30, rows=25,>{&model.program.edit} </textarea>
            </div>
        },
        State::Editing => html! {
            <div>
                <div>{err(&model.program)}</div>
                <button class="button", onclick=move|_| Msg::SaveProgram,>{"Save Program"}</button>
                <br></br>
                <textarea id="program", cols=30, rows=25,>{&model.program.edit} </textarea>
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

