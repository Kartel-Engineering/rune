//! `Cx::on_gpu`: a view reads whether its frames draw on a GPU device, and re-runs when that
//! changes, so it can hold back an effect the CPU painter draws slowly.
use rune::{El, Env, Runtime, col, text};

fn view(_: &(), cx: &mut rune::Cx<()>) -> El {
    col().child(text(if cx.on_gpu() { "gpu" } else { "cpu" }).key("label"))
}

fn label(rt: &Runtime<()>) -> String {
    let e = rt
        .elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("label"))
        .unwrap();
    e.label.or(e.text).unwrap_or_default()
}

#[test]
fn a_view_reads_the_gpu_and_follows_a_change() {
    let mut rt = Runtime::new((), view, (200.0, 100.0), Env::new(false, 1.0));
    rt.frame(0.0);
    assert_eq!(label(&rt), "cpu", "headless draws on no GPU");
    rt.set_on_gpu(true);
    rt.frame(1.0 / 60.0);
    assert_eq!(label(&rt), "gpu", "a frame presented on a device");
    rt.set_on_gpu(false);
    rt.frame(1.0 / 60.0);
    assert_eq!(label(&rt), "cpu", "its device was lost");
}
