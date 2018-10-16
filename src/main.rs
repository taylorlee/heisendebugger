#![recursion_limit = "256"] // needed for html! macro expansion
#![allow(dead_code)]
#![allow(unused_imports)]

//#![feature(test)]
//extern crate test;

#[macro_use]
extern crate yew;
extern crate num_complex;
extern crate serde;
extern crate serde_json;
extern crate stdweb;

#[macro_use]
extern crate serde_derive;

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

    Beginning,
    Prev,
    Next,
    End,

    EditProgram,
    SaveProgram,

    EditGates,
    SaveGates,

    Bell,
    XYZ,
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
            edit: "x 0
cnot 0 1
x 3
swap 2 3"
                .to_string(),
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
fn reset_prog(model: &mut Model, prog: String) {
    model.qvm.update(&prog);
    model.program.edit = prog;
    model.qvm.reset();
}
fn update(_: &mut Context, model: &mut Model, msg: Msg) {
    match msg {
        Msg::Noop => {}
        Msg::Reset => {
            model.qvm.reset();
        }
        Msg::Bell => {
            reset_prog(model, "h 0\ncnot 0 1".to_string());
        }
        Msg::XYZ => {
            reset_prog(
                model,
                "x 0
y 0
z 0
x 1
y 1
z 1
h 0
h 1
".to_string(),
            );
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
        Msg::Beginning => loop {
            if model.qvm.counter == 0 {
                break;
            }
            model.qvm.prev();
        },
        Msg::Prev => {
            model.qvm.prev();
        }
        Msg::Next => {
            model.qvm.next();
        }
        Msg::End => loop {
            if model.qvm.counter == model.qvm.program.len() {
                break;
            }
            model.qvm.next();
        },
    }
}

fn view(model: &Model) -> Html<Msg> {
    let err = |editor: &Editor| if editor.error { "ERROR!" } else { "" };
    let gates = match model.gates.state {
        State::Ready => html! {
            <div class="level",>
                <div class="level-item",>
                    <button class="button", onclick=move|_| Msg::EditGates,>{"Edit Gates"}</button>
                </div>
            </div>
        },
        State::Editing => html! {
            <div>
                <div class="level",>
                    <div class="level-item",>
                        <button class="button", onclick=move|_| Msg::SaveGates,>{"Save Gates"}</button>
                    </div>
                </div>
                <div class="level",>
                    <div class="level-item",>
                        <div>{err(&model.gates)}</div>
                    </div>
                </div>
                <div class="level",>
                    <div class="level-item",>
                    <textarea id="gates", cols=30, rows=25,>{&model.gates.edit} </textarea>
                    </div>
                </div>
            </div>
        },
    };

    let program = match model.program.state {
        State::Ready => html! {
            <div>
                <div class="level",>
                    <div class="level-item",>
                    <button class="button", onclick=move|_| Msg::EditProgram,>{"Edit Program"}</button>
                    </div>
                </div>
                <div class="level",>
                    <div class="level-item",>
                    <span class=("tag","is-primary"),> {"counter: "} { model.qvm.counter } </span>
                    </div>
                </div>
                <div class="level",>
                    <div class="level-item",>
                    <textarea disabled=true, id="program", cols=30, rows=25,>{&model.program.edit} </textarea>
                    </div>
                </div>
            </div>
        },
        State::Editing => html! {
            <div>
                <div class="level",>
                    <div class="level-item",>
                        <button class="button", onclick=move|_| Msg::SaveProgram,>{"Save Program"}</button>
                    </div>
                </div>
                <div class="level",>
                    <div class="level-item",>
                        <div>{err(&model.program)}</div>
                    </div>
                </div>
                <div class="level",>
                    <div class="level-item",>
                        <textarea id="program", cols=30, rows=25,>{&model.program.edit} </textarea>
                    </div>
                </div>
            </div>
        },
    };

    let coeff = |n| {
        let value = model.qvm.state[n];
        if qvm::is_zero(value) {
            html! {
                <span>
                <span/>
            }
        } else {
            let (tens, co) = qvm::fmt_tensor(value, n);
            html! {
                <div class="level",>
                    <div class="level-item",>
                        <div class=("tags","has-addons"),>
                            <div class=("tag","is-primary"),>
                                 { tens }
                            </div>
                            <div class="tag",>
                                 { co }
                            </div>
                        </div>
                    </div>
                </div>
            }
        }
    };
    html! {
        <section class="section",>
            <div class="container",>
                <div class="level",>
                    <div class="level-item",>
                        <div class="select",>
                            <select>
                                <option selected=true, disabled=true,>{ "Example Program:" }</option>
                                <option onclick=move|_| Msg::Bell,>{"Bell"}</option>
                                <option onclick=move|_| Msg::XYZ,>{"XYZ"}</option>
                            </select>
                        </div>
                    </div>
                </div>

                <div class="level",>
                    <div class="level-item",>
                        { gates }
                    </div>
                </div>

                { program }

                <div class="level",>
                    <div class="level-item",>
                        <button class="button", onclick=move|_| Msg::Reset,>{ "Reset" }</button>
                        <button class="button", onclick=move|_| Msg::Beginning,>{ "<<" }</button>
                        <button class="button", onclick=move|_| Msg::Prev,>{ "<" }</button>
                        <button class="button", onclick=move|_| Msg::Next,>{ ">" }</button>
                        <button class="button", onclick=move|_| Msg::End,>{ ">>" }</button>
                    </div>
                </div>
                <div class="level",>
                    <div class="level-item",>
                        <div class="tag",>{"Quantum State: "}</div>
                    </div>
                </div>
                { for (0..model.qvm.state.len()).map(coeff) }
            </div>
        </section>
    }
}
