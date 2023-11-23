use std::error;
use std::rc::Rc;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use web_sys::console;
use wasm_bindgen::JsCast;
use rand::prelude::*;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // Your code goes here!
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    wasm_bindgen_futures::spawn_local(async move {
        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);

        let image = web_sys::HtmlImageElement::new().unwrap();

        let callback = Closure::once(move || {
            // lock(): Rc<Mutex<Option<Sender>>> -> Result<MutexGuard<Option<Sender>>, Error>
            // lock() := Rcを安全に外す関数
            // ok(): Result<MutexGuard<Option<Sender>>, Error> -> Option<MutexGuard<Option<Sender>>>
            // ok() := Result<T, E>をOption<T>に変換する関数
            // and_then: Option<MutexGuard<Option<Sender>>> -> MutexGuard<Option<Sender>>
            // and_then() := OptionがSomeだった場合にその中身を引数の関数に与える関数
            // take: MutexGuard<Option<Sender>> -> Option<Sender>
            // Mutexを外したのではなく, Mutexを無視して値を取り出しただけ.
            // 正確には変換しているわけではなく, 取り出して移している.
            // take() := Optionから値を取り出す関数
            if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
                success_tx.send(Ok(()));
            }
        });

        let error_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
                error_tx.send(Err(err));
            }
        });

        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        image.set_src("Idle (1).png");

        success_rx.await;
        context.draw_image_with_html_image_element(&image, 0.0, 0.0);

        sierpinski(&context, [(300.0, 0.0), (0.0, 600.0), (600.0, 600.0)], 4);
    });

    // JavaScript の console.log関数は可変個の引数を受け取るので, log_i とするとi個の引数をとるように指定できる.
    // ここでは1つの引数をとっている.
    // このようにJavaScript は varargs(可変個の引数)をサポートしているにも関わらず, Rust はサポートしていないので,
    // web-sysモジュールが複数のバリエーションを作ってくれている.
    console::log_1(&JsValue::from_str("Hello world!"));

    Ok(())
}

fn ave(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)
}

fn sierpinski(context: &web_sys::CanvasRenderingContext2d, points: [(f64, f64); 3], depth: usize) {
    if depth == 0 { return }

    let [top, left, right] = points;
    let mut rng = thread_rng();
    let color = (
        rng.gen_range(0..255),
        rng.gen_range(0..255),
        rng.gen_range(0..255)
    );

    draw_triangle(context, [top, ave(top, left), ave(top, right)], color);
    sierpinski(context, [top, ave(top, left), ave(top, right)], depth-1);
    draw_triangle(context, [ave(top, left), left, ave(left, right)], color);
    sierpinski(context, [ave(top, left), left, ave(left, right)], depth-1);
    draw_triangle(context, [ave(top, right), ave(left, right), right], color);
    sierpinski(context, [ave(top, right), ave(left, right), right], depth-1);
}

fn draw_triangle(context: &web_sys::CanvasRenderingContext2d, points: [(f64, f64); 3], color: (u8, u8, u8)) {
    let [top, left, right] = points;
    let color_str = format!("rgb({}, {}, {})", color.0, color.1, color.2);
    context.set_fill_style(&wasm_bindgen::JsValue::from_str(&color_str));
    context.move_to(top.0, top.1);
    context.begin_path();
    context.line_to(left.0, left.1);
    context.line_to(right.0, right.1);
    context.line_to(top.0, top.1);
    context.close_path();
    context.stroke();
    context.fill();
}