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
pub enum Example {
    Bell,
    SingleGates,
    DoubleGates,
    FullSuperPosition,
}

#[derive(Clone, Copy)]
pub enum Msg {
    Noop,
    Reset,

    Beginning,
    Prev,
    Next,
    End,

    Load(Example),
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
            edit: "".to_string(),
            error: false,
        },
    };
    model.qvm.update(&model.program.edit);
    model.gates.edit = model.qvm.show_gates();
    let mut ctx = Context {};
    update(&mut ctx, &mut model, Msg::Load(Example::Bell));
    app.mount(ctx, model, update, view);
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
        Msg::Load(ex) => {
            let prog = match ex {
                Example::Bell => "h 0
cnot 0 1".to_string(),
                Example::SingleGates => "x 0
y 1
z 2
h 3
x 4
y 5
z 6
h 7".to_string(),
                Example::DoubleGates => "x 0
cnot 0 1
swap 1 2
cnot 2 3
swap 3 4
cnot 4 5
swap 5 6
cnot 6 7
swap 7 0
cnot 0 7".to_string(),
                Example::FullSuperPosition => "h 0
h 1
h 2
h 3
h 4
h 5
h 6
h 7".to_string(),
            };
            reset_prog(model, prog);
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
    let instruction = |(i, line): (usize, &str)| {
        if i == model.qvm.counter {
            html! {
                <li><b>{ line }</b> <i class="has-text-info",>{"   (next instruction)"}</i></li>
            }
        } else {
            html! {
                <li>{ line }</li>
            }
        }
    };
    let coeff = |n| {
        let value = model.qvm.state[n];
        if qvm::is_zero(value) {
            html! {
                <div></div>
            }
        } else {
            let (tens, co) = qvm::fmt_tensor(value, n);
            html! {
                <div class="level",>
                    <div class="level-item",>
                        <div class=("tags","has-addons"),>
                            <div class=("tag","is-info"),>
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

    let lines = model.program.edit.to_string() + "\n \n";
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
                        <button class="button", onclick=move|_| Msg::Reset,>{ "Reset" }</button>
                        <button class="button", onclick=move|_| Msg::Beginning,>{ "<<" }</button>
                        <button class="button", onclick=move|_| Msg::Prev,>{ "<" }</button>
                        <button class="button", onclick=move|_| Msg::Next,>{ ">" }</button>
                        <button class="button", onclick=move|_| Msg::End,>{ ">>" }</button>
                    </div>
                </div>

                <div class="level",>
                    <div class="level-item",>
                        <div>{"Program: "}</div>
                    </div>
                </div>

                <div class="level",>
                    <div class="level-item",>
                        <div class="content",>
                            <ol>
                            { for lines.lines().enumerate().map(instruction) }
                            </ol>
                        </div>
                    </div>
                </div>
                <div class="level",>
                    <div class="level-item",>
                        <div>{"Quantum State: "}</div>
                    </div>
                </div>
                { for (0..model.qvm.state.len()).map(coeff) }
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
                        <textarea id="program", cols=30, rows=15,>{&model.program.edit} </textarea>
                    </div>
                </div>
            </div>
        },
    };

    html! {
        <section class="section",>
            <div class="container",>
                <div class="level",>
                    <div class="level-item",>
                        <div class="select",>
                            <select>
                                <option disabled=true,>{ "Example Program:" }</option>
                                <option selected=true, onclick=move|_| Msg::Load(Example::Bell),>{"Bell State"}</option>
                                <option onclick=move|_| Msg::Load(Example::SingleGates),>{"One Qubit Gates"}</option>
                                <option onclick=move|_| Msg::Load(Example::DoubleGates),>{"Two Qubit Gates"}</option>
                                <option onclick=move|_| Msg::Load(Example::FullSuperPosition),>{"Full Superposition"}</option>
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

            </div>
        </section>
    }
}

//#[cfg(test)]
//mod tests {
    //use super::*;
    //use test::Bencher;

    //// last run:
    
    ////test tests::bell        ... bench: 166,905,258 ns/iter (+/- 28,038,096)
    ////test tests::gate1       ... bench:   8,055,309 ns/iter (+/- 1,420,677)
    ////test tests::gate2       ... bench: 1,177,078,906 ns/iter (+/- 150,531,188)
    ////test tests::empty       ... bench: 172,861,382 ns/iter (+/- 24,259,132)
    
    //fn run_bench(prog: String) {
        //let mut model = Model {
            //qvm: qvm::QVM::new(),
            //gates: Editor {
                //state: State::Ready,
                //edit: "".to_string(),
                //error: false,
            //},
            //program: Editor {
                //state: State::Ready,
                //edit: "".to_string(),
                //error: false,
            //},
        //};
        //model.qvm.update(&model.program.edit);
        //model.gates.edit = model.qvm.show_gates();

        //reset_prog(&mut model, prog);
        //loop {
            //if model.qvm.counter == model.qvm.program.len() {
                //break;
            //}
            //model.qvm.next();
        //}
    //}
    //#[bench]
    //fn empty(b: &mut Bencher) {
        //b.iter(|| {
            //run_bench("swap 1 0".to_string());
        //});
    //}

    //#[bench]
    //fn bell(b: &mut Bencher) {
        //b.iter(|| {
            //run_bench("h 0
//cnot 0 1".to_string());
        //});
    //}
    //#[bench]
    //fn gate1(b: &mut Bencher) {
        //b.iter(|| {
            //run_bench("x 0
//y 1
//z 2
//h 3
//x 4
//y 5
//z 6
//h 7".into());
        //});
    //}
    //#[bench]
    //fn gate2(b: &mut Bencher) {
        //b.iter(|| {
            //run_bench("x 0
//cnot 0 1
//swap 1 2
//cnot 2 3
//swap 3 4
//cnot 4 5
//swap 5 6
//cnot 6 7".to_string());

//"swap 7 0
//cnot 0 7"
        //;
        //});
    //}
//}
