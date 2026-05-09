use dioxus::prelude::*;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        main {
            class: "min-h-screen bg-[#FBFAF7] flex flex-col items-center justify-center p-6",
            h1 {
                class: "text-4xl font-bold text-[#1F2937] tracking-wide",
                "Hello, betűk!"
            }
            p {
                class: "mt-4 text-lg text-[#1F2937]",
                "Mindjárt játszunk."
            }
        }
    }
}
